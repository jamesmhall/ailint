//! AIL040 `broken-local-link` — a relative link in a Markdown document
//! points at a file that does not exist on disk.
//!
//! See: `docs/rules/structural/AIL040.md`

use std::path::{Path, PathBuf};

use crate::file_type::FileType;
use crate::parser::{DocumentContent, ParsedDocument};
use crate::rules::structural::AIL040;
use crate::rules::{Rule, RuleContext, RuleId, Severity, Violation};

/// AIL040 broken-local-link: relative link target does not exist on disk.
#[derive(Debug, Default)]
pub struct BrokenLocalLinkRule;

impl Rule for BrokenLocalLinkRule {
    fn id(&self) -> RuleId {
        AIL040
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    /// Applies to every Markdown document — guidance and generic project docs
    /// alike — since broken links are a universal quality concern.
    fn applies_to(&self, file_type: FileType) -> bool {
        file_type.is_markdown()
    }

    fn run(&self, doc: &ParsedDocument, _ctx: &RuleContext<'_>) -> Vec<Violation> {
        let md = match &doc.content {
            DocumentContent::Markdown(m) => m,
            _ => return Vec::new(),
        };

        let doc_dir = doc.path.parent().unwrap_or(Path::new(""));
        let mut out = Vec::new();

        for link in &md.links {
            if !is_local_link(&link.url) {
                continue;
            }
            let (path_part, _fragment) = split_fragment(&link.url);
            if path_part.is_empty() {
                // pure `#anchor` — skip, in-doc anchors aren't checked yet.
                continue;
            }
            let target = resolve_target(doc_dir, path_part);
            if target.exists() {
                continue;
            }
            let mut v = Violation::new(
                AIL040,
                self.default_severity(),
                doc.path.clone(),
                format!("broken local link: '{}' does not exist", link.url),
            )
            .at(link.line, 1);
            v.fix_hint = Some(format!(
                "fix or remove the link '{}' (target `{}` not found)",
                link.text,
                target.display()
            ));
            out.push(v);
        }

        out
    }
}

/// A link is "local" if it lacks a URL scheme and isn't a pure anchor or
/// mailto/tel/etc.
fn is_local_link(url: &str) -> bool {
    if url.is_empty() {
        return false;
    }
    if url.starts_with('#') {
        return false;
    }
    if url.starts_with("mailto:") || url.starts_with("tel:") {
        return false;
    }
    // http://, https://, ftp://, file://, javascript:, data:, etc.
    if has_scheme(url) {
        return false;
    }
    // Protocol-relative URLs.
    if url.starts_with("//") {
        return false;
    }
    true
}

fn has_scheme(url: &str) -> bool {
    // A scheme is [A-Za-z][A-Za-z0-9+\-.]*:
    let bytes = url.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_alphabetic() {
        return false;
    }
    for (i, &b) in bytes.iter().enumerate().skip(1) {
        if b == b':' {
            return i > 0;
        }
        if !(b.is_ascii_alphanumeric() || b == b'+' || b == b'-' || b == b'.') {
            return false;
        }
    }
    false
}

fn split_fragment(url: &str) -> (&str, Option<&str>) {
    match url.find('#') {
        Some(i) => (&url[..i], Some(&url[i + 1..])),
        None => (url, None),
    }
}

fn resolve_target(doc_dir: &Path, rel: &str) -> PathBuf {
    // Absolute paths (starting with `/`) are treated as workspace-relative:
    // resolve from the deepest existing ancestor. For simplicity, we resolve
    // from `doc_dir` for absolute paths too — this may miss some cases but
    // avoids false positives from workspace-root assumptions.
    let trimmed = rel.trim_start_matches('/');
    doc_dir.join(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_scheme() {
        assert!(has_scheme("http://example.com"));
        assert!(has_scheme("https://example.com"));
        assert!(has_scheme("mailto:a@b"));
        assert!(has_scheme("ftp://x"));
        assert!(!has_scheme("relative/path.md"));
        assert!(!has_scheme("./relative.md"));
        assert!(!has_scheme("#anchor"));
    }

    #[test]
    fn classifies_local_links() {
        assert!(is_local_link("README.md"));
        assert!(is_local_link("./sub/doc.md"));
        assert!(is_local_link("../up.md"));
        assert!(is_local_link("/absolute/rel.md"));
        assert!(!is_local_link("https://example.com"));
        assert!(!is_local_link("mailto:a@b"));
        assert!(!is_local_link("#anchor"));
        assert!(!is_local_link(""));
        assert!(!is_local_link("//cdn.example.com/a.js"));
    }

    #[test]
    fn splits_fragment() {
        assert_eq!(split_fragment("a.md#sec"), ("a.md", Some("sec")));
        assert_eq!(split_fragment("a.md"), ("a.md", None));
    }
}
