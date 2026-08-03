//! Parsers for each supported guidance format.

pub mod json;
pub mod markdown;
pub mod source_comments;
pub mod yaml;

use std::path::Path;

use anyhow::Result;

use crate::file_type::FileType;

/// A parsed guidance document — the input to the rule engine.
#[derive(Debug, Clone)]
pub struct ParsedDocument {
    /// Path the document was read from.
    pub path: std::path::PathBuf,
    /// Detected guidance file type.
    pub file_type: FileType,
    /// Full original file contents.
    pub raw: String,
    /// Format-specific parsed representation.
    pub content: DocumentContent,
}

/// Format-specific parsed representation for a document.
#[derive(Debug, Clone)]
pub enum DocumentContent {
    /// Parsed Markdown structure.
    Markdown(markdown::MarkdownDoc),
    /// Parsed YAML value.
    Yaml(serde_yaml::Value),
    /// Parsed JSON value.
    Json(serde_json::Value),
    /// The file was recognized as YAML or JSON but failed to parse. The
    /// stored string is the parser error message so rules like
    /// `malformed-yaml` (AIL041) can surface it.
    ParseError(String),
    /// Plain text with no recognized structure.
    Text,
    /// A zero-length or whitespace-only file.
    Empty,
}

/// Read and parse a guidance file according to its detected type.
pub fn parse(path: &Path, file_type: FileType) -> Result<ParsedDocument> {
    let raw = std::fs::read_to_string(path)?;
    let content = dispatch(&raw, file_type, path);
    Ok(ParsedDocument {
        path: path.to_path_buf(),
        file_type,
        raw,
        content,
    })
}

fn dispatch(raw: &str, file_type: FileType, path: &Path) -> DocumentContent {
    if raw.is_empty() {
        return DocumentContent::Empty;
    }
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase());
    let ext_str = ext.as_deref();

    match file_type {
        FileType::ClaudeMd
        | FileType::AgentsMd
        | FileType::CopilotCustomization
        | FileType::CopilotInstructions
        | FileType::JunieGuidelines
        | FileType::AiderConventions
        | FileType::GitHubSkill
        | FileType::CustomProjectRules
        | FileType::GenericMarkdown => DocumentContent::Markdown(markdown::parse(raw)),

        FileType::CursorRules
        | FileType::WindsurfRules
        | FileType::ClineRules
        | FileType::ContinueRules => match ext_str {
            Some("md") | Some("markdown") | Some("mdc") => {
                DocumentContent::Markdown(markdown::parse(raw))
            }
            Some("yaml") | Some("yml") => match yaml::parse(raw) {
                Ok(v) => DocumentContent::Yaml(v),
                Err(e) => DocumentContent::ParseError(e.to_string()),
            },
            Some("json") => match json::parse(raw) {
                Ok(v) => DocumentContent::Json(v),
                Err(e) => DocumentContent::ParseError(e.to_string()),
            },
            _ => DocumentContent::Markdown(markdown::parse(raw)),
        },

        FileType::GenericSystemPrompt => match ext_str {
            Some("json") => match json::parse(raw) {
                Ok(v) => DocumentContent::Json(v),
                Err(e) => DocumentContent::ParseError(e.to_string()),
            },
            Some("yaml") | Some("yml") => match yaml::parse(raw) {
                Ok(v) => DocumentContent::Yaml(v),
                Err(e) => DocumentContent::ParseError(e.to_string()),
            },
            Some("md") | Some("markdown") => DocumentContent::Markdown(markdown::parse(raw)),
            _ => DocumentContent::Markdown(markdown::parse(raw)),
        },

        FileType::GenericYaml => match yaml::parse(raw) {
            Ok(v) => DocumentContent::Yaml(v),
            Err(e) => DocumentContent::ParseError(e.to_string()),
        },

        FileType::McpConfig => match json::parse(raw) {
            Ok(v) => DocumentContent::Json(v),
            Err(e) => DocumentContent::ParseError(e.to_string()),
        },

        FileType::SourceCode(lang) => {
            DocumentContent::Markdown(source_comments::synthesize(raw, lang))
        }

        FileType::Unknown => DocumentContent::Text,
    }
}
