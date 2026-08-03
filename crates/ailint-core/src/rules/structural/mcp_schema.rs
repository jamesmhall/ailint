//! AIL004 `mcp-schema-validation` — MCP server config schema check.
//!
//! See: `docs/rules/structural/AIL004.md`

use serde_json::Value;

use crate::file_type::FileType;
use crate::parser::{DocumentContent, ParsedDocument};
use crate::rules::structural::AIL004;
use crate::rules::{Rule, RuleContext, RuleId, Severity, Violation};

/// AIL004 mcp-schema-validation: MCP config files must define a valid
/// `mcpServers` (or VS Code `servers`) map where each entry declares a
/// transport (`command` for stdio or `url` for HTTP/SSE).
#[derive(Debug, Default)]
pub struct McpSchemaValidationRule;

impl Rule for McpSchemaValidationRule {
    fn id(&self) -> RuleId {
        AIL004
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn description(&self) -> &'static str {
        "MCP server config is missing required fields or uses the wrong types."
    }

    fn fix_hint(&self) -> &'static str {
        "Give each server a `command` (stdio) or `url` (http/sse) and use the documented types."
    }

    fn applies_to(&self, file_type: FileType) -> bool {
        matches!(file_type, FileType::McpConfig)
    }

    fn run(&self, doc: &ParsedDocument, _ctx: &RuleContext<'_>) -> Vec<Violation> {
        let json = match &doc.content {
            DocumentContent::Json(v) => v,
            // Parse errors are handled by malformed-json / structural rules.
            _ => return Vec::new(),
        };

        let mut out = Vec::new();

        let root = match json.as_object() {
            Some(o) => o,
            None => {
                out.push(finding(doc, "root must be a JSON object"));
                return out;
            }
        };

        // Either `mcpServers` (Claude/Cline/Cursor) or `servers` (VS Code).
        let servers = root.get("mcpServers").or_else(|| root.get("servers"));
        let servers = match servers {
            Some(s) => s,
            None => {
                out.push(finding(
                    doc,
                    "missing top-level `mcpServers` (or `servers`) object",
                ));
                return out;
            }
        };

        let servers_map = match servers.as_object() {
            Some(m) => m,
            None => {
                out.push(finding(doc, "`mcpServers` must be a JSON object"));
                return out;
            }
        };

        if servers_map.is_empty() {
            out.push(finding(doc, "`mcpServers` object is empty"));
        }

        for (name, cfg) in servers_map {
            validate_server(doc, name, cfg, &mut out);
        }
        out
    }
}

fn validate_server(doc: &ParsedDocument, name: &str, cfg: &Value, out: &mut Vec<Violation>) {
    let obj = match cfg.as_object() {
        Some(o) => o,
        None => {
            out.push(finding(
                doc,
                &format!("server `{name}` must be a JSON object"),
            ));
            return;
        }
    };

    let has_command = obj.get("command").is_some();
    let has_url = obj.get("url").is_some();
    if !has_command && !has_url {
        out.push(finding(
            doc,
            &format!("server `{name}` needs a `command` (stdio) or `url` (http/sse) field"),
        ));
    }

    if let Some(cmd) = obj.get("command") {
        if !cmd.is_string() {
            out.push(finding(
                doc,
                &format!("server `{name}`: `command` must be a string"),
            ));
        }
    }
    if let Some(url) = obj.get("url") {
        if !url.is_string() {
            out.push(finding(
                doc,
                &format!("server `{name}`: `url` must be a string"),
            ));
        }
    }
    if let Some(args) = obj.get("args") {
        let bad = !args.is_array()
            || args
                .as_array()
                .is_some_and(|arr| arr.iter().any(|v| !v.is_string()));
        if bad {
            out.push(finding(
                doc,
                &format!("server `{name}`: `args` must be an array of strings"),
            ));
        }
    }
    if let Some(env) = obj.get("env") {
        let bad = !env.is_object()
            || env
                .as_object()
                .is_some_and(|m| m.values().any(|v| !v.is_string()));
        if bad {
            out.push(finding(
                doc,
                &format!("server `{name}`: `env` must be an object of string values"),
            ));
        }
    }
}

fn finding(doc: &ParsedDocument, msg: &str) -> Violation {
    Violation::new(
        AIL004,
        Severity::Error,
        doc.path.clone(),
        "invalid MCP server config",
    )
    .with_detail(msg.to_string())
}
