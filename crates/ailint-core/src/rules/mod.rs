//! The rule engine: rule trait, IDs, severity, violations, and the registry
//! that dispatches to the concrete rule modules.

pub mod consistency;
pub mod registry;
pub mod security;
pub mod semantic;
pub mod structural;

use std::fmt;
use std::path::PathBuf;

use serde::Serialize;

use crate::file_type::FileType;
use crate::parser::ParsedDocument;

/// Parse an embedded dictionary asset: one entry per line, skipping blank
/// lines and `#` comments.
pub(crate) fn dictionary_lines(raw: &'static str) -> Vec<&'static str> {
    raw.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

/// Numeric-code + slug identifier for a rule (e.g. `AIL001` /
/// `no-frontmatter-schema-error`). Every rule owns exactly one `RuleId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct RuleId {
    /// Numeric part of the `AILNNN` code.
    pub code: u16,
    /// Kebab-case human-readable name.
    pub slug: &'static str,
}

impl RuleId {
    /// Const constructor so rules can define their ID as a `const`.
    pub const fn new(code: u16, slug: &'static str) -> Self {
        Self { code, slug }
    }

    /// Format the code as `AILNNN`.
    pub fn code_str(&self) -> String {
        format!("AIL{:03}", self.code)
    }
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.code_str(), self.slug)
    }
}

/// Severity of a violation.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Fails the run (non-zero exit).
    Error,
    /// Reported; fails only past `--max-warnings`.
    Warning,
    /// Advisory only.
    Info,
}

impl Severity {
    /// Lowercase name, matching the serde representation.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

/// A concrete finding produced by a rule.
#[derive(Debug, Clone, Serialize)]
pub struct Violation {
    /// Rule that produced this finding.
    pub rule_id: RuleId,
    /// Effective severity (config overrides applied).
    pub severity: Severity,
    /// Human-readable description of the finding.
    pub message: String,
    /// File the finding was raised against.
    pub file: PathBuf,
    /// 1-based line number, if known.
    pub line: Option<usize>,
    /// 1-based column, if known.
    pub column: Option<usize>,
    /// Suggested remediation, if the rule offers one.
    pub fix_hint: Option<String>,
    /// Offending source excerpt, if captured.
    pub snippet: Option<String>,
    /// Link to the rule's documentation page.
    pub source_url: Option<String>,
}

impl Violation {
    /// Create a violation with no location or hint attached.
    pub fn new(
        rule_id: RuleId,
        severity: Severity,
        file: PathBuf,
        message: impl Into<String>,
    ) -> Self {
        Self {
            rule_id,
            severity,
            file,
            message: message.into(),
            line: None,
            column: None,
            fix_hint: None,
            snippet: None,
            source_url: None,
        }
    }

    /// Attach a 1-based line and column.
    pub fn at(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }
}

/// Per-invocation context passed to a rule's `run` method.
#[derive(Debug)]
pub struct RuleContext<'a> {
    /// Full resolved configuration.
    pub config: &'a crate::config::Config,
    /// Rule-specific options, keyed under this rule's slug in `RulesConfig::options`.
    pub options: Option<&'a serde_yaml::Value>,
    /// Severity to use — the rule's default may be overridden via config.
    pub severity: Severity,
}

/// Trait implemented by every per-document lint rule.
pub trait Rule: Send + Sync {
    /// This rule's stable identifier.
    fn id(&self) -> RuleId;
    /// Severity when no config override applies.
    fn default_severity(&self) -> Severity;
    /// Inspect one document and return any findings.
    fn run(&self, doc: &ParsedDocument, ctx: &RuleContext<'_>) -> Vec<Violation>;
    /// Whether this rule should run against files of the given type. Defaults
    /// to "AI guidance files only" — rules that also apply to generic
    /// Markdown / YAML must override this.
    fn applies_to(&self, file_type: FileType) -> bool {
        file_type.is_ai_guidance()
    }
}

/// Trait implemented by rules that need the full corpus at once
/// (cross-file consistency checks).
pub trait BatchRule: Send + Sync {
    /// This rule's stable identifier.
    fn id(&self) -> RuleId;
    /// Severity when no config override applies.
    fn default_severity(&self) -> Severity;
    /// Inspect the whole corpus at once and return any findings.
    fn run_batch(&self, docs: &[ParsedDocument], ctx: &RuleContext<'_>) -> Vec<Violation>;
    /// Filter applied to each document before the batch rule sees it.
    /// Defaults to "AI guidance files only".
    fn applies_to(&self, file_type: FileType) -> bool {
        file_type.is_ai_guidance()
    }
}
