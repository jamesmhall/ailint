//! Colored, ESLint-style terminal output.

use std::collections::BTreeMap;
use std::io::{IsTerminal, Write};

use anyhow::Result;
use colored::Colorize;

use crate::config::ColorMode;
use crate::reporter::Reporter;
use crate::rules::{Severity, Violation};

const MESSAGE_WIDTH: usize = 60;
const SEVERITY_WIDTH: usize = 7;
const POSITION_WIDTH: usize = 5;

/// Renders violations as an aligned, optionally colored table.
#[derive(Debug, Clone, Default)]
pub struct TerminalReporter {
    color: ColorMode,
}

impl TerminalReporter {
    /// Reporter with an explicit color mode instead of the default `Auto`.
    pub fn new(color: ColorMode) -> Self {
        Self { color }
    }

    fn color_on(&self) -> bool {
        match self.color {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => std::io::stdout().is_terminal(),
        }
    }
}

impl Reporter for TerminalReporter {
    fn report(&self, violations: &[Violation], out: &mut dyn Write) -> Result<()> {
        if violations.is_empty() {
            writeln!(out, "ailint: no violations")?;
            return Ok(());
        }

        let color_on = self.color_on();

        let mut by_file: BTreeMap<String, Vec<&Violation>> = BTreeMap::new();
        for v in violations {
            by_file
                .entry(v.file.display().to_string())
                .or_default()
                .push(v);
        }

        // Message column widens to the longest truncated message across the batch.
        let max_msg_len = violations
            .iter()
            .map(|v| truncate_chars(&v.message, MESSAGE_WIDTH).chars().count())
            .max()
            .unwrap_or(0);

        let mut first_group = true;
        for (path, mut items) in by_file {
            items.sort_by_key(|v| (v.line.unwrap_or(0), v.column.unwrap_or(0)));

            if !first_group {
                writeln!(out)?;
            }
            first_group = false;

            if color_on {
                writeln!(out, "{}", path.underline())?;
            } else {
                writeln!(out, "{path}")?;
            }

            for v in items {
                let line = v.line.unwrap_or(0);
                let col = v.column.unwrap_or(0);
                let pos = format!("{line}:{col}");
                let pos_col = pad_left(&pos, POSITION_WIDTH);

                let sev_padded = pad_right(v.severity.as_str(), SEVERITY_WIDTH);
                let sev_col = if color_on {
                    match v.severity {
                        Severity::Error => sev_padded.red().bold().to_string(),
                        Severity::Warning => sev_padded.yellow().bold().to_string(),
                        Severity::Info => sev_padded.cyan().to_string(),
                    }
                } else {
                    sev_padded
                };

                let msg = truncate_chars(&v.message, MESSAGE_WIDTH);
                let msg_col = pad_right(&msg, max_msg_len);

                let rule = format!("{}/{}", v.rule_id.code_str(), v.rule_id.slug);
                let rule_col = if color_on {
                    rule.dimmed().to_string()
                } else {
                    rule
                };

                writeln!(out, "{pos_col}  {sev_col} {msg_col}  {rule_col}")?;
            }
        }

        let (mut errors, mut warnings, mut info) = (0usize, 0usize, 0usize);
        for v in violations {
            match v.severity {
                Severity::Error => errors += 1,
                Severity::Warning => warnings += 1,
                Severity::Info => info += 1,
            }
        }
        let total = violations.len();
        let glyph = if color_on { "✖" } else { "x" };
        let summary = format!(
            "{glyph} {total} {} ({errors} {}, {warnings} {}, {info} info)",
            pluralize(total, "violation", "violations"),
            pluralize(errors, "error", "errors"),
            pluralize(warnings, "warning", "warnings"),
        );

        writeln!(out)?;
        if color_on {
            writeln!(out, "{}", summary.red().bold())?;
        } else {
            writeln!(out, "{summary}")?;
        }

        Ok(())
    }
}

fn pluralize(n: usize, singular: &'static str, plural: &'static str) -> &'static str {
    if n == 1 {
        singular
    } else {
        plural
    }
}

fn truncate_chars(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    let keep = max.saturating_sub(1);
    let mut out: String = s.chars().take(keep).collect();
    out.push('…');
    out
}

fn pad_right(s: &str, width: usize) -> String {
    let count = s.chars().count();
    if count >= width {
        return s.to_string();
    }
    let mut out = String::from(s);
    for _ in 0..(width - count) {
        out.push(' ');
    }
    out
}

fn pad_left(s: &str, width: usize) -> String {
    let count = s.chars().count();
    if count >= width {
        return s.to_string();
    }
    let mut out = String::with_capacity(width);
    for _ in 0..(width - count) {
        out.push(' ');
    }
    out.push_str(s);
    out
}
