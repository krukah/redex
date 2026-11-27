use crate::*;

/// A rule's schema, with pattern variables written `{x}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema(&'static str);

impl Schema {
    pub fn text(&self) -> &'static str {
        self.0
    }

    /// Pattern variables in order of first appearance.
    pub fn vars(&self) -> Vec<Var> {
        self.parts()
            .into_iter()
            .filter_map(|p| if let Part::Var(v) = p { Some(v) } else { None })
            .fold(Vec::new(), |mut vs, v| {
                if !vs.contains(&v) {
                    vs.push(v);
                }
                vs
            })
    }

    /// A variable's highlighter: its position among `vars`.
    pub fn slot(&self, var: Var) -> Option<Slot> {
        self.vars().iter().position(|v| *v == var).map(Slot::at)
    }

    /// The schema cut into literal text and pattern variables, in order. A variable is one word
    /// character between braces, as the JS `\{(\w)\}` reads it.
    pub fn parts(&self) -> Vec<Part> {
        let mut out = Vec::new();
        let mut rest = self.0;
        while let Some(i) = rest.find('{') {
            match rest[i + 1..].char_indices().nth(1) {
                Some((j, '}'))
                    if rest[i + 1..i + 1 + j]
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_') =>
                {
                    out.push(Part::Text(&rest[..i]));
                    out.push(Part::Var(Var::new(&rest[i + 1..i + 1 + j])));
                    rest = &rest[i + 2 + j..];
                }
                _ => {
                    out.push(Part::Text(&rest[..=i]));
                    rest = &rest[i + 1..];
                }
            }
        }
        out.push(Part::Text(rest));
        out.into_iter().filter(|p| *p != Part::Text("")).collect()
    }
}

impl From<&'static str> for Schema {
    fn from(s: &'static str) -> Self {
        Self(s)
    }
}
