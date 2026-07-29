//! Markdown report output (e.g. for pasting into a PR comment).

use std::io::Write;

use anyhow::Result;

use crate::reporter::Reporter;
use crate::rules::Violation;

#[derive(Debug, Default)]
pub struct MarkdownReporter;

impl Reporter for MarkdownReporter {
    fn report(&self, violations: &[Violation], out: &mut dyn Write) -> Result<()> {
        // TODO: proper Markdown report with a summary table, per-file
        // sections, links to rule docs, and severity badges.
        writeln!(out, "# ailint report")?;
        writeln!(out)?;
        writeln!(out, "**{} violation(s)**", violations.len())?;
        writeln!(out)?;
        writeln!(out, "| Severity | Rule | File | Line | Message |")?;
        writeln!(out, "|----------|------|------|------|---------|")?;
        for v in violations {
            writeln!(
                out,
                "| {} | `{}` | `{}` | {} | {} |",
                v.severity.as_str(),
                v.rule_id,
                v.file.display(),
                v.line.map(|n| n.to_string()).unwrap_or_default(),
                v.message,
            )?;
        }
        Ok(())
    }
}
