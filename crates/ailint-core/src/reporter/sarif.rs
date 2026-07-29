//! SARIF 2.1.0 output for GitHub code scanning.
//!
//! TODO: implement a proper SARIF serializer. Consider pulling in a crate
//! (e.g. `serde-sarif`) or hand-rolling the minimum SARIF the GitHub code
//! scanning ingestion pipeline requires.

use std::io::Write;

use anyhow::Result;

use crate::reporter::Reporter;
use crate::rules::Violation;

#[derive(Debug, Default)]
pub struct SarifReporter;

impl Reporter for SarifReporter {
    fn report(&self, _violations: &[Violation], out: &mut dyn Write) -> Result<()> {
        // TODO: real SARIF. Emit a valid empty run so downstream tooling can
        // still consume the output during scaffolding.
        let stub = serde_json::json!({
            "version": "2.1.0",
            "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "ailint",
                        "informationUri": "https://github.com/OWNER/ailint",
                        "version": env!("CARGO_PKG_VERSION"),
                        "rules": []
                    }
                },
                "results": []
            }]
        });
        serde_json::to_writer_pretty(&mut *out, &stub)?;
        writeln!(out)?;
        Ok(())
    }
}
