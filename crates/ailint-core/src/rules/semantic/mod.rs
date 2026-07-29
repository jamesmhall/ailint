//! Semantic rules: instruction quality, clarity, redundancy.
//! Range: **AIL100 – AIL199**.
//!
//! TODO: implement concrete rules. Planned first pass:
//! - AIL100 `no-vague-instruction` — flags generic phrases ("be helpful", "do your best")
//! - AIL101 `no-missing-examples` — rules without concrete examples
//! - AIL102 `excessive-rule-length` — single rule exceeds N words (configurable)
//! - AIL103 `no-duplicate-rules` — near-identical rules within same file

use crate::rules::RuleId;

pub const AIL100: RuleId = RuleId::new(100, "no-vague-instruction");
pub const AIL101: RuleId = RuleId::new(101, "no-missing-examples");
pub const AIL102: RuleId = RuleId::new(102, "excessive-rule-length");
pub const AIL103: RuleId = RuleId::new(103, "no-duplicate-rules");
