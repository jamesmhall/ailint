//! Central registry of all built-in rules.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::config::Config;
use crate::parser::ParsedDocument;
use crate::rules::{consistency, security, semantic, structural};
use crate::rules::{BatchRule, Rule, RuleContext, RuleId, Severity, Violation};

/// Static metadata for a rule, cached for reporters to look up.
#[derive(Debug, Clone, Copy)]
pub struct RuleMeta {
    /// One-line human description of what the rule enforces.
    pub description: &'static str,
    /// One-line suggested remediation.
    pub fix_hint: &'static str,
    /// Default severity before any config overrides.
    pub default_severity: Severity,
}

/// Look up cached metadata for a rule by its ID. Returns `None` only for
/// rule IDs not present in the built-in registry.
pub fn rule_meta(id: RuleId) -> Option<RuleMeta> {
    rule_meta_cache().get(&id).copied()
}

fn rule_meta_cache() -> &'static HashMap<RuleId, RuleMeta> {
    static CACHE: OnceLock<HashMap<RuleId, RuleMeta>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut m = HashMap::new();
        for r in all_rules() {
            m.insert(
                r.id(),
                RuleMeta {
                    description: r.description(),
                    fix_hint: r.fix_hint(),
                    default_severity: r.default_severity(),
                },
            );
        }
        for r in all_batch_rules() {
            m.insert(
                r.id(),
                RuleMeta {
                    description: r.description(),
                    fix_hint: r.fix_hint(),
                    default_severity: r.default_severity(),
                },
            );
        }
        m
    })
}

/// Return the full set of built-in per-document rules.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    let mut out: Vec<Box<dyn Rule>> = Vec::new();
    out.extend(structural::all_rules());
    out.extend(semantic::all_rules());
    out.extend(security::all_rules());
    out
}

/// Return the full set of built-in batch (cross-file) rules.
pub fn all_batch_rules() -> Vec<Box<dyn BatchRule>> {
    consistency::all_batch_rules()
}

/// Run every enabled per-document rule against a document.
pub fn run_all(doc: &ParsedDocument, config: &Config) -> Vec<Violation> {
    let mut out = Vec::new();
    for rule in all_rules() {
        let id = rule.id();
        if is_disabled(&id, &config.rules.disabled) {
            continue;
        }
        if !rule.applies_to(doc.file_type) {
            continue;
        }
        let severity = effective_severity(
            &id,
            rule.default_severity(),
            &config.rules.severity_overrides,
        );
        let options = lookup_options(&id, &config.rules.options);
        let ctx = RuleContext {
            config,
            options,
            severity,
        };
        let mut vs = rule.run(doc, &ctx);
        for v in &mut vs {
            v.severity = severity;
        }
        out.extend(vs);
    }
    out
}

/// Run every enabled batch rule against the full corpus.
pub fn run_all_batch(docs: &[ParsedDocument], config: &Config) -> Vec<Violation> {
    let mut out = Vec::new();
    for rule in all_batch_rules() {
        let id = rule.id();
        if is_disabled(&id, &config.rules.disabled) {
            continue;
        }
        let severity = effective_severity(
            &id,
            rule.default_severity(),
            &config.rules.severity_overrides,
        );
        let options = lookup_options(&id, &config.rules.options);
        let ctx = RuleContext {
            config,
            options,
            severity,
        };
        let filtered: Vec<ParsedDocument> = docs
            .iter()
            .filter(|d| rule.applies_to(d.file_type))
            .cloned()
            .collect();
        let mut vs = rule.run_batch(&filtered, &ctx);
        for v in &mut vs {
            v.severity = severity;
        }
        out.extend(vs);
    }
    out
}

fn is_disabled(id: &RuleId, disabled: &[String]) -> bool {
    let code_str = id.code_str();
    disabled.iter().any(|d| d == &code_str || d == id.slug)
}

fn effective_severity(
    id: &RuleId,
    default: Severity,
    overrides: &std::collections::BTreeMap<String, Severity>,
) -> Severity {
    let code_str = id.code_str();
    overrides
        .get(&code_str)
        .or_else(|| overrides.get(id.slug))
        .copied()
        .unwrap_or(default)
}

fn lookup_options<'a>(
    id: &RuleId,
    options: &'a std::collections::BTreeMap<String, serde_yaml::Value>,
) -> Option<&'a serde_yaml::Value> {
    options.get(id.slug).or_else(|| options.get(&id.code_str()))
}
