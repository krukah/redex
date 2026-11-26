use term::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Lambda,
    Let,
    In,
    Assign,
    Arrow,
    Proj(u8),
    Punct(char),
    Num(f64),
    Id(String),
}

impl Token {
    /// The token's type as parser errors name it.
    pub fn kind(&self) -> String {
        match self {
            Self::Lambda => "λ".to_string(),
            Self::Let => "let".to_string(),
            Self::In => "in".to_string(),
            Self::Assign => ":=".to_string(),
            Self::Arrow => "=>".to_string(),
            Self::Proj(_) => "proj".to_string(),
            Self::Punct(c) => c.to_string(),
            Self::Num(_) => "num".to_string(),
            Self::Id(_) => "id".to_string(),
        }
    }

    /// The token as parser errors quote it.
    pub fn text(&self) -> String {
        match self {
            Self::Id(v) => v.clone(),
            Self::Num(v) => Number::show(*v),
            Self::Proj(i) => format!(".{i}"),
            _ => self.kind(),
        }
    }

    pub fn starts_atom(&self) -> bool {
        matches!(self, Self::Id(_) | Self::Num(_) | Self::Punct('('))
    }
}
