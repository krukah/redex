use crate::*;

/// A piece of markup: an element with its attributes (in writing order) and children, a run of
/// text, or a list of siblings. Built by composition; rendered by `Display`, which escapes every
/// text and attribute value and is the only place markup becomes a string.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    El(Tag, Vec<Attr>, Vec<Node>),
    Text(String),
    List(Vec<Node>),
}

impl Node {
    pub fn el(tag: Tag) -> Self {
        Self::El(tag, Vec::new(), Vec::new())
    }

    pub fn text<T>(t: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Text(t.to_string())
    }

    pub fn list<I>(nodes: I) -> Self
    where
        I: IntoIterator<Item = Node>,
    {
        Self::List(nodes.into_iter().collect())
    }

    pub fn none() -> Self {
        Self::List(Vec::new())
    }

    pub fn attr(self, a: Attr) -> Self {
        match self {
            Self::El(tag, attrs, kids) => Self::El(tag, attrs.into_iter().chain([a]).collect(), kids),
            other => other,
        }
    }

    pub fn class<I>(self, classes: I) -> Self
    where
        I: IntoIterator<Item = Class>,
    {
        self.attr(Attr::Class(classes.into_iter().collect()))
    }

    pub fn kid(self, n: Node) -> Self {
        match self {
            Self::El(tag, attrs, kids) => Self::El(tag, attrs, kids.into_iter().chain([n]).collect()),
            Self::List(nodes) => Self::List(nodes.into_iter().chain([n]).collect()),
            text => Self::List(vec![text, n]),
        }
    }

    pub fn kids<I>(self, nodes: I) -> Self
    where
        I: IntoIterator<Item = Node>,
    {
        nodes.into_iter().fold(self, Self::kid)
    }

    /// Writes `s` with `&`, `<`, `>` and `"` escaped: the one place anything is escaped.
    fn escape(s: &str, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        s.chars().try_for_each(|c| match c {
            '&' => f.write_str("&amp;"),
            '<' => f.write_str("&lt;"),
            '>' => f.write_str("&gt;"),
            '"' => f.write_str("&quot;"),
            _ => write!(f, "{c}"),
        })
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(s) => Self::escape(s, f),
            Self::List(nodes) => nodes.iter().try_for_each(|n| write!(f, "{n}")),
            Self::El(tag, attrs, kids) => {
                write!(f, "<{}", tag.name())?;
                attrs.iter().try_for_each(|a| {
                    write!(f, " {}=\"", a.name())?;
                    Self::escape(&a.to_string(), f)?;
                    f.write_str("\"")
                })?;
                if tag.svg() && kids.is_empty() {
                    return f.write_str("/>");
                }
                f.write_str(">")?;
                kids.iter().try_for_each(|n| write!(f, "{n}"))?;
                write!(f, "</{}>", tag.name())
            }
        }
    }
}
