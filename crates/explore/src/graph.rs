use std::collections::HashMap;
use std::collections::VecDeque;

use rules::*;
use strategy::*;
use term::*;

use crate::*;

/// Every term reachable from a start, explored breadth-first, one edge per redex.
#[derive(Debug, Clone)]
pub struct Graph {
    vertices: Vec<Vertex>,
    index: HashMap<Key, Id>,
    edges: Vec<Edge>,
    truncated: bool,
}

impl Graph {
    /// Results larger than this are not followed.
    const LARGE: Size = Size::new(600);

    /// Explores from `start`, following η redexes only when `eta`, and expanding no vertex once
    /// `cap` vertices are known (a vertex that would expand is marked open).
    pub fn explore(start: &Term, env: &Env, eta: bool, cap: Cap) -> Self {
        let root = Vertex::new(start.key(), start.clone(), Depth::START, None, None);
        let mut g = Self {
            index: HashMap::from([(root.key().clone(), Id::ROOT)]),
            vertices: vec![root],
            edges: vec![],
            truncated: false,
        };
        let mut queue = VecDeque::from([Id::ROOT]);
        while let Some(i) = queue.pop_front() {
            g.expand(i, env, eta, cap, &mut queue);
        }
        g
    }

    fn expand(&mut self, i: Id, env: &Env, eta: bool, cap: Cap, queue: &mut VecDeque<Id>) {
        let term = self.vertex(i).term().clone();
        let rs = term
            .redexes(env)
            .into_iter()
            .filter(|r| eta || !r.kind().is_eta())
            .collect::<Vec<Redex>>();
        self.vertices[i.get()].set_nf(rs.is_empty());
        if rs.is_empty() {
            return;
        }
        if self.vertices.len() >= cap.get() {
            self.vertices[i.get()].set_open();
            self.truncated = true;
            return;
        }
        for r in rs {
            let next = term.step(&r, env);
            if next.size() > Self::LARGE {
                self.vertices[i.get()].set_open();
                self.truncated = true;
                continue;
            }
            let key = next.key();
            let j = self
                .index
                .get(&key)
                .copied()
                .unwrap_or_else(|| self.admit(key, next, i, r.kind(), queue));
            let edge = Edge::new(i, j, r.kind());
            if !self.edges.contains(&edge) {
                self.edges.push(edge);
            }
        }
    }

    /// Adds a newly reached term one row below `from`, and queues it.
    fn admit(&mut self, key: Key, term: Term, from: Id, kind: Kind, queue: &mut VecDeque<Id>) -> Id {
        let j = Id::from(self.vertices.len());
        let depth = self.vertex(from).depth().next();
        self.index.insert(key.clone(), j);
        self.vertices
            .push(Vertex::new(key, term, depth, Some(from), Some(kind)));
        queue.push_back(j);
        j
    }

    pub fn vertex(&self, id: Id) -> &Vertex {
        &self.vertices[id.get()]
    }

    /// Every vertex with its id, in exploration order.
    pub fn vertices(&self) -> impl Iterator<Item = (Id, &Vertex)> {
        self.vertices.iter().enumerate().map(|(i, v)| (Id::from(i), v))
    }

    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    pub fn truncated(&self) -> bool {
        self.truncated
    }

    pub fn root(&self) -> &Vertex {
        self.vertex(Id::ROOT)
    }

    pub fn find(&self, key: &Key) -> Option<Id> {
        self.index.get(key).copied()
    }

    /// The vertices with no redex left.
    pub fn normal(&self) -> usize {
        self.vertices.iter().map(Vertex::nf).filter(|nf| *nf).count()
    }

    /// Whether a reduction cycle is reachable from the start.
    pub fn cyclic(&self) -> bool {
        let out = self
            .edges
            .iter()
            .fold(vec![Vec::new(); self.vertices.len()], |mut out, e| {
                out[e.from().get()].push(e.to());
                out
            });
        Self::dfs(&out, &mut vec![0; self.vertices.len()], Id::ROOT)
    }

    fn dfs(out: &[Vec<Id>], state: &mut Vec<u8>, k: Id) -> bool {
        state[k.get()] = 1;
        let found = out[k.get()].iter().any(|&to| match state[to.get()] {
            1 => true,
            0 => Self::dfs(out, state, to),
            _ => false,
        });
        if !found {
            state[k.get()] = 2;
        }
        found
    }
}
