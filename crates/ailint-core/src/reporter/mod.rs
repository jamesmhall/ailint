//! Output formatters for lint results.

pub mod json;
pub mod markdown;
pub mod sarif;
pub mod terminal;

use std::io::Write;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::rules::Violation;

/// Which reporter format to use.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ReporterKind {
    /// Human-readable table on stdout.
    #[default]
    Terminal,
    /// Machine-readable JSON report.
    Json,
    /// SARIF 2.1.0, for code-scanning integrations.
    Sarif,
    /// Markdown table, for PR comments and docs.
    Markdown,
}

/// A reporter serializes violations to some sink.
pub trait Reporter {
    /// Write all violations to `out` in this reporter's format.
    fn report(&self, violations: &[Violation], out: &mut dyn Write) -> Result<()>;
}

/// Construct a boxed reporter for the given kind.
pub fn make(kind: ReporterKind) -> Box<dyn Reporter> {
    match kind {
        ReporterKind::Terminal => Box::new(terminal::TerminalReporter::default()),
        ReporterKind::Json => Box::new(json::JsonReporter::default()),
        ReporterKind::Sarif => Box::new(sarif::SarifReporter),
        ReporterKind::Markdown => Box::new(markdown::MarkdownReporter::default()),
    }
}
