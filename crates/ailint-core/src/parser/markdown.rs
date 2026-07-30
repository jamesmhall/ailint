//! Markdown parsing via `pulldown-cmark`.

use std::ops::Range;

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

/// Structural index of a Markdown document, built in one parse pass.
#[derive(Debug, Clone, Default)]
pub struct MarkdownDoc {
    /// YAML frontmatter block, if the file starts with `---`.
    pub frontmatter: Option<Frontmatter>,
    /// All headings, in document order.
    pub headings: Vec<Heading>,
    /// Heading-delimited spans covering the whole body.
    pub sections: Vec<Section>,
    /// Fenced and indented code blocks.
    pub code_blocks: Vec<CodeBlock>,
    /// Bullet and ordered list items, flattened.
    pub list_items: Vec<ListItem>,
    /// All hyperlinks.
    pub links: Vec<Link>,
}

/// Raw YAML frontmatter (`---` fenced) at the top of a document.
#[derive(Debug, Clone)]
pub struct Frontmatter {
    /// Frontmatter body without the `---` fences.
    pub raw: String,
    /// Byte span in the original input, fences included.
    pub byte_range: Range<usize>,
}

/// A Markdown heading.
#[derive(Debug, Clone)]
pub struct Heading {
    /// Heading depth, 1–6.
    pub level: u8,
    /// Heading text with inline markup stripped.
    pub text: String,
    /// Byte span in the original input.
    pub byte_range: Range<usize>,
    /// 1-based line number.
    pub line: usize,
}

/// A span of the body owned by one heading (or the preamble).
#[derive(Debug, Clone)]
pub struct Section {
    /// Index into [`MarkdownDoc::headings`]; `None` for pre-heading content.
    pub heading_index: Option<usize>,
    /// Byte span in the original input.
    pub byte_range: Range<usize>,
}

/// A fenced or indented code block.
#[derive(Debug, Clone)]
pub struct CodeBlock {
    /// Info-string language tag, if any.
    pub lang: Option<String>,
    /// Code block contents.
    pub text: String,
    /// Byte span in the original input.
    pub byte_range: Range<usize>,
    /// 1-based line number.
    pub line: usize,
}

/// A single list item (bullet or ordered), flattened.
#[derive(Debug, Clone)]
pub struct ListItem {
    /// Item text with inline markup stripped.
    pub text: String,
    /// Byte span in the original input.
    pub byte_range: Range<usize>,
    /// 1-based line number.
    pub line: usize,
}

/// A hyperlink discovered in the document (`[text](url)` or reference form).
#[derive(Debug, Clone)]
pub struct Link {
    /// Link destination as written.
    pub url: String,
    /// Link text.
    pub text: String,
    /// Byte span in the original input.
    pub byte_range: Range<usize>,
    /// 1-based line number.
    pub line: usize,
}

