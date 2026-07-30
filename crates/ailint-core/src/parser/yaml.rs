//! YAML parsing for guidance files that store rules as YAML
//! (e.g. Cursor rules, Windsurf rules).

use anyhow::Result;

/// Parse a YAML string into a generic `serde_yaml::Value`.
pub fn parse(input: &str) -> Result<serde_yaml::Value> {
    Ok(serde_yaml::from_str(input)?)
}
