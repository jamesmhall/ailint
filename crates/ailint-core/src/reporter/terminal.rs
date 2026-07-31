//! Colored, ESLint-style terminal output.

use std::collections::BTreeMap;
use std::io::{IsTerminal, Write};

use anyhow::Result;
use colored::Colorize;

use crate::config::ColorMode;
use crate::reporter::Reporter;
use crate::rules::{Severity, Violation};

const MESSAGE_WIDTH: usize = 60;

/// Renders violations grouped by rule as an aligned, optionally colored table.
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

        let mut by_rule: BTreeMap<String, Vec<&Violation>> = BTreeMap::new();
        let mut max_path_len = 0;

        for v in violations {
            let rule = format!("{}/{}", v.rule_id.code_str(), v.rule_id.slug);
            by_rule.entry(rule).or_default().push(v);

            let pos_len = match (v.line, v.column) {
                (Some(l), Some(c)) => l.to_string().len() + c.to_string().len() + 2,
                (Some(l), None) => l.to_string().len() + 1,
                _ => 0,
            };
            let path_len = v.file.display().to_string().chars().count() + pos_len;
            if path_len > max_path_len {
                max_path_len = path_len;
            }
        }

        let path_pad = max_path_len.clamp(20, 60);

        let mut first_group = true;
        for (rule, mut items) in by_rule {
            items.sort_by(|a, b| {
                a.file
                    .cmp(&b.file)
                    .then(a.line.unwrap_or(0).cmp(&b.line.unwrap_or(0)))
                    .then(a.column.unwrap_or(0).cmp(&b.column.unwrap_or(0)))
            });

            if !first_group {
                writeln!(out)?;
            }
            first_group = false;

            let sev = items[0].severity;

            let rule_colored = if color_on {
                rule.bold().to_string()
            } else {
                rule.clone()
            };

            let sev_colored = if color_on {
                match sev {
                    Severity::Error => sev.as_str().red().bold().to_string(),
                    Severity::Warning => sev.as_str().yellow().bold().to_string(),
                    Severity::Info => sev.as_str().cyan().to_string(),
                }
            } else {
                sev.as_str().to_string()
            };

            let glyph = if color_on {
                match sev {
                    Severity::Error | Severity::Warning => "✖".red().to_string(),
                    Severity::Info => "ℹ".cyan().to_string(),
                }
            } else {
                match sev {
                    Severity::Error | Severity::Warning => "x".to_string(),
                    Severity::Info => "i".to_string(),
                }
            };

            writeln!(out, "{} {} ({})", glyph, rule_colored, sev_colored)?;

            for v in items {
                let pos = match (v.line, v.column) {
                    (Some(l), Some(c)) => format!(":{}:{}", l, c),
                    (Some(l), None) => format!(":{}", l),
                    _ => String::new(),
                };

                let path_str = format!("{}{}", v.file.display(), pos);
                let path_colored = if color_on {
                    path_str.dimmed().to_string()
                } else {
                    path_str.clone()
                };

                let msg = truncate_chars(&v.message, MESSAGE_WIDTH);

                let visual_len = path_str.chars().count();
                let pad_len = path_pad.saturating_sub(visual_len);
                let spaces = " ".repeat(pad_len);

                writeln!(out, "  {}{}  {}", path_colored, spaces, msg)?;
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
