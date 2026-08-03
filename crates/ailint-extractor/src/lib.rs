//! Lexical extraction of source-code comments as virtual guidance documents.
//!
//! `ailint-extractor` uses [`logos`](https://crates.io/crates/logos) to tokenize
//! source files just enough to pull out line and block comments without building
//! a full AST. The returned [`Comment`]s can be fed to `ailint-core` as
//! "virtual documents" so semantic rules (vague instructions, negative overload,
//! bloat) can catch AI slop that leaks into inline comments and docstrings.
//!
//! Supported languages: [`Language::Rust`], [`Language::TypeScript`],
//! [`Language::JavaScript`], [`Language::Python`].
//!
//! ```
//! use ailint_extractor::{extract, Language, CommentKind};
//!
//! let src = "fn main() {\n    // TODO: refactor this later\n}\n";
//! let comments = extract(src, Language::Rust);
//! assert_eq!(comments.len(), 1);
//! assert_eq!(comments[0].kind, CommentKind::Line);
//! assert_eq!(comments[0].line, 2);
//! assert_eq!(comments[0].body().trim(), "TODO: refactor this later");
//! ```

#![warn(missing_docs)]

use std::ops::Range;
use std::path::Path;

mod lexers;

use lexers::{cs_comments, go_comments, java_comments, js_comments, py_comments, rust_comments};

/// Programming languages supported by the extractor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    /// Rust (`.rs`). Handles `//`, `///`, `//!`, `/* */`, `/** */`, `/*! */`,
    /// plus regular, raw, and byte string literals.
    Rust,
    /// TypeScript (`.ts`, `.tsx`). Handles `//`, `/* */`, `/** */`, plus single,
    /// double, and template string literals.
    TypeScript,
    /// JavaScript (`.js`, `.jsx`, `.mjs`, `.cjs`). Same handling as TypeScript.
    JavaScript,
    /// Python (`.py`). Handles `#` line comments and triple-quoted strings
    /// (surfaced as [`CommentKind::Docstring`]).
    Python,
    /// Go (`.go`). Handles `//`, `/* */`, plus `"…"`, raw `` `…` ``, and
    /// `'…'` rune literals. Go has no dedicated doc-comment syntax.
    Go,
    /// Java (`.java`). Handles `//`, `/* */`, `/** */` javadoc, plus `"…"`,
    /// `"""…"""` text blocks (Java 15+), and `'…'` char literals.
    Java,
    /// C# (`.cs`). Handles `//`, `///` XML doc, `/* */`, plus `"…"`, `@"…"`
    /// verbatim (with `""` escapes), `$"…"` interpolated (surface only),
    /// and `'…'` char literals.
    CSharp,
}

impl Language {
    /// Detect a supported language from a file path's extension.
    ///
    /// Returns `None` for unsupported extensions. Extension matching is
    /// case-insensitive.
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "rs" => Some(Self::Rust),
            "ts" | "tsx" | "mts" | "cts" => Some(Self::TypeScript),
            "js" | "jsx" | "mjs" | "cjs" => Some(Self::JavaScript),
            "py" | "pyi" => Some(Self::Python),
            "go" => Some(Self::Go),
            "java" => Some(Self::Java),
            "cs" => Some(Self::CSharp),
            _ => None,
        }
    }

    /// Short stable identifier for the language (`"rust"`, `"typescript"`, …).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::TypeScript => "typescript",
            Self::JavaScript => "javascript",
            Self::Python => "python",
            Self::Go => "go",
            Self::Java => "java",
            Self::CSharp => "csharp",
        }
    }
}

/// Classification of an extracted comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommentKind {
    /// A single-line comment: Rust/TS/JS `//`, Python `#`.
    Line,
    /// A block comment: `/* … */`. Not nested.
    Block,
    /// A documentation comment:
    /// - Rust: `///` (outer) or `//!` (inner), `/** … */`, `/*! … */`.
    /// - TS/JS: `/** … */` (JSDoc/TSDoc).
    Doc,
    /// A Python triple-quoted string (`""" … """` or `''' … '''`).
    /// Emitted regardless of syntactic position — the extractor does not
    /// distinguish docstrings from ordinary triple-quoted literals.
    Docstring,
}

/// A single extracted comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    /// The raw comment slice, including delimiters (`// foo`, `/* foo */`).
    pub raw: String,
    /// Classification of the comment.
    pub kind: CommentKind,
    /// Byte offsets within the source file.
    pub byte_range: Range<usize>,
    /// 1-based line number of the comment's start.
    pub line: usize,
}

impl Comment {
    /// The comment content with delimiters stripped, but internal whitespace
    /// preserved.
    ///
    /// - `// foo` -> `" foo"`
    /// - `/// foo` -> `" foo"`
    /// - `/* foo */` -> `" foo "`
    /// - `""" foo """` -> `" foo "`
    pub fn body(&self) -> &str {
        strip_delimiters(&self.raw, self.kind)
    }
}

