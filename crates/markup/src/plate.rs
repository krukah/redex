/// A graph drawing's page-wide number, so its arrowhead ids (`rdx1-beta`) are unique in the
/// document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plate(u64);

impl Plate {
    pub fn next() -> Self {
        thread_local! {
            static NEXT: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
        }
        NEXT.with(|n| {
            n.set(n.get() + 1);
            Self(n.get())
        })
    }
}

impl std::fmt::Display for Plate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rdx{}", self.0)
    }
}
