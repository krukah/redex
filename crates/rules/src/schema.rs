/// A rule's schema, with pattern variables written `{x}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema(&'static str);

impl Schema {
    pub fn text(&self) -> &'static str {
        self.0
    }

    /// Pattern variables in order of first appearance; a variable's highlighter slot is its index.
    pub fn vars(&self) -> Vec<&'static str> {
        self.parts()
            .into_iter()
            .filter_map(|(var, s)| var.then_some(s))
            .fold(Vec::new(), |mut vs, v| {
                if !vs.contains(&v) {
                    vs.push(v);
                }
                vs
            })
    }

    /// The schema cut into literal text and pattern variables (`true`), in order.
    pub fn parts(&self) -> Vec<(bool, &'static str)> {
        let mut out = Vec::new();
        let mut rest = self.0;
        while let Some(i) = rest.find('{') {
            match rest[i + 1..].char_indices().nth(1) {
                Some((j, '}'))
                    if rest[i + 1..i + 1 + j]
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_') =>
                {
                    out.push((false, &rest[..i]));
                    out.push((true, &rest[i + 1..i + 1 + j]));
                    rest = &rest[i + 2 + j..];
                }
                _ => {
                    out.push((false, &rest[..=i]));
                    rest = &rest[i + 1..];
                }
            }
        }
        out.push((false, rest));
        out.into_iter().filter(|(var, s)| *var || !s.is_empty()).collect()
    }
}

impl From<&'static str> for Schema {
    fn from(s: &'static str) -> Self {
        Self(s)
    }
}
