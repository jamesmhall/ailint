//! SARIF 2.1.0 output for GitHub code scanning.

use std::io::Write;

use anyhow::Result;
use serde_json::json;
use serde_sarif::sarif::{
    ArtifactChange, ArtifactLocation, Fix, Location, Message, MultiformatMessageString,
    PhysicalLocation, Region, ReportingConfiguration, ReportingDescriptor, Result as SarifResult,
    Run, Sarif, Tool, ToolComponent,
};

use crate::reporter::Reporter;
use crate::rules::{registry, Severity, Violation};
use crate::VERSION;

const INFORMATION_URI: &str = "https://github.com/jamesmhall/ailint";
const SCHEMA_URI: &str = "https://json.schemastore.org/sarif-2.1.0.json";

/// Renders violations as a SARIF 2.1.0 log.
#[derive(Debug, Default)]
pub struct SarifReporter;

impl Reporter for SarifReporter {
    fn report(&self, violations: &[Violation], out: &mut dyn Write) -> Result<()> {
        let rules = build_rule_descriptors();
        let results = violations.iter().map(build_result).collect::<Vec<_>>();

        let driver = ToolComponent::builder()
            .name("ailint".to_string())
            .information_uri(INFORMATION_URI.to_string())
            .version(VERSION.to_string())
            .rules(rules)
            .build();

        let tool = Tool::builder().driver(driver).build();

        let run = Run::builder().tool(tool).results(results).build();

        let sarif = Sarif::builder()
            .version(json!("2.1.0"))
            .schema(SCHEMA_URI.to_string())
            .runs(vec![run])
            .build();

        serde_json::to_writer_pretty(&mut *out, &sarif)?;
        writeln!(out)?;
        Ok(())
    }
}

fn build_rule_descriptors() -> Vec<ReportingDescriptor> {
    let mut descriptors = Vec::new();
    for rule in registry::all_rules() {
        descriptors.push(descriptor_for(
            rule.id().code_str(),
            rule.id().slug,
            rule.description(),
            rule.fix_hint(),
            rule.default_severity(),
        ));
    }
    for rule in registry::all_batch_rules() {
        descriptors.push(descriptor_for(
            rule.id().code_str(),
            rule.id().slug,
            rule.description(),
            rule.fix_hint(),
            rule.default_severity(),
        ));
    }
    descriptors
}

fn descriptor_for(
    id: String,
    slug: &'static str,
    description: &'static str,
    fix_hint: &'static str,
    severity: Severity,
) -> ReportingDescriptor {
    let short_description = MultiformatMessageString::builder()
        .text(if description.is_empty() {
            slug.to_string()
        } else {
            description.to_string()
        })
        .build();
    let default_configuration = ReportingConfiguration::builder()
        .level(json!(sarif_level(severity)))
        .build();
    let help = if fix_hint.is_empty() {
        None
    } else {
        Some(
            MultiformatMessageString::builder()
                .text(fix_hint.to_string())
                .build(),
        )
    };
    let builder = ReportingDescriptor::builder()
        .id(id)
        .name(slug.to_string())
        .short_description(short_description)
        .help_uri(format!("https://github.com/jamesmhall/ailint#{slug}"))
        .default_configuration(default_configuration);
    match help {
        Some(h) => builder.help(h).build(),
        None => builder.build(),
    }
}

fn build_result(v: &Violation) -> SarifResult {
    let message_text = match v.detail.as_deref() {
        Some(d) if !d.is_empty() => format!("{}: {}", v.message, d),
        _ => v.message.clone(),
    };
    let message = Message::builder().text(message_text).build();

    let artifact_location = ArtifactLocation::builder()
        .uri(v.file.to_string_lossy().into_owned())
        .build();
    let region = Region::builder()
        .start_line(v.line.unwrap_or(1) as i64)
        .start_column(v.column.unwrap_or(1) as i64)
        .build();
    let physical_location = PhysicalLocation::builder()
        .artifact_location(artifact_location)
        .region(region)
        .build();
    let location = Location::builder()
        .physical_location(physical_location)
        .build();

    let builder = SarifResult::builder()
        .rule_id(v.rule_id.code_str())
        .level(json!(sarif_level(v.severity)))
        .message(message)
        .locations(vec![location]);

    match v.fix_hint.as_ref() {
        Some(hint) => builder.fixes(vec![fix_from_hint(hint, v)]).build(),
        None => builder.build(),
    }
}

fn fix_from_hint(hint: &str, v: &Violation) -> Fix {
    let description = Message::builder().text(hint.to_string()).build();
    let artifact_location = ArtifactLocation::builder()
        .uri(v.file.to_string_lossy().into_owned())
        .build();
    let change = ArtifactChange::builder()
        .artifact_location(artifact_location)
        .build();
    Fix::builder()
        .description(description)
        .artifact_changes(vec![change])
        .build()
}

fn sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "note",
    }
}
