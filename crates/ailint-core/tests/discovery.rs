use std::path::{Path, PathBuf};

use ailint_core::discovery;
use ailint_core::file_type::FileType;
use ailint_core::Config;

fn fixture_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("tests/fixtures/discovery/nested");
    p
}

fn paths(files: &[ailint_core::DiscoveredFile]) -> Vec<String> {
    files
        .iter()
        .map(|f| f.path.to_string_lossy().into_owned())
        .collect()
}

fn ends_with(haystack: &[String], needle: &str) -> bool {
    haystack
        .iter()
        .any(|p| p.replace('\\', "/").ends_with(needle))
}

#[test]
fn discovers_dotfile_paths() {
    let root = fixture_root();
    let cfg = Config::default();
    let files = discovery::walk(&root, &cfg).expect("walk");
    let ps = paths(&files);
    assert!(
        ends_with(&ps, ".github/copilot-instructions.md"),
        "expected copilot-instructions.md, got {ps:?}"
    );
}

#[test]
fn discovers_nested_agents_files() {
    let root = fixture_root();
    let cfg = Config::default();
    let files = discovery::walk(&root, &cfg).expect("walk");
    let ps = paths(&files);

    assert!(ends_with(&ps, "nested/AGENTS.md"), "AGENTS.md: {ps:?}");
    assert!(
        ends_with(&ps, "nested/subdir/CLAUDE.md"),
        "CLAUDE.md: {ps:?}"
    );
    assert!(
        ends_with(&ps, "nested/subdir/CONVENTIONS.md"),
        "CONVENTIONS.md: {ps:?}"
    );
    assert!(
        ends_with(&ps, ".github/skills/example/SKILL.md"),
        "SKILL.md: {ps:?}"
    );
    assert!(
        ends_with(&ps, ".github/copilot-instructions.md"),
        "copilot-instructions.md: {ps:?}"
    );

    let skill = files
        .iter()
        .find(|f| f.path.file_name().and_then(|s| s.to_str()) == Some("SKILL.md"))
        .expect("SKILL.md discovered");
    assert_eq!(skill.file_type, FileType::GitHubSkill);

    let conv = files
        .iter()
        .find(|f| f.path.file_name().and_then(|s| s.to_str()) == Some("CONVENTIONS.md"))
        .expect("CONVENTIONS.md discovered");
    assert_eq!(conv.file_type, FileType::AiderConventions);
}

#[test]
fn respects_gitignore_by_default() {
    let root = fixture_root();
    let mut cfg = Config::default();
    cfg.paths.exclude.clear();
    assert!(cfg.paths.respect_gitignore);
    let files = discovery::walk(&root, &cfg).expect("walk");
    let ps = paths(&files);

    assert!(
        !ends_with(&ps, "nested/secret/CLAUDE.md"),
        "gitignored file leaked: {ps:?}"
    );

    cfg.paths.respect_gitignore = false;
    let files = discovery::walk(&root, &cfg).expect("walk");
    let ps = paths(&files);
    assert!(
        ends_with(&ps, "nested/secret/CLAUDE.md"),
        "gitignore off should surface secret/CLAUDE.md, got {ps:?}"
    );
}

#[test]
fn honors_config_exclude_globs() {
    let root = fixture_root();
    let mut cfg = Config::default();
    cfg.paths.respect_gitignore = false;
    cfg.paths.exclude = vec!["**/node_modules/**".to_string()];
    let files = discovery::walk(&root, &cfg).expect("walk");
    let ps = paths(&files);

    assert!(
        !ps.iter().any(|p| p.contains("/node_modules/")),
        "node_modules leaked: {ps:?}"
    );
    assert!(
        ends_with(&ps, "nested/AGENTS.md"),
        "AGENTS.md still present: {ps:?}"
    );
}

#[test]
fn reclassifies_files_under_configured_prompt_dirs() {
    let root = fixture_root();
    let cfg = Config::default();
    assert_eq!(cfg.paths.prompt_dirs, vec!["prompts/**".to_string()]);
    let files = discovery::walk(&root, &cfg).expect("walk");

    let onboarding = files
        .iter()
        .find(|f| f.path.file_name().and_then(|s| s.to_str()) == Some("onboarding.md"))
        .expect("onboarding.md discovered");
    assert_eq!(onboarding.file_type, FileType::GenericSystemPrompt);

    let base = files
        .iter()
        .find(|f| f.path.file_name().and_then(|s| s.to_str()) == Some("base.txt"))
        .expect("base.txt discovered");
    assert_eq!(base.file_type, FileType::GenericSystemPrompt);
}

#[test]
fn prompt_dir_reclassification_is_opt_in() {
    let root = fixture_root();
    let mut cfg = Config::default();
    cfg.paths.prompt_dirs = Vec::new();
    let files = discovery::walk(&root, &cfg).expect("walk");

    // With no configured prompt dirs, generic Markdown stays generic and the
    // extensionless-convention .txt file is not discovered at all.
    let onboarding = files
        .iter()
        .find(|f| f.path.file_name().and_then(|s| s.to_str()) == Some("onboarding.md"))
        .expect("onboarding.md discovered");
    assert_eq!(onboarding.file_type, FileType::GenericMarkdown);
    assert!(
        !files
            .iter()
            .any(|f| f.path.file_name().and_then(|s| s.to_str()) == Some("base.txt")),
        "base.txt should not be discovered without a matching prompt dir"
    );
}

#[test]
fn walks_relative_path_root() {
    let cfg = Config::default();
    let files = discovery::walk(Path::new("."), &cfg).expect("walk");
    assert!(!files.is_empty(), "expected non-empty discovery from '.'");
}
