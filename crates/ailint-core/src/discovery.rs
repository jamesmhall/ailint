//! Filesystem discovery of AI agent guidance files.

use std::path::Path;

use anyhow::Result;
use ignore::overrides::OverrideBuilder;
use ignore::WalkBuilder;

use crate::config::Config;
use crate::file_type::FileType;
use crate::DiscoveredFile;

/// Walk `root` and return every recognized guidance file.
///
/// Honors `.gitignore` when `config.paths.respect_gitignore` is true,
/// applies `paths.include`/`paths.exclude` globs, and follows symlinks
/// when `paths.follow_symlinks` is set. Files under a `paths.prompt_dirs`
/// directory are reclassified as `FileType::GenericSystemPrompt` (see
/// `reclassify_prompt_dir`).
pub fn walk(root: &Path, config: &Config) -> Result<Vec<DiscoveredFile>> {
    let overrides = build_overrides(root, config)?;
    let prompt_overrides = build_prompt_overrides(root, config)?;

    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(false)
        .git_ignore(config.paths.respect_gitignore)
        .git_global(config.paths.respect_gitignore)
        .git_exclude(config.paths.respect_gitignore)
        .require_git(false)
        .follow_links(config.paths.follow_symlinks)
        .overrides(overrides);

    let mut out = Vec::new();
    for result in builder.build() {
        let entry = match result {
            Ok(e) => e,
            Err(err) => {
                tracing::warn!(error = %err, "discovery walk error");
                continue;
            }
        };
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }
        let in_prompt_dir = prompt_overrides.matched(entry.path(), false).is_whitelist();
        if let Some(file_type) =
            reclassify_prompt_dir(FileType::detect(entry.path()), entry.path(), in_prompt_dir)
        {
            out.push(DiscoveredFile {
                path: entry.path().to_path_buf(),
                file_type,
            });
        }
    }

    Ok(out)
}

fn build_overrides(root: &Path, config: &Config) -> Result<ignore::overrides::Override> {
    let mut builder = OverrideBuilder::new(root);
    for pattern in &config.paths.include {
        builder.add(pattern)?;
    }
    for pattern in &config.paths.exclude {
        builder.add(&format!("!{pattern}"))?;
    }
    Ok(builder.build()?)
}

fn build_prompt_overrides(root: &Path, config: &Config) -> Result<ignore::overrides::Override> {
    let mut builder = OverrideBuilder::new(root);
    for pattern in &config.paths.prompt_dirs {
        builder.add(pattern)?;
    }
    Ok(builder.build()?)
}

/// Files that would otherwise fall back to a generic doc type (or go
/// undetected entirely) are treated as system prompts when they live under a
/// configured `paths.prompt_dirs` directory. Files matching a real tool
/// convention (`FileType::detect` returning anything more specific) are left
/// alone — the directory heuristic never overrides a known convention.
fn reclassify_prompt_dir(
    detected: Option<FileType>,
    path: &Path,
    in_prompt_dir: bool,
) -> Option<FileType> {
    if !in_prompt_dir {
        return detected;
    }
    match detected {
        Some(FileType::GenericMarkdown) | Some(FileType::GenericYaml) => {
            Some(FileType::GenericSystemPrompt)
        }
        Some(other) => Some(other),
        None if is_prompt_like_extension(path) => Some(FileType::GenericSystemPrompt),
        None => None,
    }
}

fn is_prompt_like_extension(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|s| s.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("md") | Some("markdown") | Some("txt") | Some("json") | Some("yaml") | Some("yml")
    )
}
