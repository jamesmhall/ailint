//! Central registry of all built-in rules.
//!
//! TODO: this should be data-driven — populate once at startup and let
//! `Config` filter by disabled IDs / severity overrides.

use crate::config::Config;
use crate::parser::ParsedDocument;
use crate::rules::{Rule, Violation};

/// Return the full set of built-in rules.
///
/// TODO: register concrete rule structs from each `structural`, `semantic`,
/// `security`, `consistency` module here.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    Vec::new()
}

/// Run every enabled rule against a document.
pub fn run_all(_doc: &ParsedDocument, _config: &Config) -> Vec<Violation> {
    // TODO:
    //   let mut out = Vec::new();
    //   for rule in all_rules() {
    //       if config.rules.disabled.contains(&rule.id().slug.into()) { continue; }
    //       out.extend(rule.run(doc));
    //   }
    //   out
    Vec::new()
}
