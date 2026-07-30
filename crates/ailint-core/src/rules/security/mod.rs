//! Security rules: prompt-injection surface, dangerous permissions, secrets.
//! Range: **AIL200 – AIL299**.

pub mod prompt_injection;
pub mod sensitive_data;
pub mod tool_confirmation;
pub mod unrestricted_tool;

pub use prompt_injection::NoPromptInjectionMarkerRule;
pub use sensitive_data::NoSensitiveDataInInstructionsRule;
pub use tool_confirmation::ToolConfirmationRequiredRule;
pub use unrestricted_tool::NoUnrestrictedToolGrantRule;

use crate::rules::{Rule, RuleId};

/// AIL200: text contains a known prompt-injection marker.
pub const AIL200: RuleId = RuleId::new(200, "no-prompt-injection-marker");
/// AIL201: guidance grants a tool unrestricted or auto-approved access.
pub const AIL201: RuleId = RuleId::new(201, "no-unrestricted-tool-grant");
/// AIL202: instructions embed secrets or other sensitive data.
pub const AIL202: RuleId = RuleId::new(202, "no-sensitive-data-in-instructions");
/// AIL203: destructive actions described without a confirmation step.
pub const AIL203: RuleId = RuleId::new(203, "tool-confirmation-required");

/// All security rules, in registration order.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(NoPromptInjectionMarkerRule),
        Box::new(NoUnrestrictedToolGrantRule),
        Box::new(NoSensitiveDataInInstructionsRule),
        Box::new(ToolConfirmationRequiredRule),
    ]
}

// 1-based line number containing the byte at `offset` in `raw`.
pub(crate) fn line_of_offset(raw: &str, offset: usize) -> usize {
    let cap = offset.min(raw.len());
    raw.as_bytes()[..cap]
        .iter()
        .filter(|&&b| b == b'\n')
        .count()
        + 1
}

// Trimmed content of the line containing byte `offset`.
pub(crate) fn line_containing(raw: &str, offset: usize) -> String {
    let bytes = raw.as_bytes();
    let cap = offset.min(bytes.len());
    let start = bytes[..cap]
        .iter()
        .rposition(|&b| b == b'\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    let end = bytes[cap..]
        .iter()
        .position(|&b| b == b'\n')
        .map(|i| cap + i)
        .unwrap_or(bytes.len());
    raw[start..end].trim().to_string()
}

// Char-safe truncation to at most `max` characters.
pub(crate) fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    s.chars().take(max).collect()
}
