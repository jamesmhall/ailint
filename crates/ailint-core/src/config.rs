//! Configuration loading for ailint (`.ailint.yaml`).
//!
//! TODO: define the full schema, add JSON schema export, and support
//! `AILINT_CONFIG` env override.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::reporter::ReporterKind;
use crate::rules::Severity;

/// Root configuration object, mirrored from `.ailint.yaml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub rules: RulesConfig,
    pub paths: PathsConfig,
    pub llm: Option<LlmConfig>,
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RulesConfig {
    /// Rule IDs or slugs to disable (e.g. `AIL100` or `no-vague-instruction`).
    pub disabled: Vec<String>,
    /// Overrides for rule severity, keyed by ID or slug.
    #[serde(default)]
    pub severity_overrides: std::collections::BTreeMap<String, Severity>,
    // TODO: per-rule options (map<RuleId, serde_yaml::Value>).
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PathsConfig {
    /// Additional path globs to exclude from discovery.
    pub exclude: Vec<String>,
    // TODO: `include` allow-list, follow-symlinks, respect_gitignore flag.
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LlmConfig {
    pub provider: LlmProviderKind,
    pub model: String,
    // TODO: base_url override for OpenAI-compatible endpoints, timeout, max
    // tokens, temperature, cost caps.
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LlmProviderKind {
    Openai,
    Anthropic,
    Google,
    Ollama,
    /// Any OpenAI-compatible endpoint.
    Compatible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OutputConfig {
    pub format: ReporterKind,
    pub color: ColorMode,
    // TODO: `output_file` default, quiet flag, summary format.
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: ReporterKind::Terminal,
            color: ColorMode::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}

impl Config {
    /// Load a config file from disk. Returns `Config::default()` if the file
    /// does not exist.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        // TODO: support .ailint.json alongside .ailint.yaml.
        let cfg: Self = serde_yaml::from_str(&raw)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        Ok(cfg)
    }

    /// Locate the closest config file, walking up from `start`.
    pub fn discover(_start: &Path) -> Option<std::path::PathBuf> {
        // TODO: walk parents looking for `.ailint.yaml` / `.ailint.yml` /
        // `.ailint.json`.
        None
    }
}
