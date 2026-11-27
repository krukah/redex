use std::cell::Cell;

use crate::*;

/// One node. The shape is fixed; `from`, `mark` and `cap` are decorations the workbench strips
/// after it first draws them, and history entries share subtrees, so they live in cells.
#[derive(Debug)]
pub(crate) struct Node {
    uid: Uid,
    shape: Shape,
    from: Cell<Option<Uid>>,
    mark: Cell<Option<Mark>>,
    cap: Cell<bool>,
}

impl Node {
    pub fn new(uid: Uid, shape: Shape) -> Self {
        Self {
            uid,
            shape,
            from: Cell::new(None),
            mark: Cell::new(None),
            cap: Cell::new(false),
        }
    }

    pub fn uid(&self) -> Uid {
        self.uid
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    pub fn from(&self) -> Option<Uid> {
        self.from.get()
    }

    pub fn mark(&self) -> Option<Mark> {
        self.mark.get()
    }

    pub fn cap(&self) -> bool {
        self.cap.get()
    }

    pub fn set_from(&self, uid: Option<Uid>) {
        self.from.set(uid);
    }

    pub fn set_mark(&self, mark: Option<Mark>) {
        self.mark.set(mark);
    }

    pub fn set_cap(&self, cap: bool) {
        self.cap.set(cap);
    }
}
