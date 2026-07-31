//! AIL301 `no-duplicate-guidance-files` — detect substantively identical files.
//!
//! See: `docs/rules/consistency/AIL301.md`

use std::collections::HashMap;

use serde::Deserialize;

use crate::parser::{DocumentContent, ParsedDocument};
use crate::rules::consistency::AIL301;
use crate::rules::{BatchRule, RuleContext, RuleId, Severity, Violation};

const DEFAULT_MIN_LEN: usize = 100;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Options {
    min_fingerprint_length: Option<usize>,
}

/// AIL301 no-duplicate-guidance-files: flags files duplicating the same content.
#[derive(Debug, Default)]
pub struct NoDuplicateGuidanceFilesRule;

impl BatchRule for NoDuplicateGuidanceFilesRule {
    fn id(&self) -> RuleId {
        AIL301
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &'static str {
        "File content is byte-identical to another guidance file."
    }

    fn fix_hint(&self) -> &'static str {
        "Keep one file as the source of truth; delete or symlink the other."
    }

    fn run_batch(&self, docs: &[ParsedDocument], ctx: &RuleContext<'_>) -> Vec<Violation> {
        let opts: Options = ctx
            .options
            .and_then(|v| serde_yaml::from_value(v.clone()).ok())
            .unwrap_or_default();
        let min_len = opts.min_fingerprint_length.unwrap_or(DEFAULT_MIN_LEN);

        let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
        for (idx, doc) in docs.iter().enumerate() {
            let fp = fingerprint(doc);
            if fp.len() < min_len {
                continue;
            }
            groups.entry(fp).or_default().push(idx);
        }

        let mut out = Vec::new();
        for indices in groups.values() {
            if indices.len() < 2 {
                continue;
            }
            let first = &docs[indices[0]];
            let first_name = first
                .path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("<unknown>");
            for &idx in indices.iter().skip(1) {
                let doc = &docs[idx];
                let v = Violation::new(
                    AIL301,
                    self.default_severity(),
                    doc.path.clone(),
                    "duplicate content",
                )
                .with_detail(format!("matches {first_name}"));
                out.push(v);
            }
        }
        out
    }
}

// Strip frontmatter (if any), lowercase, strip common markdown syntax noise,
// collapse whitespace, trim.
fn fingerprint(doc: &ParsedDocument) -> String {
    let body: &str = match &doc.content {
        DocumentContent::Markdown(md) => match &md.frontmatter {
            Some(fm) => &doc.raw[fm.byte_range.end..],
            None => &doc.raw,
        },
        _ => &doc.raw,
    };

    let mut buf = String::with_capacity(body.len());
    for line in body.lines() {
        let mut stripped = line.trim_start();
        // Drop heading markers.
        while let Some(rest) = stripped.strip_prefix('#') {
            stripped = rest;
        }
        stripped = stripped.trim_start();
        // Drop leading list markers.
        for marker in ["- ", "* ", "+ "] {
            if let Some(rest) = stripped.strip_prefix(marker) {
                stripped = rest;
                break;
            }
        }
        for ch in stripped.chars() {
            if ch == '`' {
                continue;
            }
            for lc in ch.to_lowercase() {
                buf.push(lc);
            }
        }
        buf.push(' ');
    }

    let mut out = String::with_capacity(buf.len());
    let mut prev_space = true;
    for ch in buf.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(ch);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::markdown::MarkdownDoc;
    use std::path::PathBuf;

    fn md_doc(raw: &str) -> ParsedDocument {
        ParsedDocument {
            path: PathBuf::from("x.md"),
            file_type: crate::file_type::FileType::AgentsMd,
            raw: raw.to_string(),
            content: DocumentContent::Markdown(MarkdownDoc::default()),
        }
    }

    #[test]
    fn fingerprint_ignores_markdown_syntax() {
        let a = md_doc("# Heading\n\n- one\n- two\n");
        let b = md_doc("## HEADING\n\n* one\n* two\n");
        assert_eq!(fingerprint(&a), fingerprint(&b));
    }
}
