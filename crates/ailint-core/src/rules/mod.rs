//! The rule engine: rule trait, IDs, severity, violations, and the registry
//! that dispatches to the concrete rule modules.

pub mod consistency;
pub mod registry;
pub mod security;
pub mod semantic;
pub mod structural;

use std::fmt;

use serde::Serialize;

use crate::parser::ParsedDocument;

/// Numeric-code + slug identifier for a rule (e.g. `AIL001` /
/// `no-frontmatter-schema-error`). Every rule owns exactly one `RuleId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct RuleId {
    pub code: u16,
    pub slug: &'static str,
}

impl RuleId {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
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
    pub rule_id: RuleId,
    pub severity: Severity,
    pub message: String,
    pub file: std::path::PathBuf,
    /// 1-based line number, if known.
    pub line: Option<usize>,
    /// 1-based column, if known.
    pub column: Option<usize>,
    // TODO: fix hints, snippet, source URL for the rule doc.
}

/// Trait implemented by every lint rule.
pub trait Rule: Send + Sync {
    fn id(&self) -> RuleId;
    fn default_severity(&self) -> Severity;

    /// Run the rule against a single parsed document.
    ///
    /// TODO: many rules will need cross-file context (`consistency::*`).
    /// Introduce a second trait or a `run_batch` method for those.
    fn run(&self, doc: &ParsedDocument) -> Vec<Violation>;
}