/// Parse `input` into a [`MarkdownDoc`].
pub fn parse(input: &str) -> MarkdownDoc {
    let (frontmatter, body_start) = split_frontmatter(input);
    let body = &input[body_start..];
    let line_starts = compute_line_starts(input);

    let mut doc = MarkdownDoc {
        frontmatter,
        headings: Vec::new(),
        sections: Vec::new(),
        code_blocks: Vec::new(),
        list_items: Vec::new(),
        links: Vec::new(),
    };

    let mut heading_stack: Vec<PendingHeading> = Vec::new();
    let mut code_stack: Vec<PendingCodeBlock> = Vec::new();
    let mut item_stack: Vec<PendingListItem> = Vec::new();
    let mut link_stack: Vec<PendingLink> = Vec::new();

    for (event, raw_range) in Parser::new_ext(body, Options::empty()).into_offset_iter() {
        let range = (raw_range.start + body_start)..(raw_range.end + body_start);
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                heading_stack.push(PendingHeading {
                    level: heading_level_u8(level),
                    text: String::new(),
                    start: range.start,
                });
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(pending) = heading_stack.pop() {
                    let line = offset_to_line(&line_starts, pending.start);
                    doc.headings.push(Heading {
                        level: pending.level,
                        text: pending.text.trim().to_string(),
                        byte_range: pending.start..range.end,
                        line,
                    });
                }
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(l) => {
                        let s = l.to_string();
                        if s.is_empty() {
                            None
                        } else {
                            Some(s)
                        }
                    }
                    CodeBlockKind::Indented => None,
                };
                code_stack.push(PendingCodeBlock {
                    lang,
                    text: String::new(),
                    start: range.start,
                });
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(pending) = code_stack.pop() {
                    let line = offset_to_line(&line_starts, pending.start);
                    doc.code_blocks.push(CodeBlock {
                        lang: pending.lang,
                        text: pending.text,
                        byte_range: pending.start..range.end,
                        line,
                    });
                }
            }
            Event::Start(Tag::Item) => {
                item_stack.push(PendingListItem {
                    text: String::new(),
                    start: range.start,
                });
            }
            Event::End(TagEnd::Item) => {
                if let Some(pending) = item_stack.pop() {
                    let line = offset_to_line(&line_starts, pending.start);
                    doc.list_items.push(ListItem {
                        text: pending.text.trim().to_string(),
                        byte_range: pending.start..range.end,
                        line,
                    });
                }
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                link_stack.push(PendingLink {
                    url: dest_url.to_string(),
                    text: String::new(),
                    start: range.start,
                });
            }
            Event::End(TagEnd::Link) => {
                if let Some(pending) = link_stack.pop() {
                    let line = offset_to_line(&line_starts, pending.start);
                    doc.links.push(Link {
                        url: pending.url,
                        text: pending.text.trim().to_string(),
                        byte_range: pending.start..range.end,
                        line,
                    });
                }
            }
            Event::Text(t) => {
                if let Some(c) = code_stack.last_mut() {
                    c.text.push_str(&t);
                } else {
                    if let Some(h) = heading_stack.last_mut() {
                        h.text.push_str(&t);
                    }
                    if let Some(li) = item_stack.last_mut() {
                        li.text.push_str(&t);
                    }
                    if let Some(lk) = link_stack.last_mut() {
                        lk.text.push_str(&t);
                    }
                }
            }
            Event::Code(t) => {
                if let Some(h) = heading_stack.last_mut() {
                    h.text.push_str(&t);
                }
                if let Some(li) = item_stack.last_mut() {
                    li.text.push_str(&t);
                }
                if let Some(lk) = link_stack.last_mut() {
                    lk.text.push_str(&t);
                }
            }
            _ => {}
        }
    }

    doc.sections = build_sections(&doc.headings, body_start, input.len());
    doc
}

struct PendingHeading {
    level: u8,
    text: String,
    start: usize,
}

struct PendingCodeBlock {
    lang: Option<String>,
    text: String,
    start: usize,
}

struct PendingListItem {
    text: String,
    start: usize,
}

struct PendingLink {
    url: String,
    text: String,
    start: usize,
}

fn heading_level_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn build_sections(headings: &[Heading], body_start: usize, file_end: usize) -> Vec<Section> {
    let mut sections = Vec::new();
    let first_start = headings
        .first()
        .map(|h| h.byte_range.start)
        .unwrap_or(file_end);
    if body_start < first_start {
        sections.push(Section {
            heading_index: None,
            byte_range: body_start..first_start,
        });
    }
    for (i, h) in headings.iter().enumerate() {
        let end = headings
            .get(i + 1)
            .map(|nh| nh.byte_range.start)
            .unwrap_or(file_end);
        sections.push(Section {
            heading_index: Some(i),
            byte_range: h.byte_range.end..end,
        });
    }
    sections
}

fn compute_line_starts(input: &str) -> Vec<usize> {
    let mut v = Vec::with_capacity(64);
    v.push(0);
    for (i, b) in input.bytes().enumerate() {
        if b == b'\n' {
            v.push(i + 1);
        }
    }
    v
}

fn offset_to_line(starts: &[usize], offset: usize) -> usize {
    match starts.binary_search(&offset) {
        Ok(i) => i + 1,
        Err(i) => i.max(1),
    }
}

