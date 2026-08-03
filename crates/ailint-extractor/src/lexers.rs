//! Logos-based lexers that identify comment spans while ignoring strings.
//!
//! Each language exposes one function returning [`RawComment`]s in source
//! order. Non-comment tokens are consumed to skip past string literals so
//! `//` or `#` inside a string does not turn into a false-positive comment.
//!
//! Nested block comments (`/* /* */ */`) are **not** handled — the outer
//! `*/` closes the first `/*`. Rust allows nesting but it is rare in
//! practice; we accept the limitation rather than growing the lexer.

use logos::{Lexer, Logos};

use crate::{CommentKind, RawComment};

// -------- Rust --------

#[derive(Logos, Debug, PartialEq)]
enum RustTok {
    // Line comment — Doc vs Line resolved after lex.
    #[regex(r"//[^\n]*")]
    LineComment,

    // Block comment; callback advances past `*/`.
    #[token("/*", block_comment)]
    BlockComment,

    // Cook, byte, and c strings all share the same escape rules for our
    // purposes (we only care about consuming them).
    #[regex(r#""([^"\\]|\\.)*""#)]
    #[regex(r#"b"([^"\\]|\\.)*""#)]
    String,

    // Char / byte-char literals: skip so `'/' '/'` never confuses us.
    #[regex(r"'([^'\\]|\\.)'")]
    #[regex(r"b'([^'\\]|\\.)'")]
    Char,

    // Raw and byte-raw strings: r"…", r#"…"#, r##"…"##, br"…", etc.
    #[token("r\"", raw_string)]
    #[token("r#", raw_hash_string)]
    #[token("br\"", raw_string)]
    #[token("br#", raw_hash_string)]
    RawString,
}

fn block_comment(lex: &mut Lexer<RustTok>) -> Option<()> {
    let rem = lex.remainder();
    let end = rem.find("*/")?;
    lex.bump(end + 2);
    Some(())
}

fn raw_string(lex: &mut Lexer<RustTok>) -> Option<()> {
    let rem = lex.remainder();
    let end = rem.find('"')?;
    lex.bump(end + 1);
    Some(())
}

fn raw_hash_string(lex: &mut Lexer<RustTok>) -> Option<()> {
    // Count leading `#`s (we already ate one via the `r#` / `br#` token).
    let rem = lex.remainder();
    let extra_hashes = rem.bytes().take_while(|b| *b == b'#').count();
    // Skip the extra hashes plus the opening `"`.
    let after_hashes = &rem[extra_hashes..];
    if !after_hashes.starts_with('"') {
        return None;
    }
    let total_hashes = 1 + extra_hashes;
    let terminator: String = std::iter::once('"')
        .chain(std::iter::repeat('#').take(total_hashes))
        .collect();
    let body = &after_hashes[1..];
    let end = body.find(&terminator)?;
    lex.bump(extra_hashes + 1 + end + terminator.len());
    Some(())
}

pub(crate) fn rust_comments(source: &str) -> Vec<RawComment> {
    let mut out = Vec::new();
    let mut lex = RustTok::lexer(source);
    while let Some(tok) = lex.next() {
        match tok {
            Ok(RustTok::LineComment) => {
                let span = lex.span();
                let raw = source[span.clone()].to_string();
                let kind = if raw.starts_with("////") {
                    // Four+ slashes is not a doc comment in Rust.
                    CommentKind::Line
                } else if raw.starts_with("///") || raw.starts_with("//!") {
                    CommentKind::Doc
                } else {
                    CommentKind::Line
                };
                out.push(RawComment {
                    raw,
                    kind,
                    byte_range: span,
                });
            }
            Ok(RustTok::BlockComment) => {
                let span = lex.span();
                let raw = source[span.clone()].to_string();
                let is_outer_doc = raw.starts_with("/**") && !raw.starts_with("/***");
                let is_inner_doc = raw.starts_with("/*!");
                let kind = if is_outer_doc || is_inner_doc {
                    CommentKind::Doc
                } else {
                    CommentKind::Block
                };
                out.push(RawComment {
                    raw,
                    kind,
                    byte_range: span,
                });
            }
            _ => {}
        }
    }
    out
}

// -------- JavaScript / TypeScript --------

#[derive(Logos, Debug, PartialEq)]
enum JsTok {
    #[regex(r"//[^\n]*")]
    LineComment,

    #[token("/*", block_comment_js)]
    BlockComment,

    #[regex(r#""([^"\\\n]|\\.)*""#)]
    #[regex(r"'([^'\\\n]|\\.)*'")]
    String,

    // Template literals. We do not descend into `${…}` expressions, so a `//`
    // inside a `${}` interpolation could be missed. Accepted tradeoff.
    #[token("`", template_string)]
    Template,
}

