use crate::*;
use term::*;

/// Turns source text into tokens.
pub struct Lexer(Vec<char>);

impl Lexer {
    pub fn tokens(src: &str) -> Result<Vec<Token>, Failure> {
        let lexer = Self(src.chars().collect());
        let mut toks = Vec::new();
        let mut i = 0;
        while i < lexer.0.len() {
            let (tok, next) = lexer.scan(i)?;
            toks.extend(tok);
            i = next;
        }
        Ok(toks)
    }

    fn at(&self, i: usize) -> Option<char> {
        self.0.get(i).copied()
    }

    fn starts(&self, i: usize, s: &str) -> bool {
        s.chars().enumerate().all(|(j, c)| self.at(i + j) == Some(c))
    }

    fn digit(c: Option<char>) -> bool {
        c.as_ref().is_some_and(char::is_ascii_digit)
    }

    fn id_start(c: char) -> bool {
        (c.is_ascii_alphabetic() || c == '_' || ('\u{370}'..='\u{3ff}').contains(&c)) && c != 'λ'
    }

    fn id_char(c: char) -> bool {
        Self::id_start(c) || c.is_ascii_digit() || matches!(c, '\'' | '!' | '?') || term::Name::sub(c)
    }

    /// The token at `i`, if any, and where the next one starts.
    fn scan(&self, i: usize) -> Result<(Option<Token>, usize), Failure> {
        let c = self.0[i];
        match c {
            _ if Line::space(c) => Ok((None, i + 1)),
            _ if self.starts(i, "--") => {
                Ok((None, (i..self.0.len()).find(|&j| self.0[j] == '\n').unwrap_or(self.0.len())))
            }
            _ if self.starts(i, ":=") => Ok((Some(Token::Assign), i + 2)),
            _ if self.starts(i, "=>") => Ok((Some(Token::Arrow), i + 2)),
            '↦' => Ok((Some(Token::Arrow), i + 1)),
            'λ' | '\\' => Ok((Some(Token::Lambda), i + 1)),
            '.' => Ok(self.dot(i)),
            '(' | ')' | ',' | ';' => Ok((Some(Token::Punct(c)), i + 1)),
            _ if c.is_ascii_digit() => Ok(self.number(i)),
            _ if Self::id_start(c) => Ok(self.word(i)),
            _ => Err(Failure::new(format!("Unexpected character “{c}”."))),
        }
    }

    /// `.1` or `.2` not followed by a digit is a projection; any other `.` is punctuation.
    fn dot(&self, i: usize) -> (Option<Token>, usize) {
        match (self.at(i + 1).and_then(Field::parse), Self::digit(self.at(i + 2))) {
            (Some(f), false) => (Some(Token::Proj(f)), i + 2),
            _ => (Some(Token::Punct('.')), i + 1),
        }
    }

    fn number(&self, i: usize) -> (Option<Token>, usize) {
        let j = (i..self.0.len())
            .find(|&j| !self.0[j].is_ascii_digit())
            .unwrap_or(self.0.len());
        (
            Some(Token::Num(
                self.0[i..j]
                    .iter()
                    .collect::<String>()
                    .parse::<f64>()
                    .unwrap_or(f64::INFINITY),
            )),
            j,
        )
    }

    fn word(&self, i: usize) -> (Option<Token>, usize) {
        let j = (i + 1..self.0.len())
            .find(|&j| {
                !Self::id_char(self.0[j])
                    && !(self.0[i].is_ascii_uppercase()
                        && self.0[j] == '.'
                        && self.at(j + 1).is_some_and(Self::id_start))
            })
            .unwrap_or(self.0.len());
        let w = self.0[i..j].iter().collect::<String>();
        (
            Some(match w.as_str() {
                "fun" => Token::Lambda,
                "let" => Token::Let,
                "in" => Token::In,
                _ => Token::Id(Name::from(w.as_str())),
            }),
            j,
        )
    }
}
