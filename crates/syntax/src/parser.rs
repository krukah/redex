use term::*;

use crate::*;

/// A recursive-descent parser over tokens. Error messages are user copy.
pub struct Parser {
    toks: Vec<Token>,
    k: usize,
}

impl Parser {
    pub fn parse(src: &str) -> Result<Term, Failure> {
        let mut p = Self {
            toks: Lexer::tokens(src)?,
            k: 0,
        };
        let t = p.term()?;
        match p.peek() {
            Some(tk) => Err(Failure::new(format!("Unexpected “{}” after the term.", tk.text()))),
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

    fn expect(&mut self, kind: &str) -> Result<Token, Failure> {
        match self.next() {
            Some(x) if x.kind() == kind => Ok(x),
            Some(x) => Err(Failure::new(format!("Expected “{kind}” before “{}”.", x.text()))),
            None => Err(Failure::new(format!("Expected “{kind}” at the end."))),
        }
    }

    fn term(&mut self) -> Result<Term, Failure> {
        match self.peek() {
            None => Err(Failure::from("The term ends too early.")),
            Some(Token::Lambda) => self.lambda(),
            Some(Token::Let) => self.local(),
            Some(_) => self.apply(),
        }
    }

    fn lambda(&mut self) -> Result<Term, Failure> {
        self.next();
        let mut xs = Vec::new();
        while let Some(Token::Id(x)) = self.peek().cloned() {
            self.next();
            xs.push(x);
        }
        if xs.is_empty() {
            return Err(Failure::from("Write a variable after λ."));
        }
        match self.next() {
            Some(Token::Punct('.') | Token::Arrow) => {
                Ok(xs.into_iter().rev().fold(self.term()?, |b, x| Term::lam(x, b)))
            }
            _ => Err(Failure::from("Write “.” or “=>” after the binders.")),
        }
    }

    fn local(&mut self) -> Result<Term, Failure> {
        self.next();
        let Token::Id(x) = self.expect("id")? else {
            unreachable!("expect checks the kind")
        };
        self.expect(":=")?;
        let v = self.term()?;
        match self.next() {
            Some(Token::Punct(';') | Token::In) => Ok(Term::local(x, v, self.term()?)),
            _ => Err(Failure::from("Write “;” after the let value.")),
        }
    }

    fn apply(&mut self) -> Result<Term, Failure> {
        let mut f = self.postfix()?;
        while self.peek().is_some_and(Token::starts_atom) {
            f = Term::app(f, self.postfix()?);
        }
        match self.peek() {
            Some(Token::Lambda | Token::Let) => Ok(Term::app(f, self.term()?)),
            _ => Ok(f),
        }
    }

    fn postfix(&mut self) -> Result<Term, Failure> {
        let mut a = self.atom()?;
        while let Some(Token::Proj(i)) = self.peek().cloned() {
            self.next();
            a = Term::proj(i, a);
        }
        Ok(a)
    }

    fn atom(&mut self) -> Result<Term, Failure> {
        match self.next() {
            None => Err(Failure::from("The term ends too early.")),
            Some(Token::Id(v)) => Ok(Term::var(v)),
            Some(Token::Num(v)) if v > Nat::MAX.get() as f64 => {
                Err(Failure::from("Keep numerals at 40 or below; each one is a chain of succ."))
            }
            Some(Token::Num(v)) => Ok(Term::numeral(Nat::new(v as usize))),
            Some(Token::Punct('(')) => self.tuple(),
            Some(tk) => Err(Failure::new(format!("Unexpected “{}”.", tk.text()))),
        }
    }

    fn tuple(&mut self) -> Result<Term, Failure> {
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
