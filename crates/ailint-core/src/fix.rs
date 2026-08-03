//! Deterministic auto-fix pipeline for [`Violation::fixes`].
//!
//! Groups edits by file, sorts descending by byte-offset, refuses to touch
//! any file with overlapping edits, and rewrites the survivors on disk.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::rules::{TextEdit, Violation};

/// Outcome of applying `--fix` to one file.
#[derive(Debug, Clone)]
pub struct FileFixResult {
    /// File that was considered.
    pub path: PathBuf,
    /// Number of edits applied. Zero when the file was skipped.
    pub applied: usize,
    /// Present when the file was skipped because two edits overlapped.
    pub conflict: Option<FixConflict>,
}

/// A pair of edits from the same file that overlap in byte range.
#[derive(Debug, Clone)]
pub struct FixConflict {
    /// Byte range of the first (higher-offset) edit.
    pub first: std::ops::Range<usize>,
    /// Byte range of the second (lower-offset) edit that overlaps it.
    pub second: std::ops::Range<usize>,
}

/// Apply every `TextEdit` on `violations` to disk, one file at a time.
///
/// Edits within a file are applied in reverse byte-order so earlier offsets
/// remain valid. Any file with overlapping edits is skipped whole and its
/// result carries `Some(FixConflict)` so the caller can warn or error.
pub fn apply_all(violations: &[Violation]) -> Result<Vec<FileFixResult>> {
    let mut by_file: BTreeMap<PathBuf, Vec<TextEdit>> = BTreeMap::new();
    for v in violations {
        if v.fixes.is_empty() {
            continue;
        }
        by_file
            .entry(v.file.clone())
            .or_default()
            .extend(v.fixes.iter().cloned());
    }

    let mut results = Vec::with_capacity(by_file.len());
    for (path, edits) in by_file {
        results.push(apply_to_file(&path, edits)?);
    }
    Ok(results)
}

fn apply_to_file(path: &Path, mut edits: Vec<TextEdit>) -> Result<FileFixResult> {
    // Descending by start so earlier offsets are not invalidated as we splice.
    edits.sort_by_key(|e| std::cmp::Reverse(e.range.start));

    // Overlap check: adjacent items in the sorted list cannot share bytes.
    for pair in edits.windows(2) {
        let higher = &pair[0];
        let lower = &pair[1];
        if lower.range.end > higher.range.start {
            return Ok(FileFixResult {
                path: path.to_path_buf(),
                applied: 0,
                conflict: Some(FixConflict {
                    first: higher.range.clone(),
                    second: lower.range.clone(),
                }),
            });
        }
    }

    let mut raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read {} for fixing", path.display()))?;
    let applied = edits.len();
    for edit in edits {
        if edit.range.end > raw.len() {
            anyhow::bail!(
                "fix range {:?} out of bounds ({} bytes) in {}",
                edit.range,
                raw.len(),
                path.display()
            );
        }
        raw.replace_range(edit.range, &edit.replacement);
    }
    fs::write(path, raw)
        .with_context(|| format!("failed to write fixed content to {}", path.display()))?;

    Ok(FileFixResult {
        path: path.to_path_buf(),
        applied,
        conflict: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{RuleId, Severity};

    fn tmp_file(name: &str, body: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("ailint-fix-{}-{name}", std::process::id()));
        std::fs::write(&p, body).unwrap();
        p
    }

    fn violation_with(path: PathBuf, edits: Vec<TextEdit>) -> Violation {
        let mut v = Violation::new(RuleId::new(999, "test"), Severity::Warning, path, "test");
        v.fixes = edits;
        v
    }

    #[test]
    fn applies_single_edit() {
        let path = tmp_file("single.md", "hello world");
        let v = violation_with(
            path.clone(),
            vec![TextEdit {
                range: 6..11,
                replacement: "there".into(),
            }],
        );
        let results = apply_all(&[v]).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].applied, 1);
        assert!(results[0].conflict.is_none());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello there");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn applies_two_non_overlapping_in_reverse_order() {
        let path = tmp_file("two.md", "AAA BBB CCC");
        // If applied in forward order the second range would be shifted.
        // Reverse order proves the sort keeps offsets valid.
        let v = violation_with(
            path.clone(),
            vec![
                TextEdit {
                    range: 0..3,
                    replacement: "aaaa".into(),
                },
                TextEdit {
                    range: 8..11,
                    replacement: "cc".into(),
                },
            ],
        );
        let results = apply_all(&[v]).unwrap();
        assert_eq!(results[0].applied, 2);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "aaaa BBB cc");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn refuses_overlapping_edits() {
        let path = tmp_file("overlap.md", "hello world");
        let v = violation_with(
            path.clone(),
            vec![
                TextEdit {
                    range: 0..5,
                    replacement: "HI".into(),
                },
                TextEdit {
                    range: 3..8,
                    replacement: "XX".into(),
                },
            ],
        );
        let results = apply_all(&[v]).unwrap();
        assert_eq!(results[0].applied, 0);
        assert!(results[0].conflict.is_some());
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "hello world",
            "content must be untouched when edits conflict"
        );
        let _ = std::fs::remove_file(&path);
    }
}
