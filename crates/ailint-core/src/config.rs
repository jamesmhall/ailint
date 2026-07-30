//! Configuration loading for ailint (`.ailint.yaml`).
//!
//! The schema is exported as JSON Schema via [`Config::json_schema`]
//! (`ailint schema` on the CLI). `AILINT_CONFIG` overrides discovery.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::reporter::ReporterKind;
use crate::rules::Severity;

/// Root configuration object, mirrored from `.ailint.yaml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Rule enablement, severity, and options.
    pub rules: RulesConfig,
    /// Discovery include/exclude behavior.
    pub paths: PathsConfig,
    /// Provider settings for opt-in LLM rules.
    pub llm: Option<LlmConfig>,
    /// Reporter format and output destination.
    pub output: OutputConfig,
}

/// The `rules:` block of `.ailint.yaml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct RulesConfig {
    /// Rule IDs or slugs to disable (e.g. `AIL100` or `no-vague-instruction`).
    pub disabled: Vec<String>,
    /// Overrides for rule severity, keyed by ID or slug.
    #[serde(default)]
    pub severity_overrides: std::collections::BTreeMap<String, Severity>,
    /// Per-rule options, keyed by ID or slug.
    #[serde(default)]
    #[schemars(with = "std::collections::BTreeMap<String, serde_json::Value>")]
    pub options: std::collections::BTreeMap<String, serde_yaml::Value>,
}

/// The `paths:` block of `.ailint.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct PathsConfig {
    /// Additional path globs to exclude from discovery.
    pub exclude: Vec<String>,
    /// If non-empty, only paths matching one of these globs are linted.
    #[serde(default)]
    pub include: Vec<String>,
    /// Follow symbolic links during discovery.
    #[serde(default)]
    pub follow_symlinks: bool,
    /// Honor `.gitignore` and other VCS ignore files during discovery.
    #[serde(default = "default_true")]
    pub respect_gitignore: bool,
    /// Gitignore-style globs (relative to the repo root) marking prompt
    /// directories. Files under a matching directory that would otherwise be
    /// classified as generic Markdown/YAML docs are treated as
    /// `FileType::GenericSystemPrompt` instead, since no single filename
    /// convention for system prompts exists across agent frameworks.
    #[serde(default = "default_prompt_dirs")]
    pub prompt_dirs: Vec<String>,
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            exclude: vec![
                "node_modules".into(),
                ".git".into(),
                "target".into(),
                "dist".into(),
            ],
            include: Vec::new(),
            follow_symlinks: false,
            respect_gitignore: true,
            prompt_dirs: default_prompt_dirs(),
        }
    }
}

fn default_prompt_dirs() -> Vec<String> {
    vec!["prompts/**".into()]
}

fn default_true() -> bool {
    true
}

/// The `llm:` block of `.ailint.yaml`; enables the opt-in AIL9xx rules.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LlmConfig {
    /// Which provider client to use.
    pub provider: LlmProviderKind,
    /// Model identifier passed to the provider.
    pub model: String,
    /// Override endpoint URL for OpenAI-compatible providers.
    #[serde(default)]
    pub base_url: Option<String>,
    /// Per-request timeout in seconds.
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
    /// Cap on tokens the provider may generate per request.
    #[serde(default)]
    pub max_tokens: Option<u32>,
    /// Sampling temperature forwarded to the provider.
    #[serde(default)]
    pub temperature: Option<f32>,
    /// Stop running LLM rules once cumulative spend exceeds this many USD.
    #[serde(default)]
    pub cost_cap_usd: Option<f64>,
}

/// Supported LLM provider clients.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LlmProviderKind {
    /// OpenAI's hosted API.
    Openai,
    /// Anthropic's hosted API.
    Anthropic,
    /// Google's Gemini API.
    Google,
    /// A local Ollama server.
    Ollama,
    /// Any OpenAI-compatible endpoint.
    Compatible,
}

/// The `output:` block of `.ailint.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct OutputConfig {
    /// Reporter used for violations.
    pub format: ReporterKind,
    /// When to colorize terminal output.
    pub color: ColorMode,
    /// Write reporter output to this file instead of stdout.
    #[serde(default)]
    pub output_file: Option<std::path::PathBuf>,
    /// Suppress non-violation output (banners, progress, summary lines).
    #[serde(default)]
    pub quiet: bool,
    /// Verbosity of the trailing summary block.
    #[serde(default)]
    pub summary_format: SummaryFormat,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: ReporterKind::Terminal,
            color: ColorMode::Auto,
            output_file: None,
            quiet: false,
            summary_format: SummaryFormat::default(),
        }
    }
}

/// Verbosity of the summary block after a `check` run.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum SummaryFormat {
    /// Per-severity counts plus file totals.
    #[default]
    Full,
    /// A single totals line.
    Compact,
    /// No summary block.
    None,
}

/// When to colorize terminal output.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    /// Colorize only when stdout is a TTY.
    #[default]
    Auto,
    /// Always emit color codes.
    Always,
    /// Never emit color codes.
    Never,
}

impl Config {
    /// JSON Schema for the config file, as pretty-printed JSON.
    pub fn json_schema() -> Result<String> {
        let schema = schemars::schema_for!(Config);
        serde_json::to_string_pretty(&schema).context("failed to serialize config schema")
    }

