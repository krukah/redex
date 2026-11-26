use std::collections::HashSet;

use crate::*;

/// Variable names: subscripts and fresh names.
pub struct Name;

impl Name {
    const SUBS: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];

    pub fn sub(c: char) -> bool {
        ('₀'..='₉').contains(&c)
    }

    /// The name without its trailing subscript digits.
    pub fn strip(name: &str) -> &str {
        name.trim_end_matches(Self::sub)
    }

    /// The first `root₁`, `root₂`, … that is neither in `avoid` nor a constant.
    pub fn fresh(base: &str, avoid: &HashSet<String>) -> String {
        (1..=u32::MAX)
            .map(|i| {
                format!(
                    "{}{}",
                    Self::strip(base),
                    i.to_string()
                        .chars()
                        .map(|d| Self::SUBS[d as usize - '0' as usize])
                        .collect::<String>()
                )
            })
            .find(|n| !avoid.contains(n) && !Consts::has(n))
            .expect("names are unbounded")
    }
}
