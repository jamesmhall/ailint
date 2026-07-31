//! AIL340 `orphaned-document` — a Markdown file exists in the corpus but is
//! not reachable via local links from a root document (typically
//! `README.md`).
//!
//! See: `docs/rules/consistency/AIL340.md`

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use crate::file_type::FileType;
use crate::parser::{DocumentContent, ParsedDocument};
use crate::rules::consistency::AIL340;
use crate::rules::{BatchRule, RuleContext, RuleId, Severity, Violation};

/// AIL340 orphaned-document: flags guidance files no other document references.
#[derive(Debug, Default)]
pub struct OrphanedDocumentRule;

impl BatchRule for OrphanedDocumentRule {
    fn id(&self) -> RuleId {
        AIL340
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn description(&self) -> &'static str {
        "Document is not reachable via local links from any README or AGENTS root."
    }

    fn fix_hint(&self) -> &'static str {
        "Link it from a README or AGENTS file, or delete it."
    }

    /// Applies to every Markdown document — this rule needs generic docs in
    /// scope to discover islands.
    fn applies_to(&self, file_type: FileType) -> bool {
        file_type.is_markdown()
    }

    fn run_batch(&self, docs: &[ParsedDocument], _ctx: &RuleContext<'_>) -> Vec<Violation> {
        // Canonicalize all doc paths for graph keys.
        let mut canonical: HashMap<PathBuf, usize> = HashMap::new();
        let mut paths: Vec<PathBuf> = Vec::with_capacity(docs.len());
        for (i, d) in docs.iter().enumerate() {
            let p = canonicalize(&d.path);
            canonical.insert(p.clone(), i);
            paths.push(p);
        }

        // Build adjacency list from resolved local links.
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); docs.len()];
        for (i, d) in docs.iter().enumerate() {
            let md = match &d.content {
                DocumentContent::Markdown(m) => m,
                _ => continue,
            };
            let doc_dir = d.path.parent().unwrap_or(Path::new(""));
            for link in &md.links {
                let Some(target) = resolve_link(doc_dir, &link.url) else {
                    continue;
                };
                if let Some(&j) = canonical.get(&target) {
                    adj[i].push(j);
                }
            }
        }

        // Roots for reachability:
        //   * every AI guidance file (each is an entry point for its tool)
        //   * the shallowest `README.md` and `AGENTS.md` files
        let roots = find_roots(docs, &paths);
        if roots.is_empty() {
            return Vec::new();
        }

        // BFS reachability.
        let mut reached: HashSet<usize> = HashSet::new();
        let mut queue: VecDeque<usize> = VecDeque::new();
        for &r in &roots {
            reached.insert(r);
            queue.push_back(r);
        }
        while let Some(i) = queue.pop_front() {
            for &j in &adj[i] {
                if reached.insert(j) {
                    queue.push_back(j);
                }
            }
        }

        // Anything unreached is an orphan.
        let mut out = Vec::new();
        let mut orphans: BTreeSet<usize> = BTreeSet::new();
        for i in 0..docs.len() {
            if !reached.contains(&i) {
                orphans.insert(i);
            }
        }
        for i in orphans {
            let doc = &docs[i];
            let v = Violation::new(
                AIL340,
                self.default_severity(),
                doc.path.clone(),
                "orphaned document",
            );
            out.push(v);
        }
        out
    }
}

fn canonicalize(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn resolve_link(doc_dir: &Path, url: &str) -> Option<PathBuf> {
    if url.is_empty() || url.starts_with('#') || url.starts_with("//") {
        return None;
    }
    if url.starts_with("mailto:") || url.starts_with("tel:") {
        return None;
    }
    if has_scheme(url) {
        return None;
    }
    let path_part = match url.find('#') {
        Some(i) => &url[..i],
        None => url,
    };
    if path_part.is_empty() {
        return None;
    }
    let joined = doc_dir.join(path_part.trim_start_matches('/'));
    Some(canonicalize(&joined))
}

fn has_scheme(url: &str) -> bool {
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

/// Roots for reachability: every AI guidance document (each is an entry
/// point for its tool) plus the shallowest `README.md` and `AGENTS.md`
/// files (either counts as a project entry point).
fn find_roots(docs: &[ParsedDocument], paths: &[PathBuf]) -> Vec<usize> {
    let mut roots: BTreeSet<usize> = BTreeSet::new();

    for (i, d) in docs.iter().enumerate() {
        if d.file_type.is_ai_guidance() {
            roots.insert(i);
        }
    }

    // README.md and AGENTS.md are treated the same: shallowest instance(s)
    // of each name count as roots. We compute the minimum depth per name
    // separately so a shallow README doesn't shadow a deeper AGENTS or vice
    // versa.
    for target in ["README.md", "AGENTS.md"] {
        let mut candidates: Vec<(usize, usize)> = Vec::new();
        for (i, p) in paths.iter().enumerate() {
            if p.file_name().and_then(|n| n.to_str()) == Some(target) {
                candidates.push((i, p.components().count()));
            }
        }
        if let Some(min_depth) = candidates.iter().map(|(_, d)| *d).min() {
            for (i, d) in candidates {
                if d == min_depth {
                    roots.insert(i);
                }
            }
        }
    }

    roots.into_iter().collect()
}
