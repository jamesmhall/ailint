//! Cross-file consistency rules.
//! Range: **AIL300 – AIL399**.
//!
//! These rules operate on the full corpus of discovered files, not a single
//! document. TODO: introduce a `BatchRule` trait separate from `Rule`.
//!
//! Planned first pass:
//! - AIL300 `no-conflicting-rules` — contradictory instructions across files
//! - AIL301 `no-duplicate-guidance-files` — same guidance in multiple files

use crate::rules::RuleId;

pub const AIL300: RuleId = RuleId::new(300, "no-conflicting-rules");
pub const AIL301: RuleId = RuleId::new(301, "no-duplicate-guidance-files");
