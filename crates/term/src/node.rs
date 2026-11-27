use std::cell::Cell;

use crate::*;

/// One node. The shape is fixed; its origin, flash and capture flag are decorations the workbench strips
/// after it first draws them, and history entries share subtrees, so they live in cells.
#[derive(Debug)]
pub(crate) struct Node {
    uid: Uid,
    shape: Shape,
    origin: Cell<Option<Uid>>,
    flash: Cell<Option<Mark>>,
    caught: Cell<bool>,
}

impl Node {
    pub fn new(uid: Uid, shape: Shape) -> Self {
        Self {
            uid,
            shape,
            origin: Cell::new(None),
            flash: Cell::new(None),
            caught: Cell::new(false),
        }
    }

    pub fn uid(&self) -> Uid {
        self.uid
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    fn origin(&self) -> &Cell<Option<Uid>> {
        &self.origin
    }

    fn flash(&self) -> &Cell<Option<Mark>> {
        &self.flash
    }

    fn caught(&self) -> &Cell<bool> {
        &self.caught
    }

    pub fn from(&self) -> Option<Uid> {
        self.origin().get()
    }

    pub fn mark(&self) -> Option<Mark> {
        self.flash().get()
    }

    pub fn cap(&self) -> bool {
        self.caught().get()
    }

    pub fn set_from(&self, uid: Option<Uid>) {
        self.origin().set(uid);
    }

    pub fn set_mark(&self, mark: Option<Mark>) {
        self.flash().set(mark);
    }

    pub fn set_cap(&self, cap: bool) {
        self.caught().set(cap);
    }
}
