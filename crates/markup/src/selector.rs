use crate::*;

/// A CSS selector, built from the same classes, tags and data attributes the markup uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    Class(Class),
    Tag(Tag),
    /// `[data-x]`.
    Has(Data),
    /// `[data-x="v"]`.
    Is(Data, String),
    /// `:scope`, the element the query runs on.
    Scope,
    /// Both at once: `a.b`.
    And(Box<Selector>, Box<Selector>),
    /// `a > b`.
    Child(Box<Selector>, Box<Selector>),
    /// `a b`.
    Within(Box<Selector>, Box<Selector>),
    /// `a, b`.
    Or(Box<Selector>, Box<Selector>),
}

impl Selector {
    pub fn is<T>(data: Data, value: &T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Is(data, value.to_string())
    }

    pub fn and(self, other: Selector) -> Self {
        Self::And(Box::new(self), Box::new(other))
    }

    pub fn child(self, other: Selector) -> Self {
        Self::Child(Box::new(self), Box::new(other))
    }

    pub fn within(self, other: Selector) -> Self {
        Self::Within(Box::new(self), Box::new(other))
    }

    pub fn or(self, other: Selector) -> Self {
        Self::Or(Box::new(self), Box::new(other))
    }
}

impl From<Class> for Selector {
    fn from(c: Class) -> Self {
        Self::Class(c)
    }
}

impl From<Tag> for Selector {
    fn from(t: Tag) -> Self {
        Self::Tag(t)
    }
}

impl std::fmt::Display for Selector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Class(c) => write!(f, ".{c}"),
            Self::Tag(t) => f.write_str(t.name()),
            Self::Has(d) => write!(f, "[{}]", d.name()),
            Self::Is(d, v) => write!(f, "[{}=\"{}\"]", d.name(), v.replace('\\', "\\\\").replace('"', "\\\"")),
            Self::Scope => f.write_str(":scope"),
            Self::And(a, b) => write!(f, "{a}{b}"),
            Self::Child(a, b) => write!(f, "{a} > {b}"),
            Self::Within(a, b) => write!(f, "{a} {b}"),
            Self::Or(a, b) => write!(f, "{a}, {b}"),
        }
    }
}
