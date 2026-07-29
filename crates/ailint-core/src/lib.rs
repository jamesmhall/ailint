//! # ailint-core
//!
//! Core library for `ailint`: discovers AI agent guidance files, parses them,
//! runs a configurable set of lint rules, and emits reports in multiple
//! formats.
//!
//! The public surface is intentionally small right now — this crate is a
//! scaffold and most modules contain `TODO` markers.

#![deny(rust_2018_idioms)]
#![warn(missing_debug_implementations)]
// TODO: enable `#![warn(missing_docs)]` once the public API stabilizes.

pub mod config;
pub mod discovery;
pub mod file_type;
pub mod parser;
pub mod reporter;
pub mod rules;

use std::path::{Path, PathBuf};

use anyhow::Result;

pub use crate::config::Config;
pub use crate::file_type::FileType;
pub use crate::reporter::{Reporter, ReporterKind};
pub use crate::rules::{RuleId, Severity, Violation};

/// A single discovered guidance file and its detected type.
#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    pub path: PathBuf,
    pub file_type: FileType,
}

/// Top-level entry point: discover files under `root`, run enabled rules, and
/// return the aggregated violations.
///
/// TODO: wire this to the real discovery + rule engine. Currently returns an
/// empty vec so downstream code (CLI, tests) can compile.
pub fn lint(root: &Path, _config: &Config) -> Result<Vec<Violation>> {
    tracing::debug!(root = %root.display(), "ailint::lint invoked (stub)");
    // TODO: discovery::walk(root, config) -> Vec<DiscoveredFile>
    // TODO: parser::parse(&file) -> ParsedDocument
    // TODO: rules::registry::run_all(&doc, config) -> Vec<Violation>
    Ok(Vec::new())
}

/// Version string embedded from Cargo.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
