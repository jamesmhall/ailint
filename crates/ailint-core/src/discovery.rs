//! Filesystem discovery of AI agent guidance files.

use std::path::Path;

use anyhow::Result;

use crate::config::Config;
use crate::file_type::FileType;
use crate::DiscoveredFile;

/// Walk `root` and return every recognized guidance file.
///
/// TODO: honor `config.paths.exclude`, respect .gitignore via the `ignore`
/// crate, follow-symlinks flag, max-depth limit, and content-sniffing fallback
/// for unclassified markdown.
pub fn walk(root: &Path, _config: &Config) -> Result<Vec<DiscoveredFile>> {
    let mut out = Vec::new();

    // TODO: switch to `ignore::WalkBuilder` so `.gitignore` is honored by
    // default. Using `walkdir` here as the simplest possible stub.
    for entry in walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if let Some(ft) = FileType::detect(entry.path()) {
            out.push(DiscoveredFile {
                path: entry.path().to_path_buf(),
                file_type: ft,
            });
        }
    }

    Ok(out)
}
