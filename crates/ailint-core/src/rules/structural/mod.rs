//! Structural rules: schema / frontmatter / required-section validation.
//! Range: **AIL001 – AIL099**.
//!
//! TODO: implement concrete rules. Planned first pass:
//! - AIL001 `no-frontmatter-schema-error` — invalid YAML frontmatter
//! - AIL002 `instructions-file-empty` — file is empty or whitespace only
//! - AIL003 `missing-required-section` — required heading missing

use crate::rules::RuleId;

pub const AIL001: RuleId = RuleId::new(1, "no-frontmatter-schema-error");
pub const AIL002: RuleId = RuleId::new(2, "instructions-file-empty");
pub const AIL003: RuleId = RuleId::new(3, "missing-required-section");
