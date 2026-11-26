use term::*;

use crate::*;

/// A strategy run from a start term: every term it passed through, and how it ended.
#[derive(Debug, Clone)]
pub struct Run {
    rows: Vec<Row>,
    status: Status,
}

impl Run {
    const BIG: usize = 1500;

    pub fn new(start: &Term, env: &Env, strategy: Strategy, cap: usize) -> Self {
        let mut rows = vec![Row::new(start.clone(), None)];
        let mut t = start.clone();
        for _ in 0..cap {
            let Some(r) = strategy.next(&t, env) else {
                return Self {
                    rows,
                    status: Status::Done,
                };
            };
            t = rules::Reduce::step(&t, &r, env);
            rows.push(Row::new(t.clone(), Some(r.kind())));
            if t.size() > Self::BIG {
                return Self {
                    rows,
                    status: Status::Big,
                };
            }
        }
        let status = if strategy.next(&t, env).is_some() { Status::Cap } else { Status::Done };
        Self { rows, status }
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn steps(&self) -> usize {
        self.rows.len() - 1
    }

    pub fn last(&self) -> &Term {
        self.rows.last().expect("a run holds its start").term()
    }
}
