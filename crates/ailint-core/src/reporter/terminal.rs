//! Colored, human-readable terminal output.

use std::io::Write;

use anyhow::Result;

use crate::reporter::Reporter;
use crate::rules::Violation;

#[derive(Debug, Default)]
pub struct TerminalReporter;

impl Reporter for TerminalReporter {
    fn report(&self, violations: &[Violation], out: &mut dyn Write) -> Result<()> {
        // TODO: proper ESLint-style output with file grouping, colored
        // severities via `colored`, and a summary line. This stub is only
        // sufficient to compile.
        if violations.is_empty() {
            writeln!(out, "ailint: no violations")?;
            return Ok(());
        }
        for v in violations {
            writeln!(
                out,
                "{}:{}:{} {} {} {}",
                v.file.display(),
                v.line.unwrap_or(0),
                v.column.unwrap_or(0),
                v.severity.as_str(),
                v.rule_id,
                v.message,
            )?;
        }
        writeln!(out, "\n{} violation(s)", violations.len())?;
        Ok(())
    }
}
