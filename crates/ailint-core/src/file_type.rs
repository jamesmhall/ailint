//! Auto-detection of AI agent guidance file types by path and filename.

use std::path::Path;

/// The kind of AI agent guidance file we recognize.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileType {
    /// Anthropic Claude Code: `CLAUDE.md` or files under `.claude/rules/`.
    ClaudeMd,
    /// Generic agent instructions: `AGENTS.md`.
    AgentsMd,
    /// VS Code Copilot customizations: `*.instructions.md`, `*.prompt.md`,
    /// `*.agent.md`, `SKILL.md`.
    CopilotCustomization,
    /// GitHub Copilot repo instructions: `.github/copilot-instructions.md`.
    CopilotInstructions,
    /// Cursor rules: `.cursorrules` or files under `.cursor/rules/`.
    CursorRules,
    /// Windsurf rules: `.windsurfrules`.
    WindsurfRules,
    /// Cline rules: `.clinerules`.
    ClineRules,
    /// JetBrains Junie project guidelines: `.junie/guidelines.md`.
    JunieGuidelines,
    /// Generic system prompt: `SYSTEM_PROMPT.md` / `system_prompt.{json,yaml}`,
    /// or any file under a configured `paths.prompt_dirs` directory (applied
    /// by [`crate::discovery::walk`], since no filename convention for
    /// system prompts is standard across agent frameworks).
    GenericSystemPrompt,
    /// aider conventions file: `CONVENTIONS.md` at any depth.
    AiderConventions,
    /// Continue.dev rules: `.continuerules` or files under `.continue/rules/`.
    ContinueRules,
    /// GitHub Skill entrypoint: `.github/skills/**/SKILL.md`.
    GitHubSkill,
    /// Custom project rule files: `PROJECT_RULES.md`, `AI_GUIDELINES.md`,
    /// `ai-rules.md` (matched case-insensitively).
    CustomProjectRules,
    /// Generic project Markdown documentation (e.g. `README.md`, `docs/**/*.md`).
    /// Subject only to structural, link, and discoverability rules — not AI
    /// semantic or security rules.
    GenericMarkdown,
    /// Generic project YAML file (e.g. `.github/workflows/*.yml`, config).
    /// Subject only to YAML syntax validation.
    GenericYaml,
    /// Something detected as guidance but not classified more specifically.
    Unknown,
}

impl FileType {
    /// Best-effort detection based on file path.
    pub fn detect(path: &Path) -> Option<Self> {
        let name = path.file_name()?.to_str()?;
        // Normalize separators so path-based matches work on Windows.
        let path_str = path.to_string_lossy().replace('\\', "/");
        let name_lower = name.to_ascii_lowercase();

        // Exact-name matches first.
        match name {
            "CLAUDE.md" => return Some(Self::ClaudeMd),
            "AGENTS.md" => return Some(Self::AgentsMd),
            "CONVENTIONS.md" => return Some(Self::AiderConventions),
            ".cursorrules" => return Some(Self::CursorRules),
            ".windsurfrules" => return Some(Self::WindsurfRules),
            ".clinerules" => return Some(Self::ClineRules),
            ".continuerules" => return Some(Self::ContinueRules),
            "copilot-instructions.md" => return Some(Self::CopilotInstructions),
            "SYSTEM_PROMPT.md" | "system_prompt.md" => {
                return Some(Self::GenericSystemPrompt);
            }
            "system_prompt.json" | "system_prompt.yaml" | "system_prompt.yml" => {
                return Some(Self::GenericSystemPrompt);
            }
            _ => {}
        }

        // Case-insensitive custom project rule filenames.
        if matches!(
            name_lower.as_str(),
            "project_rules.md" | "ai_guidelines.md" | "ai-rules.md"
        ) {
            return Some(Self::CustomProjectRules);
        }

        // Path-based matches. GitHub Skill wins over the generic
        // Copilot-customization `SKILL.md` suffix rule below.
        if (path_str.contains("/.github/skills/") || path_str.starts_with(".github/skills/"))
            && name == "SKILL.md"
        {
            return Some(Self::GitHubSkill);
        }
        if path_str.contains("/.claude/rules/") && name.ends_with(".md") {
            return Some(Self::ClaudeMd);
        }
        if path_str.contains("/.cursor/rules/") {
            return Some(Self::CursorRules);
        }
        if path_str.contains("/.continue/rules/") {
            return Some(Self::ContinueRules);
        }
        if path_str.contains("/.github/copilot-instructions.md") {
            return Some(Self::CopilotInstructions);
        }
        if path_str.contains("/.junie/guidelines.md") || path_str.ends_with(".junie/guidelines.md")
        {
            return Some(Self::JunieGuidelines);
        }

        // Suffix-based matches (VS Code Copilot customization files).
        if name.ends_with(".instructions.md")
            || name.ends_with(".prompt.md")
            || name.ends_with(".agent.md")
            || name == "SKILL.md"
        {
            return Some(Self::CopilotCustomization);
        }

        // Generic fallback: any `.md` / `.yaml` / `.yml` is picked up for
        // structural + link validation. AI semantic rules skip these via
        // `Rule::applies_to`.
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase());
        match ext.as_deref() {
            Some("md") | Some("markdown") => return Some(Self::GenericMarkdown),
            Some("yaml") | Some("yml") => return Some(Self::GenericYaml),
            _ => {}
        }

