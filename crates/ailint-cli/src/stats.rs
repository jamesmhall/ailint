//! `ailint stats`: aggregate coverage / rule-density report.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Result;

use ailint_core::config::Config;
use ailint_core::file_type::FileType;
use ailint_core::parser::{self, DocumentContent};
use ailint_core::rules::registry;
use ailint_core::rules::Severity;

#[derive(Debug, Default)]
struct Report {
    total_files: usize,
    files_by_type: BTreeMap<String, usize>,
    total_violations: usize,
    violations_by_rule: BTreeMap<String, usize>,
    violations_by_severity: BTreeMap<&'static str, usize>,
    per_file_counts: Vec<(PathBuf, usize)>,
    markdown_files: usize,
    markdown_word_total: usize,
}

pub fn run(paths: &[PathBuf], config: &Config) -> Result<ExitCode> {
    let mut report = Report::default();

    for path in paths {
        for file in ailint_core::discovery::walk(path, config)? {
            report.total_files += 1;
            let type_label = format!("{:?}", file.file_type);
            *report.files_by_type.entry(type_label).or_insert(0) += 1;

            let doc = parser::parse(&file.path, file.file_type)?;
            if matches!(file.file_type, FileType::Unknown) {
                // no-op
            }
            if is_markdown(&doc.content) {
                report.markdown_files += 1;
                report.markdown_word_total += word_count(&doc.raw);
            }

            let violations = registry::run_all(&doc, config);
            let count = violations.len();
            report.total_violations += count;
            report.per_file_counts.push((file.path.clone(), count));
            for v in &violations {
                *report
                    .violations_by_rule
                    .entry(v.rule_id.code_str())
                    .or_insert(0) += 1;
                *report
                    .violations_by_severity
                    .entry(severity_label(v.severity))
                    .or_insert(0) += 1;
            }
        }
    }

    print_report(&report);
    Ok(ExitCode::SUCCESS)
}

fn is_markdown(content: &DocumentContent) -> bool {
    matches!(content, DocumentContent::Markdown(_))
}

fn severity_label(s: Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

fn word_count(raw: &str) -> usize {
    raw.split_whitespace().count()
}

fn print_report(r: &Report) {
    println!("ailint stats");
    println!("============");
    println!("total files: {}", r.total_files);
    if !r.files_by_type.is_empty() {
        println!();
        println!("files by type:");
        for (k, v) in &r.files_by_type {
            println!("  {:<28} {}", k, v);
        }
    }

    println!();
    println!("total violations: {}", r.total_violations);
    if !r.violations_by_severity.is_empty() {
        println!("  by severity:");
        for (k, v) in &r.violations_by_severity {
            println!("    {:<10} {}", k, v);
        }
    }
    if !r.violations_by_rule.is_empty() {
        println!("  by rule:");
        for (k, v) in &r.violations_by_rule {
            println!("    {:<10} {}", k, v);
        }
    }

    let top = top_n(&r.per_file_counts, 5);
    if !top.is_empty() {
        println!();
        println!("top files by violation count:");
        for (path, count) in top {
            println!("  {:>4}  {}", count, display_path(path));
        }
    }

    if r.markdown_files > 0 {
        let avg = r.markdown_word_total as f64 / r.markdown_files as f64;
        println!();
        println!(
            "markdown: {} file(s), avg {:.1} words/file",
            r.markdown_files, avg
        );
    }
}

fn top_n(items: &[(PathBuf, usize)], n: usize) -> Vec<(&PathBuf, usize)> {
    let mut ranked: Vec<(&PathBuf, usize)> = items
        .iter()
        .filter(|(_, c)| *c > 0)
        .map(|(p, c)| (p, *c))
        .collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    ranked.truncate(n);
    ranked
}

fn display_path(p: &Path) -> String {
    p.display().to_string()
}