fn block_comment_js(lex: &mut Lexer<JsTok>) -> Option<()> {
    let rem = lex.remainder();
    let end = rem.find("*/")?;
    lex.bump(end + 2);
    Some(())
}

fn template_string(lex: &mut Lexer<JsTok>) -> Option<()> {
    let rem = lex.remainder();
    let mut i = 0;
    let bytes = rem.as_bytes();
    while i < bytes.len() {
        match bytes[i] {
            b'\\' if i + 1 < bytes.len() => i += 2,
            b'`' => {
                lex.bump(i + 1);
                return Some(());
            }
            _ => i += 1,
        }
    }
    None
}

pub(crate) fn js_comments(source: &str) -> Vec<RawComment> {
    let mut out = Vec::new();
    let mut lex = JsTok::lexer(source);
    while let Some(tok) = lex.next() {
        match tok {
            Ok(JsTok::LineComment) => {
                let span = lex.span();
                let raw = source[span.clone()].to_string();
                out.push(RawComment {
                    raw,
                    kind: CommentKind::Line,
                    byte_range: span,
                });
            }
            Ok(JsTok::BlockComment) => {
                let span = lex.span();
                let raw = source[span.clone()].to_string();
                let kind = if raw.starts_with("/**") && !raw.starts_with("/***") && raw != "/**/" {
                    CommentKind::Doc
                } else {
                    CommentKind::Block
                };
                out.push(RawComment {
                    raw,
                    kind,
                    byte_range: span,
                });
            }
            _ => {}
        }
    }
    out
}

// -------- Python --------

#[derive(Logos, Debug, PartialEq)]
enum PyTok {
    #[regex(r"#[^\n]*")]
    LineComment,

    // Triple-quoted strings come before regular strings so they win on length.
    #[token("\"\"\"", triple_double)]
    #[token("'''", triple_single)]
    TripleString,

    #[regex(r#""([^"\\\n]|\\.)*""#)]
    #[regex(r"'([^'\\\n]|\\.)*'")]
    String,
}

fn triple_double(lex: &mut Lexer<PyTok>) -> Option<()> {
    let rem = lex.remainder();
    let end = rem.find("\"\"\"")?;
    lex.bump(end + 3);
    Some(())
}

fn triple_single(lex: &mut Lexer<PyTok>) -> Option<()> {
    let rem = lex.remainder();
    let end = rem.find("'''")?;
    lex.bump(end + 3);
    Some(())
}

pub(crate) fn py_comments(source: &str) -> Vec<RawComment> {
    let mut out = Vec::new();
    let mut lex = PyTok::lexer(source);
    while let Some(tok) = lex.next() {
        match tok {
            Ok(PyTok::LineComment) => {
                let span = lex.span();
                let raw = source[span.clone()].to_string();
                out.push(RawComment {
                    raw,
                    kind: CommentKind::Line,
                    byte_range: span,
                });
            }
            Ok(PyTok::TripleString) => {
                let span = lex.span();
                let raw = source[span.clone()].to_string();
                out.push(RawComment {
                    raw,
                    kind: CommentKind::Docstring,
                    byte_range: span,
                });
            }
            _ => {}
        }
    }
    out
}

// -------- Go --------

#[derive(Logos, Debug, PartialEq)]
enum GoTok {
    #[regex(r"//[^\n]*")]
    LineComment,

    #[token("/*", block_comment_go)]
    BlockComment,

    #[regex(r#""([^"\\\n]|\\.)*""#)]
    String,

    // Raw string literals: `…` — no escapes, may contain newlines.
    #[token("`", raw_string_go)]
    RawString,

    // Rune literal: single-quoted character with optional escape.
    #[regex(r"'([^'\\]|\\.)*'")]
    Rune,
}

fn block_comment_go(lex: &mut Lexer<GoTok>) -> Option<()> {
    let rem = lex.remainder();
    let end = rem.find("*/")?;
    lex.bump(end + 2);
    Some(())
}

fn raw_string_go(lex: &mut Lexer<GoTok>) -> Option<()> {
    let rem = lex.remainder();
    let end = rem.find('`')?;
    lex.bump(end + 1);
    Some(())
}

pub(crate) fn go_comments(source: &str) -> Vec<RawComment> {
    let mut out = Vec::new();
    let mut lex = GoTok::lexer(source);
    while let Some(tok) = lex.next() {
        match tok {
            Ok(GoTok::LineComment) => {
                let span = lex.span();
                out.push(RawComment {
                    raw: source[span.clone()].to_string(),
                    kind: CommentKind::Line,
                    byte_range: span,
                });
            }
            Ok(GoTok::BlockComment) => {
                let span = lex.span();
                out.push(RawComment {
                    raw: source[span.clone()].to_string(),
                    kind: CommentKind::Block,
                    byte_range: span,
                });
            }
            _ => {}
        }
    }
    out
}