fn strip_delimiters(raw: &str, kind: CommentKind) -> &str {
    match kind {
        CommentKind::Line => {
            if let Some(rest) = raw.strip_prefix("///") {
                rest
            } else if let Some(rest) = raw.strip_prefix("//!") {
                rest
            } else if let Some(rest) = raw.strip_prefix("//") {
                rest
            } else if let Some(rest) = raw.strip_prefix('#') {
                rest
            } else {
                raw
            }
        }
        CommentKind::Doc => {
            if let Some(rest) = raw.strip_prefix("///") {
                rest
            } else if let Some(rest) = raw.strip_prefix("//!") {
                rest
            } else if let Some(rest) = raw.strip_prefix("/**").and_then(|r| r.strip_suffix("*/")) {
                rest
            } else if let Some(rest) = raw.strip_prefix("/*!").and_then(|r| r.strip_suffix("*/")) {
                rest
            } else {
                raw
            }
        }
        CommentKind::Block => raw
            .strip_prefix("/*")
            .and_then(|r| r.strip_suffix("*/"))
            .unwrap_or(raw),
        CommentKind::Docstring => raw
            .strip_prefix("\"\"\"")
            .and_then(|r| r.strip_suffix("\"\"\""))
            .or_else(|| raw.strip_prefix("'''").and_then(|r| r.strip_suffix("'''")))
            .unwrap_or(raw),
    }
}

/// Extract every comment from `source` for the given `language`.
///
/// Comments are returned in source order. Byte ranges refer to the input
/// `source`. Line numbers are 1-based.
pub fn extract(source: &str, language: Language) -> Vec<Comment> {
    let raw_comments = match language {
        Language::Rust => rust_comments(source),
        Language::TypeScript | Language::JavaScript => js_comments(source),
        Language::Python => py_comments(source),
        Language::Go => go_comments(source),
        Language::Java => java_comments(source),
        Language::CSharp => cs_comments(source),
    };
    attach_lines(source, raw_comments)
}

/// A raw comment span produced by a language lexer, before line-number
/// annotation. Kept crate-internal.
pub(crate) struct RawComment {
    pub raw: String,
    pub kind: CommentKind,
    pub byte_range: Range<usize>,
}

fn attach_lines(source: &str, raws: Vec<RawComment>) -> Vec<Comment> {
    if raws.is_empty() {
        return Vec::new();
    }
    let bytes = source.as_bytes();
    let mut newline_offsets: Vec<usize> = bytes
        .iter()
        .enumerate()
        .filter_map(|(i, b)| if *b == b'\n' { Some(i) } else { None })
        .collect();
    newline_offsets.push(bytes.len());

    raws.into_iter()
        .map(|r| {
            let line = 1 + newline_offsets
                .binary_search(&r.byte_range.start)
                .unwrap_or_else(|idx| idx);
            Comment {
                raw: r.raw,
                kind: r.kind,
                byte_range: r.byte_range,
                line,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn language_from_path_covers_common_extensions() {
        for (path, lang) in [
            ("src/lib.rs", Language::Rust),
            ("src/App.tsx", Language::TypeScript),
            ("index.ts", Language::TypeScript),
            ("index.js", Language::JavaScript),
            ("bundle.mjs", Language::JavaScript),
            ("main.py", Language::Python),
            ("types.pyi", Language::Python),
            ("cmd/main.go", Language::Go),
            ("App.java", Language::Java),
            ("Program.cs", Language::CSharp),
        ] {
            assert_eq!(
                Language::from_path(&PathBuf::from(path)),
                Some(lang),
                "path={path}"
            );
        }
    }

    #[test]
    fn language_from_path_rejects_unknown() {
        assert!(Language::from_path(&PathBuf::from("README.md")).is_none());
        assert!(Language::from_path(&PathBuf::from("Cargo.toml")).is_none());
        assert!(Language::from_path(&PathBuf::from("noext")).is_none());
    }

    #[test]
    fn language_as_str_stable() {
        assert_eq!(Language::Rust.as_str(), "rust");
        assert_eq!(Language::TypeScript.as_str(), "typescript");
        assert_eq!(Language::JavaScript.as_str(), "javascript");
        assert_eq!(Language::Python.as_str(), "python");
        assert_eq!(Language::Go.as_str(), "go");
        assert_eq!(Language::Java.as_str(), "java");
        assert_eq!(Language::CSharp.as_str(), "csharp");
    }

    #[test]
    fn body_strips_line_markers() {
        let c = Comment {
            raw: "/// docstring".into(),
            kind: CommentKind::Doc,
            byte_range: 0..13,
            line: 1,
        };
        assert_eq!(c.body(), " docstring");
    }

    #[test]
    fn body_strips_block_markers() {
        let c = Comment {
            raw: "/* hi */".into(),
            kind: CommentKind::Block,
            byte_range: 0..8,
            line: 1,
        };
        assert_eq!(c.body(), " hi ");
    }

    #[test]
    fn body_strips_python_docstring() {
        let c = Comment {
            raw: "\"\"\"module\"\"\"".into(),
            kind: CommentKind::Docstring,
            byte_range: 0..12,
            line: 1,
        };
        assert_eq!(c.body(), "module");
    }

    #[test]
    fn empty_source_returns_empty_vec() {
        assert!(extract("", Language::Rust).is_empty());
        assert!(extract("", Language::Python).is_empty());
        assert!(extract("", Language::TypeScript).is_empty());
    }
}
