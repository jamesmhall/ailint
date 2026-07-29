//! Security rules: prompt-injection surface, dangerous permissions, secrets.
//! Range: **AIL200 – AIL299**.
//!
//! TODO: implement concrete rules. Planned first pass:
//! - AIL200 `no-prompt-injection-marker` — content matching injection patterns
//! - AIL201 `no-unrestricted-tool-grant` — "you have access to all tools" style phrasing
//! - AIL202 `no-sensitive-data-in-instructions` — API keys / secrets in plain text

use crate::rules::RuleId;

pub const AIL200: RuleId = RuleId::new(200, "no-prompt-injection-marker");
pub const AIL201: RuleId = RuleId::new(201, "no-unrestricted-tool-grant");
pub const AIL202: RuleId = RuleId::new(202, "no-sensitive-data-in-instructions");
