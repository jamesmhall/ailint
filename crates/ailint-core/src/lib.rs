//! # ailint-core
//!
//! Core library for `ailint`: discovers AI agent guidance files, parses them,
//! runs a configurable set of lint rules, and emits reports in multiple
//! formats.
//!
//! Discovers guidance files via [`discovery`], parses them via [`parser`],
//! runs rules registered in [`rules::registry`], and emits reports via
//! [`reporter`].

#![deny(rust_2018_idioms)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]

pub mod config;
pub mod discovery;
pub mod file_type;
pub mod fix;
pub mod parser;
pub mod reporter;
pub mod rules;

use std::path::{Path, PathBuf};

use anyhow::Result;

pub use crate::config::Config;
pub use crate::file_type::FileType;
pub use crate::fix::{apply_all as apply_fixes, FileFixResult, FixConflict};
pub use crate::reporter::{Reporter, ReporterKind};
pub use crate::rules::{RuleId, Severity, TextEdit, Violation};

/// A single discovered guidance file and its detected type.
#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    /// Location on disk.
    pub path: PathBuf,
    /// Detected guidance file type.
    pub file_type: FileType,
}

/// Top-level entry point: discover files under `root`, run enabled rules, and
/// return the aggregated violations.
pub fn lint(root: &Path, config: &Config) -> Result<Vec<Violation>> {
    let files = discovery::walk(root, config)?;
    tracing::debug!(root = %root.display(), files = files.len(), "ailint::lint");
    let mut docs = Vec::with_capacity(files.len());
    for file in files {
        docs.push(parser::parse(&file.path, file.file_type)?);
    }
    let mut violations = Vec::new();
    for doc in &docs {
        violations.extend(rules::registry::run_all(doc, config));
    }
    violations.extend(rules::registry::run_all_batch(&docs, config));
    Ok(violations)
}

/// Version string embedded from Cargo.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
