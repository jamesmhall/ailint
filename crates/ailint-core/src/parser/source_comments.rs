//! Synthesize a [`MarkdownDoc`] from source-code comments so prose-oriented
//! rules (bloat, vague instruction, negative overload) can run against them
//! without change.
//!
//! Each extracted comment becomes one `Paragraph`. Byte ranges and line
//! numbers refer to the original source file so reporters point at the real
//! line the offending comment lives on.

use ailint_extractor::{extract, Comment, CommentKind, Language};

use crate::parser::markdown::{ListItem, MarkdownDoc, Paragraph};

/// Extract comments from `source` for `language` and pack them into a
/// [`MarkdownDoc`] whose `paragraphs` list one comment each. Each comment
/// is also mirrored into `list_items` so rules that currently scan bullets
/// (AIL100 vague-instruction, AIL104 negative-constraint-overload) fire on
/// source-code comments without change.
pub fn synthesize(source: &str, language: Language) -> MarkdownDoc {
    let mut doc = MarkdownDoc::default();
    for c in extract(source, language) {
        let byte_range = c.byte_range.clone();
        let line = c.line;
        let Some(text) = comment_prose(c) else {
            continue;
        };
        doc.paragraphs.push(Paragraph {
            text: text.clone(),
            byte_range: byte_range.clone(),
            line,
        });
        doc.list_items.push(ListItem {
            text,
            byte_range,
            line,
        });
    }
    doc
}

fn comment_prose(c: Comment) -> Option<String> {
    let text = normalize_body(c.body(), c.kind);
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Reduce a comment body to a single line of prose for rule inspection:
/// - Line and doc comments -> raw body trimmed.
/// - Block, doc-block, and Python docstrings -> collapse internal newlines
///   and strip the leading `*` gutter common in JSDoc / Rustdoc block
///   comments so each `*` line does not become a false-positive "paragraph
///   break" once we join.
fn normalize_body(body: &str, kind: CommentKind) -> String {
    match kind {
        CommentKind::Line => body.trim().to_string(),
        CommentKind::Doc | CommentKind::Block | CommentKind::Docstring => body
            .lines()
            .map(strip_star_gutter)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
    }
}

fn strip_star_gutter(line: &str) -> &str {
    let trimmed = line.trim_start();
    trimmed
        .strip_prefix("* ")
        .unwrap_or_else(|| if trimmed == "*" { "" } else { line })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_comments_become_paragraphs() {
        let doc = synthesize("// hello\n// world\n", Language::Rust);
        assert_eq!(doc.paragraphs.len(), 2);
        assert_eq!(doc.list_items.len(), 2);
        assert_eq!(doc.paragraphs[0].text, "hello");
        assert_eq!(doc.paragraphs[0].line, 1);
        assert_eq!(doc.paragraphs[1].text, "world");
        assert_eq!(doc.paragraphs[1].line, 2);
        assert_eq!(doc.list_items[0].text, "hello");
    }

    #[test]
    fn jsdoc_block_gutter_stripped_and_joined() {
        let src = "\
/**
 * First line.
 * Second line.
 */
const x = 1;
";
        let doc = synthesize(src, Language::TypeScript);
        assert_eq!(doc.paragraphs.len(), 1);
        assert_eq!(doc.paragraphs[0].text, "First line. Second line.");
        assert_eq!(doc.paragraphs[0].line, 1);
    }

    #[test]
    fn python_docstring_folded_to_single_paragraph() {
        let src = "\
def f():
    \"\"\"
    First sentence.
    Second sentence.
    \"\"\"
    return 1
";
        let doc = synthesize(src, Language::Python);
        assert_eq!(doc.paragraphs.len(), 1);
        assert_eq!(doc.paragraphs[0].text, "First sentence. Second sentence.");
    }

    #[test]
    fn empty_comment_is_skipped() {
        let doc = synthesize("//\n// real\n//    \n", Language::Rust);
        assert_eq!(doc.paragraphs.len(), 1);
        assert_eq!(doc.paragraphs[0].text, "real");
    }
}
