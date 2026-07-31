//! Human- and LLM-readable terminal output, grouped by rule and by file.
//!
//! Output shape:
//!
//! ```text
//! ⨯ AIL040  broken-local-link                  warning · 79 findings
//!    Markdown link points to a path that does not exist on disk.
//!    Fix: Correct the path, or remove the link.
//!
//!    .claude/agents/belva-project.md · 9
//!      L28   ../.github/skills/story-writing/SKILL.md
//!      L29   ../.github/skills/bug-writing/SKILL.md
//!    ...
//! ```

use std::collections::BTreeMap;
use std::io::{IsTerminal, Write};
use std::path::{Component, Path, PathBuf};

use anyhow::Result;
use colored::Colorize;

use crate::config::ColorMode;
use crate::reporter::Reporter;
use crate::rules::registry::rule_meta;
use crate::rules::{RuleId, Severity, Violation};

/// Renders violations grouped by rule, then by file, in an aligned layout
/// designed to be easy for both humans and LLMs to audit.
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
        let prefix = common_path_prefix(violations.iter().map(|v| v.file.as_path()));

        // Group by rule, stable-sort inside each group by (file, line, column).
        let mut by_rule: BTreeMap<RuleId, Vec<&Violation>> = BTreeMap::new();
        for v in violations {
            by_rule.entry(v.rule_id).or_default().push(v);
        }

        let mut first_group = true;
        for (rule_id, mut items) in by_rule {
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
            let meta = rule_meta(rule_id);
            write_rule_header(out, rule_id, sev, items.len(), color_on)?;
            if let Some(m) = meta {
                if !m.description.is_empty() {
                    writeln!(out, "   {}", dim(m.description, color_on))?;
                }
                if !m.fix_hint.is_empty() {
                    writeln!(
                        out,
                        "   {} {}",
                        bold("Fix:", color_on),
                        dim(m.fix_hint, color_on)
                    )?;
                }
            }

            // Sub-group by file. Preserve first-seen order so violations in the
            // same file stay contiguous and sorted by line.
            let mut by_file: Vec<(PathBuf, Vec<&Violation>)> = Vec::new();
            for v in items {
                match by_file.last_mut() {
                    Some((p, group)) if *p == v.file => group.push(v),
                    _ => by_file.push((v.file.clone(), vec![v])),
                }
            }

            for (file, group) in by_file {
                let rel = strip_prefix(&file, prefix.as_deref());
                let path_str = rel.display().to_string();
                let refs = format_file_refs(&group);
                if refs.is_empty() {
                    writeln!(out, "   {}", dim(&path_str, color_on))?;
                } else {
                    writeln!(out, "   {}  {}", dim(&path_str, color_on), refs,)?;
                }
            }
        }

        write_summary(out, violations, color_on)?;
        Ok(())
    }
}

fn write_rule_header(
    out: &mut dyn Write,
    id: RuleId,
    sev: Severity,
    count: usize,
    color_on: bool,
) -> Result<()> {
    let glyph = if color_on {
        match sev {
            Severity::Error => "\u{2a2f}".red().bold().to_string(),
            Severity::Warning => "\u{2a2f}".yellow().bold().to_string(),
            Severity::Info => "\u{2139}".cyan().to_string(),
        }
    } else {
        match sev {
            Severity::Error | Severity::Warning => "x".to_string(),
            Severity::Info => "i".to_string(),
        }
    };
    let code = id.code_str();
    let slug = id.slug;
    let sev_str = sev.as_str();
    let count_str = format!("{count} {}", pluralize(count, "finding", "findings"));
    if color_on {
        writeln!(
            out,
            "{} {}  {}  {} {} {}",
            glyph,
            code.bold(),
            slug.bold(),
            severity_colored(sev),
            "\u{00b7}".dimmed(),
            count_str.dimmed(),
        )?;
    } else {
        writeln!(
            out,
            "{} {}  {}  {} \u{00b7} {}",
            glyph, code, slug, sev_str, count_str,
        )?;
    }
    Ok(())
}

fn severity_colored(sev: Severity) -> String {
    match sev {
        Severity::Error => sev.as_str().red().bold().to_string(),
        Severity::Warning => sev.as_str().yellow().bold().to_string(),
        Severity::Info => sev.as_str().cyan().to_string(),
    }
}

