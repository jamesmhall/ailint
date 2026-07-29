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
    /// Generic system prompt: `SYSTEM_PROMPT.md` / `system_prompt.{json,yaml}`.
    GenericSystemPrompt,
    /// Something detected as guidance but not classified more specifically.
    Unknown,
}

impl FileType {
    /// Best-effort detection based on file path.
    ///
    /// TODO: expand patterns (e.g. Continue.dev, aider, GitHub Skills, custom
    /// project rules). Consider content-sniffing for `.md` files that look
    /// like agent guidance but aren't at a known path.
    pub fn detect(path: &Path) -> Option<Self> {
        let name = path.file_name()?.to_str()?;
        let path_str = path.to_string_lossy();

        // Exact-name matches first.
        match name {
            "CLAUDE.md" => return Some(Self::ClaudeMd),
            "AGENTS.md" => return Some(Self::AgentsMd),
            ".cursorrules" => return Some(Self::CursorRules),
            ".windsurfrules" => return Some(Self::WindsurfRules),
            ".clinerules" => return Some(Self::ClineRules),
            "copilot-instructions.md" => return Some(Self::CopilotInstructions),
            "SYSTEM_PROMPT.md" | "system_prompt.md" => {
                return Some(Self::GenericSystemPrompt);
            }
            "system_prompt.json" | "system_prompt.yaml" | "system_prompt.yml" => {
                return Some(Self::GenericSystemPrompt);
            }
            _ => {}
        }

        // Path-based matches.
        if path_str.contains("/.claude/rules/") && name.ends_with(".md") {
            return Some(Self::ClaudeMd);
        }
        if path_str.contains("/.cursor/rules/") {
            return Some(Self::CursorRules);
        }
        if path_str.contains("/.github/copilot-instructions.md") {
            return Some(Self::CopilotInstructions);
        }

        // Suffix-based matches (VS Code Copilot customization files).
        if name.ends_with(".instructions.md")
            || name.ends_with(".prompt.md")
            || name.ends_with(".agent.md")
            || name == "SKILL.md"
        {
            return Some(Self::CopilotCustomization);
        }

        None
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ClaudeMd => "claude-md",
            Self::AgentsMd => "agents-md",
            Self::CopilotCustomization => "copilot-customization",
            Self::CopilotInstructions => "copilot-instructions",
            Self::CursorRules => "cursor-rules",
            Self::WindsurfRules => "windsurf-rules",
            Self::ClineRules => "cline-rules",
            Self::GenericSystemPrompt => "generic-system-prompt",
            Self::Unknown => "unknown",
        }
    }
}
