//! JSON output for machine consumption.

use std::io::Write;

use anyhow::Result;

use crate::reporter::Reporter;
use crate::rules::Violation;

#[derive(Debug, Default)]
pub struct JsonReporter;

impl Reporter for JsonReporter {
    fn report(&self, violations: &[Violation], out: &mut dyn Write) -> Result<()> {
        // TODO: stable, versioned schema — include tool version, timestamp,
        // summary counts, file list, and violations.
        serde_json::to_writer_pretty(&mut *out, &violations)?;
        writeln!(out)?;
        Ok(())
    }
}
