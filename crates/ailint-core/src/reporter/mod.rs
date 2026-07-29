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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReporterKind {
    #[default]
    Terminal,
    Json,
    Sarif,
    Markdown,
}

/// A reporter serializes violations to some sink.
pub trait Reporter {
    fn report(&self, violations: &[Violation], out: &mut dyn Write) -> Result<()>;
}

/// Construct a boxed reporter for the given kind.
pub fn make(kind: ReporterKind) -> Box<dyn Reporter> {
    match kind {
        ReporterKind::Terminal => Box::new(terminal::TerminalReporter),
        ReporterKind::Json => Box::new(json::JsonReporter),
        ReporterKind::Sarif => Box::new(sarif::SarifReporter),
        ReporterKind::Markdown => Box::new(markdown::MarkdownReporter),
    }
}
