//! JSON parsing for guidance files that store rules as JSON.
//!
//! TODO: return a typed representation once the schemas are known.

use anyhow::Result;

/// Parse a JSON string into a generic `serde_json::Value`.
pub fn parse(input: &str) -> Result<serde_json::Value> {
    Ok(serde_json::from_str(input)?)
}
