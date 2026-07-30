//! Semantic rules: instruction quality, clarity, redundancy.
//! Range: **AIL100 – AIL199**.

pub mod duplicate_rules;
pub mod excessive_length;
pub mod missing_examples;
pub mod negative_constraint_overload;
pub mod vague_instruction;
pub mod vendor_optimization;

pub use duplicate_rules::NoDuplicateRulesRule;
pub use excessive_length::ExcessiveRuleLengthRule;
pub use missing_examples::NoMissingExamplesRule;
pub use negative_constraint_overload::NegativeConstraintOverloadRule;
pub use vague_instruction::NoVagueInstructionRule;
pub use vendor_optimization::VendorOptimizationSyntaxRule;

use crate::rules::{Rule, RuleId};

/// AIL100: instruction uses vague, unactionable phrasing.
pub const AIL100: RuleId = RuleId::new(100, "no-vague-instruction");
/// AIL101: rules reference behavior without concrete examples.
pub const AIL101: RuleId = RuleId::new(101, "no-missing-examples");
/// AIL102: document exceeds the practical context-length budget.
pub const AIL102: RuleId = RuleId::new(102, "excessive-rule-length");
/// AIL103: the same rule is stated more than once in a document.
pub const AIL103: RuleId = RuleId::new(103, "no-duplicate-rules");
/// AIL104: instruction list dominated by negative ("do not") constraints.
pub const AIL104: RuleId = RuleId::new(104, "negative-constraint-overload");
/// AIL105: Claude/Cline guidance without the XML tags those tools favor.
pub const AIL105: RuleId = RuleId::new(105, "vendor-optimization-syntax");

/// All semantic rules, in registration order.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(NoVagueInstructionRule),
        Box::new(NoMissingExamplesRule),
        Box::new(ExcessiveRuleLengthRule),
        Box::new(NoDuplicateRulesRule),
        Box::new(NegativeConstraintOverloadRule),
        Box::new(VendorOptimizationSyntaxRule),
    ]
}
