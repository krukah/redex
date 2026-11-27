use std::cell::RefCell;
use std::collections::HashSet;

use crate::*;

/// Names that are constants, never fresh variables: the primitives, plus every name a session has
/// defined. Per thread and only ever growing.
pub struct Consts;

impl Consts {
    pub fn has(name: &Name) -> bool {
        Self::with(|s| s.contains(name))
    }

    pub fn add(name: &Name) {
        Self::with(|s| s.insert(name.clone()));
    }

    fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&mut HashSet<Name>) -> R,
    {
        thread_local! {
            static SET: RefCell<HashSet<Name>> = RefCell::new(Prim::ALL.into_iter().map(Name::from).collect());
        }
        SET.with(|s| f(&mut s.borrow_mut()))
    }
}
