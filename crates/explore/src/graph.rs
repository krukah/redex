use std::collections::HashMap;
use std::collections::VecDeque;

use rules::*;
use term::*;

use crate::*;

/// Every term reachable from a start, explored breadth-first, one edge per redex. The start is
/// vertex 0.
#[derive(Debug, Clone)]
pub struct Graph {
    vertices: Vec<Vertex>,
    index: HashMap<String, usize>,
    edges: Vec<Edge>,
    truncated: bool,
}

impl Graph {
    /// Results larger than this are not followed.
    const LARGE: usize = 600;

    pub fn explore(start: &Term, env: &Env, eta: bool, cap: usize) -> Self {
        let root = Vertex::new(start.key(), start.clone(), 0, None, None);
        let mut g = Self {
            index: HashMap::from([(root.key().to_string(), 0)]),
            vertices: vec![root],
            edges: vec![],
            truncated: false,
        };
        let mut queue = VecDeque::from([0]);
        while let Some(i) = queue.pop_front() {
            g.expand(i, env, eta, cap, &mut queue);
        }
        g
    }

    fn expand(&mut self, i: usize, env: &Env, eta: bool, cap: usize, queue: &mut VecDeque<usize>) {
        let term = self.vertices[i].term().clone();
        let rs = term
            .redexes(env)
            .into_iter()
            .filter(|r| eta || !r.kind().is_eta())
            .collect::<Vec<Redex>>();
        self.vertices[i].set_nf(rs.is_empty());
        if rs.is_empty() {
            return;
        }
        if self.vertices.len() >= cap {
            self.vertices[i].set_open();
            self.truncated = true;
            return;
        }
        for r in rs {
            let next = term.step(&r, env);
            if next.size() > Self::LARGE {
                self.vertices[i].set_open();
                self.truncated = true;
                continue;
            }
            let key = next.key();
            let j = if let Some(&j) = self.index.get(&key) {
                j
            } else {
                let j = self.vertices.len();
                let depth = self.vertices[i].depth() + 1;
                self.index.insert(key.clone(), j);
                self.vertices
                    .push(Vertex::new(key, next, depth, Some(i), Some(r.kind())));
                queue.push_back(j);
                j
            };
            let edge = Edge::new(i, j, r.kind());
            if !self.edges.contains(&edge) {
                self.edges.push(edge);
            }
        }
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn truncated(&self) -> bool {
        self.truncated
    }

    pub fn root(&self) -> &Vertex {
        &self.vertices[0]
    }

    pub fn find(&self, key: &str) -> Option<usize> {
        self.index.get(key).copied()
    }

    /// Whether a reduction cycle is reachable from the start.
    pub fn cyclic(&self) -> bool {
        let out = self
            .edges
            .iter()
            .fold(vec![Vec::new(); self.vertices.len()], |mut out, e| {
                out[e.from()].push(e.to());
                out
            });
        Self::dfs(&out, &mut vec![0; self.vertices.len()], 0)
    }

    fn dfs(out: &[Vec<usize>], state: &mut Vec<u8>, k: usize) -> bool {
        state[k] = 1;
        let found = out[k].iter().any(|&to| match state[to] {
            1 => true,
            0 => Self::dfs(out, state, to),
            _ => false,
        });
        if !found {
            state[k] = 2;
        }
        found
    }
}
