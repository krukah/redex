use std::cell::RefCell;
use std::collections::HashSet;

/// Names that are constants, never fresh variables: the primitives, plus every name a session has
/// defined. Per thread and only ever growing, like the JS engine's `CONSTS`.
pub struct Consts;

impl Consts {
    pub const PRIMS: [&'static str; 3] = ["zero", "succ", "Nat.rec"];

    pub fn has(name: &str) -> bool {
        Self::with(|s| s.contains(name))
    }

    pub fn add(name: &str) {
        Self::with(|s| s.insert(name.to_string()));
    }

    pub fn prim(name: &str) -> bool {
        Self::PRIMS.contains(&name)
    }

    fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&mut HashSet<String>) -> R,
    {
        thread_local! {
            static SET: RefCell<HashSet<String>> = RefCell::new(Consts::PRIMS.iter().map(ToString::to_string).collect());
        }
        SET.with(|s| f(&mut s.borrow_mut()))
    }
}