    /// Load a config file from disk. Returns `Config::default()` if the file
    /// does not exist. Dispatches on file extension: `.yaml`/`.yml` parse as
    /// YAML, `.json` as JSON, anything else tries YAML then falls back to JSON.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(str::to_ascii_lowercase);
        let cfg: Self = match ext.as_deref() {
            Some("yaml") | Some("yml") => serde_yaml::from_str(&raw)
                .with_context(|| format!("failed to parse {}", path.display()))?,
            Some("json") => serde_json::from_str(&raw)
                .with_context(|| format!("failed to parse {}", path.display()))?,
            _ => match serde_yaml::from_str(&raw) {
                Ok(cfg) => cfg,
                Err(yaml_err) => serde_json::from_str(&raw).with_context(|| {
                    format!(
                        "failed to parse {} as YAML ({yaml_err}) or JSON",
                        path.display()
                    )
                })?,
            },
        };
        Ok(cfg)
    }

    /// Locate the closest config file, walking up from `start`.
    pub fn discover(start: &Path) -> Option<std::path::PathBuf> {
        let abs = start.canonicalize().ok()?;
        let start_dir: &Path = if abs.is_file() { abs.parent()? } else { &abs };
        const NAMES: &[&str] = &[".ailint.yaml", ".ailint.yml", ".ailint.json"];
        for dir in start_dir.ancestors() {
            for name in NAMES {
                let candidate = dir.join(name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_path(name: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        let pid = std::process::id();
        p.push(format!("ailint-cfg-{pid}-{name}"));
        p
    }

    #[test]
    fn load_yaml_config_parses_defaults() {
        let path = tmp_path("defaults.yaml");
        std::fs::write(&path, "rules:\n  disabled: []\n").expect("write");
        let cfg = Config::load(&path).expect("load");
        assert!(cfg.rules.disabled.is_empty());
        assert!(cfg.paths.respect_gitignore);
        assert!(!cfg.paths.follow_symlinks);
        assert!(cfg.paths.include.is_empty());
        assert_eq!(cfg.paths.prompt_dirs, vec!["prompts/**".to_string()]);
        assert_eq!(cfg.output.summary_format, SummaryFormat::Full);
        assert!(!cfg.output.quiet);
        assert!(cfg.output.output_file.is_none());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_json_config_parses_defaults() {
        let path = tmp_path("defaults.json");
        std::fs::write(&path, r#"{"rules": {"disabled": []}}"#).expect("write");
        let cfg = Config::load(&path).expect("load");
        assert!(cfg.rules.disabled.is_empty());
        assert!(cfg.paths.respect_gitignore);
        assert_eq!(cfg.output.summary_format, SummaryFormat::Full);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_yaml_with_all_fields() {
        let path = tmp_path("full.yaml");
        let body = r#"
rules:
  disabled: [AIL100]
  severity_overrides:
    AIL101: warning
  options:
    AIL100:
      phrases: ["maybe"]
paths:
  exclude: [node_modules]
  include: ["docs/**/*.md"]
  follow_symlinks: true
  respect_gitignore: false
  prompt_dirs: ["prompts/**", "assistants/*/prompts/**"]
llm:
  provider: openai
  model: gpt-4o
  base_url: https://example.com/v1
  timeout_seconds: 30
  max_tokens: 1024
  temperature: 0.2
  cost_cap_usd: 5.0
output:
  format: json
  color: never
  output_file: /tmp/out.json
  quiet: true
  summary_format: compact
"#;
        std::fs::write(&path, body).expect("write");
        let cfg = Config::load(&path).expect("load");
        assert_eq!(cfg.rules.disabled, vec!["AIL100".to_string()]);
        assert_eq!(cfg.paths.include, vec!["docs/**/*.md".to_string()]);
        assert!(cfg.paths.follow_symlinks);
        assert!(!cfg.paths.respect_gitignore);
        assert_eq!(
            cfg.paths.prompt_dirs,
            vec![
                "prompts/**".to_string(),
                "assistants/*/prompts/**".to_string()
            ]
        );
        let llm = cfg.llm.expect("llm block");
        assert_eq!(llm.base_url.as_deref(), Some("https://example.com/v1"));
        assert_eq!(llm.timeout_seconds, Some(30));
        assert_eq!(llm.max_tokens, Some(1024));
        assert_eq!(llm.temperature, Some(0.2));
        assert_eq!(llm.cost_cap_usd, Some(5.0));
        assert_eq!(cfg.output.summary_format, SummaryFormat::Compact);
        assert!(cfg.output.quiet);
        assert_eq!(
            cfg.output.output_file.as_deref(),
            Some(std::path::Path::new("/tmp/out.json"))
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn unknown_field_rejected() {
        let path = tmp_path("unknown.yaml");
        std::fs::write(&path, "not_a_real_field: true\n").expect("write");
        let err = Config::load(&path).expect_err("should reject unknown field");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("not_a_real_field") || msg.contains("unknown field"),
            "error was: {msg}"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn json_schema_exports() {
        let schema = Config::json_schema().expect("schema");
        assert!(schema.contains("\"prompt_dirs\""));
        assert!(schema.contains("\"severity_overrides\""));
        assert!(schema.contains("\"cost_cap_usd\""));
    }

    #[test]
    fn template_yaml_parses_cleanly() {
        let raw = include_str!("../../../.ailint.yaml.template");
        let cfg: Config =
            serde_yaml::from_str(raw).unwrap_or_else(|e| panic!("template failed to parse: {e}"));
        assert!(cfg.paths.respect_gitignore);
    }
}
