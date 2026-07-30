//! Cross-file consistency rules.
//! Range: **AIL300 – AIL399**.

pub mod conflicting_rules;
pub mod duplicate_files;
pub mod orphaned_document;

pub use conflicting_rules::NoConflictingRulesRule;
pub use duplicate_files::NoDuplicateGuidanceFilesRule;
pub use orphaned_document::OrphanedDocumentRule;

use crate::rules::{BatchRule, RuleId};

/// AIL300: two guidance files give contradictory instructions.
pub const AIL300: RuleId = RuleId::new(300, "no-conflicting-rules");
/// AIL301: multiple guidance files duplicate the same content.
pub const AIL301: RuleId = RuleId::new(301, "no-duplicate-guidance-files");
/// AIL340: guidance file is never referenced by any other document.
pub const AIL340: RuleId = RuleId::new(340, "orphaned-document");

/// All cross-file batch rules, in registration order.
pub fn all_batch_rules() -> Vec<Box<dyn BatchRule>> {
    vec![
        Box::new(NoConflictingRulesRule),
        Box::new(NoDuplicateGuidanceFilesRule),
        Box::new(OrphanedDocumentRule),
    ]
}
