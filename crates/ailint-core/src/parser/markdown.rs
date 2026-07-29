//! Markdown parsing via `pulldown-cmark`.
//!
//! TODO: build a lightweight AST (sections, headings, code blocks, frontmatter)
//! that rules can query without touching pulldown-cmark directly.

/// Placeholder for the parsed representation.
#[derive(Debug, Clone, Default)]
pub struct MarkdownDoc {
    pub headings: Vec<String>,
    // TODO: sections with byte ranges, code blocks, list items, frontmatter.
}

/// Parse a Markdown string into a [`MarkdownDoc`].
///
/// TODO: split YAML frontmatter (delimited by `---`) from the body, then walk
/// the pulldown-cmark event stream to populate the AST.
pub fn parse(_input: &str) -> MarkdownDoc {
    MarkdownDoc::default()
}