fn write_summary(out: &mut dyn Write, violations: &[Violation], color_on: bool) -> Result<()> {
    let (mut errors, mut warnings, mut info) = (0usize, 0usize, 0usize);
    for v in violations {
        match v.severity {
            Severity::Error => errors += 1,
            Severity::Warning => warnings += 1,
            Severity::Info => info += 1,
        }
    }
    let total = violations.len();
    let glyph = if color_on { "\u{2a2f}" } else { "x" };
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

fn line_label(v: &Violation) -> Option<String> {
    match (v.line, v.column) {
        (Some(l), Some(c)) if c > 1 => Some(format!("L{l}:{c}")),
        (Some(l), _) => Some(format!("L{l}")),
        _ => None,
    }
}

/// Render every violation in a single file group as one line's worth of
/// references. Keeps output compact and avoids repeating rule-level text
/// on every row.
///
/// Rules:
/// * If nothing is known (no line, no detail), return "" — the file path
///   alone communicates the hit.
/// * If every violation has the same detail (or none), emit only line
///   labels like `L28, L29, L30`.
/// * Otherwise, pair each label with its detail: `L28 target1, L29 target2`.
fn format_file_refs(group: &[&Violation]) -> String {
    if group.is_empty() {
        return String::new();
    }
    let all_no_line = group.iter().all(|v| v.line.is_none());
    let all_no_detail = group.iter().all(|v| v.detail.is_none());
    if all_no_line && all_no_detail {
        return String::new();
    }
    let details_uniform = {
        let first = group[0].detail.as_deref();
        group.iter().all(|v| v.detail.as_deref() == first)
    };
    let mut parts: Vec<String> = Vec::with_capacity(group.len());
    if details_uniform {
        // Show line labels only; the (uniform) detail, if any, goes at the end.
        for v in group {
            if let Some(lbl) = line_label(v) {
                parts.push(lbl);
            }
        }
        let joined = parts.join(", ");
        match group[0].detail.as_deref() {
            Some(d) if !d.is_empty() && !joined.is_empty() => format!("{joined}  {d}"),
            Some(d) if !d.is_empty() => d.to_string(),
            _ => joined,
        }
    } else {
        for v in group {
            let label = line_label(v);
            let detail = v.detail.as_deref();
            match (label, detail) {
                (Some(l), Some(d)) => parts.push(format!("{l} {d}")),
                (Some(l), None) => parts.push(l),
                (None, Some(d)) => parts.push(d.to_string()),
                (None, None) => {}
            }
        }
        parts.join(", ")
    }
}

/// Longest shared leading path (component-wise) across all files. Returns
/// `None` when the set spans more than one root, or when every violation
/// is on a file in the current directory (nothing to strip).
fn common_path_prefix<'a, I>(paths: I) -> Option<PathBuf>
where
    I: IntoIterator<Item = &'a Path>,
{
    let mut iter = paths.into_iter();
    let first = iter.next()?;
    let mut prefix: Vec<Component<'a>> = first.parent()?.components().collect();
    for p in iter {
        let parent = p.parent()?;
        let mut new_len = 0;
        for (a, b) in prefix.iter().zip(parent.components()) {
            if a == &b {
                new_len += 1;
            } else {
                break;
            }
        }
        prefix.truncate(new_len);
        if prefix.is_empty() {
            return None;
        }
    }
    if prefix.is_empty() {
        None
    } else {
        Some(prefix.iter().collect())
    }
}

fn strip_prefix(path: &Path, prefix: Option<&Path>) -> PathBuf {
    match prefix {
        Some(p) => path.strip_prefix(p).unwrap_or(path).to_path_buf(),
        None => path.to_path_buf(),
    }
}

fn pluralize(n: usize, singular: &'static str, plural: &'static str) -> &'static str {
    if n == 1 {
        singular
    } else {
        plural
    }
}

fn dim(s: &str, color_on: bool) -> String {
    if color_on {
        s.dimmed().to_string()
    } else {
        s.to_string()
    }
}

fn bold(s: &str, color_on: bool) -> String {
    if color_on {
        s.bold().to_string()
    } else {
        s.to_string()
    }
}