fn split_frontmatter(input: &str) -> (Option<Frontmatter>, usize) {
    let bom_len = if input.starts_with('\u{FEFF}') {
        '\u{FEFF}'.len_utf8()
    } else {
        0
    };
    let after_bom = &input[bom_len..];

    let open_len = if after_bom.starts_with("---\r\n") {
        5
    } else if after_bom.starts_with("---\n") {
        4
    } else {
        return (None, 0);
    };

    let body_after_open = &after_bom[open_len..];
    let mut cursor = 0usize;
    while cursor <= body_after_open.len() {
        let rest = &body_after_open[cursor..];
        let (line, consumed) = match rest.find('\n') {
            Some(nl) => (&rest[..nl], nl + 1),
            None => (rest, rest.len()),
        };
        let content = line.strip_suffix('\r').unwrap_or(line);
        if content == "---" {
            let close_end = cursor + consumed;
            let raw_slice = &body_after_open[..cursor];
            let raw_trimmed = raw_slice.strip_suffix('\n').unwrap_or(raw_slice);
            let raw_trimmed = raw_trimmed.strip_suffix('\r').unwrap_or(raw_trimmed);
            let full_end = bom_len + open_len + close_end;
            return (
                Some(Frontmatter {
                    raw: raw_trimmed.to_string(),
                    byte_range: bom_len..full_end,
                }),
                full_end,
            );
        }
        if consumed == 0 {
            break;
        }
        cursor += consumed;
    }
    (None, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_frontmatter_when_present() {
        let src = "---\ntitle: Hi\nkey: val\n---\n# Heading\n";
        let doc = parse(src);
        let fm = doc.frontmatter.expect("frontmatter");
        assert_eq!(fm.raw, "title: Hi\nkey: val");
        assert_eq!(fm.byte_range.start, 0);
        assert_eq!(
            &src[fm.byte_range.clone()],
            "---\ntitle: Hi\nkey: val\n---\n"
        );
        assert_eq!(doc.headings.len(), 1);
        assert_eq!(doc.headings[0].text, "Heading");
    }

    #[test]
    fn no_frontmatter_when_missing_closing_fence() {
        let src = "---\ntitle: unterminated\n# Heading\n";
        let doc = parse(src);
        assert!(doc.frontmatter.is_none());
    }

    #[test]
    fn no_frontmatter_when_not_delimited() {
        let src = "# Heading\n\nBody text.\n";
        let doc = parse(src);
        assert!(doc.frontmatter.is_none());
        assert_eq!(doc.headings.len(), 1);
    }

    #[test]
    fn extracts_headings_with_line_numbers() {
        let src = "# One\n\n## Two\n\nBody\n\n### Three\n";
        let doc = parse(src);
        assert_eq!(doc.headings.len(), 3);
        assert_eq!(doc.headings[0].level, 1);
        assert_eq!(doc.headings[0].text, "One");
        assert_eq!(doc.headings[0].line, 1);
        assert_eq!(doc.headings[1].level, 2);
        assert_eq!(doc.headings[1].line, 3);
        assert_eq!(doc.headings[2].level, 3);
        assert_eq!(doc.headings[2].line, 7);
    }

    #[test]
    fn sections_span_between_headings() {
        let src = "prelude\n\n# One\nbody-one\n\n# Two\nbody-two\n";
        let doc = parse(src);
        assert_eq!(doc.sections.len(), 3);
        assert!(doc.sections[0].heading_index.is_none());
        assert_eq!(doc.sections[1].heading_index, Some(0));
        assert_eq!(doc.sections[2].heading_index, Some(1));
        let s1 = &src[doc.sections[1].byte_range.clone()];
        assert!(s1.contains("body-one"));
        assert!(!s1.contains("body-two"));
    }

    #[test]
    fn code_blocks_capture_language() {
        let src = "text\n\n```rust\nfn main() {}\n```\n\n```\nplain\n```\n";
        let doc = parse(src);
        assert_eq!(doc.code_blocks.len(), 2);
        assert_eq!(doc.code_blocks[0].lang.as_deref(), Some("rust"));
        assert!(doc.code_blocks[0].text.contains("fn main"));
        assert_eq!(doc.code_blocks[1].lang, None);
    }

    #[test]
    fn handles_crlf_line_endings() {
        let src = "---\r\ntitle: crlf\r\n---\r\n# Heading\r\n\r\n- item one\r\n- item two\r\n";
        let doc = parse(src);
        let fm = doc.frontmatter.expect("frontmatter");
        assert_eq!(fm.raw, "title: crlf");
        assert_eq!(doc.headings.len(), 1);
        assert_eq!(doc.headings[0].text, "Heading");
        assert_eq!(doc.list_items.len(), 2);
        assert_eq!(doc.list_items[0].text, "item one");
        assert_eq!(doc.list_items[1].text, "item two");
    }
}
