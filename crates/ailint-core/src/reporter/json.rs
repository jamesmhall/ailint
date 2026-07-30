//! JSON output for machine consumption.

use std::collections::BTreeMap;
use std::io::Write;

use anyhow::Result;
use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serialize;

use crate::reporter::Reporter;
use crate::rules::{Severity, Violation};
use crate::VERSION;

const SCHEMA_VERSION: &str = "1";

/// JSON reporter emitting a versioned schema for machine consumers.
#[derive(Debug)]
pub struct JsonReporter {
    now: fn() -> DateTime<Utc>,
}

impl Default for JsonReporter {
    fn default() -> Self {
        Self { now: Utc::now }
    }
}

impl JsonReporter {
    /// Override the clock — used by snapshot tests to keep output stable.
    pub fn with_now(now: fn() -> DateTime<Utc>) -> Self {
        Self { now }
    }
}

impl Reporter for JsonReporter {
    fn report(&self, violations: &[Violation], out: &mut dyn Write) -> Result<()> {
        let report = build_report(violations, (self.now)());
        serde_json::to_writer_pretty(&mut *out, &report)?;
        writeln!(out)?;
        Ok(())
    }
}

#[derive(Serialize)]
struct Report<'a> {
    schema_version: &'static str,
    tool: Tool,
    generated_at: String,
    summary: Summary,
    files: Vec<FileEntry>,
    violations: Vec<ViolationEntry<'a>>,
}

#[derive(Serialize)]
struct Tool {
    name: &'static str,
    version: &'static str,
}

#[derive(Serialize)]
struct Summary {
    total: usize,
    errors: usize,
    warnings: usize,
    info: usize,
    file_count: usize,
}

#[derive(Serialize)]
struct FileEntry {
    path: String,
    violation_count: usize,
}

#[derive(Serialize)]
struct ViolationEntry<'a> {
    rule: RuleEntry,
    severity: Severity,
    message: &'a str,
    file: String,
    line: Option<usize>,
    column: Option<usize>,
    fix_hint: Option<&'a str>,
    snippet: Option<&'a str>,
    source_url: Option<&'a str>,
}

#[derive(Serialize)]
struct RuleEntry {
    code: String,
    slug: &'static str,
}

fn build_report(violations: &[Violation], now: DateTime<Utc>) -> Report<'_> {
    let mut errors = 0;
    let mut warnings = 0;
    let mut info = 0;
    let mut per_file: BTreeMap<String, usize> = BTreeMap::new();

    let mut entries = Vec::with_capacity(violations.len());
    for v in violations {
        match v.severity {
            Severity::Error => errors += 1,
            Severity::Warning => warnings += 1,
            Severity::Info => info += 1,
        }
        let file = v.file.to_string_lossy().into_owned();
        *per_file.entry(file.clone()).or_insert(0) += 1;
        entries.push(ViolationEntry {
            rule: RuleEntry {
                code: v.rule_id.code_str(),
                slug: v.rule_id.slug,
            },
            severity: v.severity,
            message: &v.message,
            file,
            line: v.line,
            column: v.column,
            fix_hint: v.fix_hint.as_deref(),
            snippet: v.snippet.as_deref(),
            source_url: v.source_url.as_deref(),
        });
    }

    let files: Vec<FileEntry> = per_file
        .into_iter()
        .map(|(path, violation_count)| FileEntry {
            path,
            violation_count,
        })
        .collect();

    Report {
        schema_version: SCHEMA_VERSION,
        tool: Tool {
            name: "ailint",
            version: VERSION,
        },
        generated_at: now.to_rfc3339_opts(SecondsFormat::Secs, true),
        summary: Summary {
            total: violations.len(),
            errors,
            warnings,
            info,
            file_count: files.len(),
        },
        files,
        violations: entries,
    }
}