// -------- Java --------

#[derive(Logos, Debug, PartialEq)]
enum JavaTok {
    #[regex(r"//[^\n]*")]
    LineComment,

    #[token("/*", block_comment_java)]
    BlockComment,

    // Text blocks come before regular strings so they win on length.
    #[token("\"\"\"", text_block_java)]
    TextBlock,

    #[regex(r#""([^"\\\n]|\\.)*""#)]
    String,

    #[regex(r"'([^'\\]|\\.)'")]
    Char,
}

fn block_comment_java(lex: &mut Lexer<JavaTok>) -> Option<()> {
    let rem = lex.remainder();
    let end = rem.find("*/")?;
    lex.bump(end + 2);
    Some(())
}

fn text_block_java(lex: &mut Lexer<JavaTok>) -> Option<()> {
    let rem = lex.remainder();
    let end = rem.find("\"\"\"")?;
    lex.bump(end + 3);
    Some(())
}

pub(crate) fn java_comments(source: &str) -> Vec<RawComment> {
    let mut out = Vec::new();
    let mut lex = JavaTok::lexer(source);
    while let Some(tok) = lex.next() {
        match tok {
            Ok(JavaTok::LineComment) => {
                let span = lex.span();
                out.push(RawComment {
                    raw: source[span.clone()].to_string(),
                    kind: CommentKind::Line,
                    byte_range: span,
                });
            }
            Ok(JavaTok::BlockComment) => {
                let span = lex.span();
                let raw = source[span.clone()].to_string();
                let kind = if raw.starts_with("/**") && !raw.starts_with("/***") && raw != "/**/" {
                    CommentKind::Doc
                } else {
                    CommentKind::Block
                };
                out.push(RawComment {
                    raw,
                    kind,
                    byte_range: span,
                });
            }
            _ => {}
        }
    }
    out
}

// -------- C# --------

#[derive(Logos, Debug, PartialEq)]
enum CsTok {
    #[regex(r"//[^\n]*")]
    LineComment,

    #[token("/*", block_comment_cs)]
    BlockComment,

    // Raw string literals (C# 11+): `"""…"""`. Must come before regular strings.
    #[token("\"\"\"", raw_string_cs)]
    RawString,

    // Verbatim string: `@"…"` with `""` as escape for a literal quote.
    #[token("@\"", verbatim_string_cs)]
    Verbatim,

    // Interpolated verbatim: `$@"…"` / `@$"…"`. Treated like a verbatim string
    // — we don't descend into `{…}` interpolations.
    #[token("$@\"", verbatim_string_cs)]
    #[token("@$\"", verbatim_string_cs)]
    InterpVerbatim,

    // Regular and interpolated strings share the same escape rules for our
    // purposes (we don't parse `{expr}` in `$"…"`).
    #[regex(r#""([^"\\\n]|\\.)*""#)]
    #[regex(r#"\$"([^"\\\n]|\\.)*""#)]
    String,

    #[regex(r"'([^'\\]|\\.)'")]
    Char,
}

fn block_comment_cs(lex: &mut Lexer<CsTok>) -> Option<()> {
    let rem = lex.remainder();
    let end = rem.find("*/")?;
    lex.bump(end + 2);
    Some(())
}

fn raw_string_cs(lex: &mut Lexer<CsTok>) -> Option<()> {
    let rem = lex.remainder();
    let end = rem.find("\"\"\"")?;
    lex.bump(end + 3);
    Some(())
}

fn verbatim_string_cs(lex: &mut Lexer<CsTok>) -> Option<()> {
    // `""` inside a verbatim string is an escaped quote, not a terminator.
    let rem = lex.remainder();
    let bytes = rem.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'"' {
                i += 2;
                continue;
            }
            lex.bump(i + 1);
            return Some(());
        }
        i += 1;
    }
    None
}

pub(crate) fn cs_comments(source: &str) -> Vec<RawComment> {
    let mut out = Vec::new();
    let mut lex = CsTok::lexer(source);
    while let Some(tok) = lex.next() {
        match tok {
            Ok(CsTok::LineComment) => {
                let span = lex.span();
                let raw = source[span.clone()].to_string();
                // `////` and more are not XML doc, per Roslyn.
                let kind = if raw.starts_with("////") {
                    CommentKind::Line
                } else if raw.starts_with("///") {
                    CommentKind::Doc
                } else {
                    CommentKind::Line
                };
                out.push(RawComment {
                    raw,
                    kind,
                    byte_range: span,
                });
            }
            Ok(CsTok::BlockComment) => {
                let span = lex.span();
                out.push(RawComment {
                    raw: source[span.clone()].to_string(),
                    kind: CommentKind::Block,
                    byte_range: span,
                });
            }
            _ => {}
        }
    }
    out
}