        None
    }

    /// True if this file is an AI agent guidance file (not generic project
    /// documentation or config). AI-specific rules use this to filter.
    pub fn is_ai_guidance(self) -> bool {
        !matches!(self, Self::GenericMarkdown | Self::GenericYaml)
    }

    /// True if this file is Markdown (either agent guidance or generic docs).
    pub fn is_markdown(self) -> bool {
        !matches!(self, Self::GenericYaml)
    }

    /// Stable kebab-case name, used in reports and config.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ClaudeMd => "claude-md",
            Self::AgentsMd => "agents-md",
            Self::CopilotCustomization => "copilot-customization",
            Self::CopilotInstructions => "copilot-instructions",
            Self::CursorRules => "cursor-rules",
            Self::WindsurfRules => "windsurf-rules",
            Self::ClineRules => "cline-rules",
            Self::JunieGuidelines => "junie-guidelines",
            Self::GenericSystemPrompt => "generic-system-prompt",
            Self::AiderConventions => "aider-conventions",
            Self::ContinueRules => "continue-rules",
            Self::GitHubSkill => "github-skill",
            Self::CustomProjectRules => "custom-project-rules",
            Self::GenericMarkdown => "generic-markdown",
            Self::GenericYaml => "generic-yaml",
            Self::Unknown => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_junie_guidelines() {
        assert_eq!(
            FileType::detect(Path::new(".junie/guidelines.md")),
            Some(FileType::JunieGuidelines)
        );
        assert_eq!(
            FileType::detect(Path::new("/repo/.junie/guidelines.md")),
            Some(FileType::JunieGuidelines)
        );
    }

    #[test]
    fn detects_top_level_files() {
        assert_eq!(
            FileType::detect(Path::new("CLAUDE.md")),
            Some(FileType::ClaudeMd)
        );
        assert_eq!(
            FileType::detect(Path::new("AGENTS.md")),
            Some(FileType::AgentsMd)
        );
        assert_eq!(
            FileType::detect(Path::new(".cursorrules")),
            Some(FileType::CursorRules)
        );
        assert_eq!(
            FileType::detect(Path::new(".windsurfrules")),
            Some(FileType::WindsurfRules)
        );
        assert_eq!(
            FileType::detect(Path::new(".clinerules")),
            Some(FileType::ClineRules)
        );
        assert_eq!(
            FileType::detect(Path::new("SYSTEM_PROMPT.md")),
            Some(FileType::GenericSystemPrompt)
        );
    }

    #[test]
    fn detects_path_based() {
        assert_eq!(
            FileType::detect(Path::new("/repo/.github/copilot-instructions.md")),
            Some(FileType::CopilotInstructions)
        );
        assert_eq!(
            FileType::detect(Path::new("/repo/.cursor/rules/rust.mdc")),
            Some(FileType::CursorRules)
        );
        assert_eq!(
            FileType::detect(Path::new("/repo/.claude/rules/style.md")),
            Some(FileType::ClaudeMd)
        );
    }

    #[test]
    fn detects_copilot_customization_suffixes() {
        assert_eq!(
            FileType::detect(Path::new(".github/instructions/rust.instructions.md")),
            Some(FileType::CopilotCustomization)
        );
        assert_eq!(
            FileType::detect(Path::new(".github/prompts/new-rule.prompt.md")),
            Some(FileType::CopilotCustomization)
        );
        assert_eq!(
            FileType::detect(Path::new(".github/agents/reviewer.agent.md")),
            Some(FileType::CopilotCustomization)
        );
        // Bare `SKILL.md` outside `.github/skills/` remains a Copilot skill.
        assert_eq!(
            FileType::detect(Path::new("some/dir/SKILL.md")),
            Some(FileType::CopilotCustomization)
        );
    }

    #[test]
    fn detects_github_skills_over_copilot_customization() {
        assert_eq!(
            FileType::detect(Path::new(".github/skills/add-rule/SKILL.md")),
            Some(FileType::GitHubSkill)
        );
        assert_eq!(
            FileType::detect(Path::new("/repo/.github/skills/example/nested/SKILL.md")),
            Some(FileType::GitHubSkill)
        );
    }

    // Backslashes are separators only on Windows; on Unix this path is a
    // single filename and the test would be meaningless.
    #[cfg(windows)]
    #[test]
    fn detects_github_skills_with_windows_separators() {
        assert_eq!(
            FileType::detect(Path::new(r"repo\.github\skills\example\SKILL.md")),
            Some(FileType::GitHubSkill)
        );
    }

    #[test]
    fn detects_aider_conventions() {
        assert_eq!(
            FileType::detect(Path::new("CONVENTIONS.md")),
            Some(FileType::AiderConventions)
        );
        assert_eq!(
            FileType::detect(Path::new("/repo/subdir/CONVENTIONS.md")),
            Some(FileType::AiderConventions)
        );
    }

    #[test]
    fn detects_continue_rules() {
        assert_eq!(
            FileType::detect(Path::new(".continuerules")),
            Some(FileType::ContinueRules)
        );
        assert_eq!(
            FileType::detect(Path::new("/repo/.continue/rules/python.md")),
            Some(FileType::ContinueRules)
        );
    }

    #[test]
    fn detects_custom_project_rules_case_insensitive() {
        assert_eq!(
            FileType::detect(Path::new("PROJECT_RULES.md")),
            Some(FileType::CustomProjectRules)
        );
        assert_eq!(
            FileType::detect(Path::new("docs/AI_GUIDELINES.md")),
            Some(FileType::CustomProjectRules)
        );
        assert_eq!(
            FileType::detect(Path::new("ai-rules.md")),
            Some(FileType::CustomProjectRules)
        );
        assert_eq!(
            FileType::detect(Path::new("Project_Rules.md")),
            Some(FileType::CustomProjectRules)
        );
    }

    #[test]
    fn generic_markdown_is_fallback() {
        assert_eq!(
            FileType::detect(Path::new("README.md")),
            Some(FileType::GenericMarkdown)
        );
        assert_eq!(
            FileType::detect(Path::new("docs/architecture/overview.md")),
            Some(FileType::GenericMarkdown)
        );
    }

    #[test]
    fn generic_yaml_is_fallback() {
        assert_eq!(
            FileType::detect(Path::new(".github/workflows/ci.yml")),
            Some(FileType::GenericYaml)
        );
        assert_eq!(
            FileType::detect(Path::new("config/settings.yaml")),
            Some(FileType::GenericYaml)
        );
    }

    #[test]
    fn is_ai_guidance_predicate() {
        assert!(FileType::ClaudeMd.is_ai_guidance());
        assert!(FileType::AgentsMd.is_ai_guidance());
        assert!(!FileType::GenericMarkdown.is_ai_guidance());
        assert!(!FileType::GenericYaml.is_ai_guidance());
    }
}
