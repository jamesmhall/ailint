//! Parsers for each supported guidance format.
//!
//! TODO: unify these behind a `Parser` trait once we know what all
//! call sites need. For now each submodule exposes a plain function.

pub mod json;
pub mod markdown;
pub mod yaml;

use std::path::Path;

use anyhow::Result;

use crate::file_type::FileType;

/// A parsed guidance document — the input to the rule engine.
#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub path: std::path::PathBuf,
    pub file_type: FileType,
    pub raw: String,
    // TODO: hold format-specific representations (Markdown AST, YAML value,
    // etc.) so rules don't have to re-parse.
}

/// Read and parse a guidance file according to its detected type.
///
/// TODO: implement the format-specific paths (markdown AST, YAML/JSON parse).
pub fn parse(path: &Path, file_type: FileType) -> Result<ParsedDocument> {
    let raw = std::fs::read_to_string(path)?;
    Ok(ParsedDocument {
        path: path.to_path_buf(),
        file_type,
        raw,
    })
}
