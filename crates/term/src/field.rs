/// Which component of a pair a projection takes: `.1` or `.2`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Field {
    First,
    Second,
}

impl Field {
    pub fn parse(c: char) -> Option<Field> {
        match c {
            '1' => Some(Self::First),
            '2' => Some(Self::Second),
            _ => None,
        }
    }
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::First => "1",
            Self::Second => "2",
        })
    }
}
