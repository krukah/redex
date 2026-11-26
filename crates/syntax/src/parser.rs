use term::*;

use crate::*;

/// A recursive-descent parser over tokens. Error messages are user copy.
pub struct Parser {
    toks: Vec<Token>,
    k: usize,
}

impl Parser {
    pub fn parse(src: &str) -> Result<Term, String> {
        let mut p = Self {
            toks: Lexer::tokens(src)?,
            k: 0,
        };
        let t = p.term()?;
        match p.peek() {
            Some(tk) => Err(format!("Unexpected “{}” after the term.", tk.text())),
            None => Ok(t),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.toks.get(self.k)
    }

    fn next(&mut self) -> Option<Token> {
        let tk = self.toks.get(self.k).cloned();
        self.k += 1;
        tk
    }

    fn expect(&mut self, kind: &str) -> Result<Token, String> {
        match self.next() {
            Some(x) if x.kind() == kind => Ok(x),
            Some(x) => Err(format!("Expected “{kind}” before “{}”.", x.text())),
            None => Err(format!("Expected “{kind}” at the end.")),
        }
    }

    fn term(&mut self) -> Result<Term, String> {
        match self.peek() {
            None => Err("The term ends too early.".to_string()),
            Some(Token::Lambda) => self.lambda(),
            Some(Token::Let) => self.local(),
            Some(_) => self.apply(),
        }
    }

    fn lambda(&mut self) -> Result<Term, String> {
        self.next();
        let mut xs = Vec::new();
        while let Some(Token::Id(x)) = self.peek().cloned() {
            self.next();
            xs.push(x);
        }
        if xs.is_empty() {
            return Err("Write a variable after λ.".to_string());
        }
        match self.next() {
            Some(Token::Punct('.') | Token::Arrow) => Ok(xs
                .iter()
                .rev()
                .try_fold(self.term()?, |b, x| Ok::<Term, String>(Term::lam(x, b)))?),
            _ => Err("Write “.” or “=>” after the binders.".to_string()),
        }
    }

    fn local(&mut self) -> Result<Term, String> {
        self.next();
        let Token::Id(x) = self.expect("id")? else {
            unreachable!("expect checks the kind")
        };
        self.expect(":=")?;
        let v = self.term()?;
        match self.next() {
            Some(Token::Punct(';') | Token::In) => Ok(Term::local(&x, v, self.term()?)),
            _ => Err("Write “;” after the let value.".to_string()),
        }
    }

    fn apply(&mut self) -> Result<Term, String> {
        let mut f = self.postfix()?;
        while self.peek().is_some_and(Token::starts_atom) {
            f = Term::app(f, self.postfix()?);
        }
        match self.peek() {
            Some(Token::Lambda | Token::Let) => Ok(Term::app(f, self.term()?)),
            _ => Ok(f),
        }
    }

    fn postfix(&mut self) -> Result<Term, String> {
        let mut a = self.atom()?;
        while let Some(Token::Proj(i)) = self.peek().cloned() {
            self.next();
            a = Term::proj(i, a);
        }
        Ok(a)
    }

    fn atom(&mut self) -> Result<Term, String> {
        match self.next() {
            None => Err("The term ends too early.".to_string()),
            Some(Token::Id(v)) => Ok(Term::var(&v)),
            Some(Token::Num(v)) if v > 40.0 => {
                Err("Keep numerals at 40 or below; each one is a chain of succ.".to_string())
            }
            Some(Token::Num(v)) => {
                Ok((0..v as usize).fold(Term::cons("zero"), |t, _| Term::app(Term::cons("succ"), t)))
            }
            Some(Token::Punct('(')) => self.tuple(),
            Some(tk) => Err(format!("Unexpected “{}”.", tk.text())),
        }
    }

    fn tuple(&mut self) -> Result<Term, String> {
        let mut items = vec![self.term()?];
        while self.peek() == Some(&Token::Punct(',')) {
            self.next();
            items.push(self.term()?);
        }
        self.expect(")")?;
        let last = items.pop().expect("a tuple holds a term");
        Ok(items.into_iter().rev().fold(last, |t, a| Term::pair(a, t)))
    }
}
