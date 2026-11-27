/// A start term for the strategy race. `bench` has no source: it races whatever the workbench
/// started from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Race {
    id: &'static str,
    label: &'static str,
    src: Option<&'static str>,
}

impl Race {
    pub const ALL: [Race; 4] = [
        Self {
            id: "lazy",
            label: "Discarded loop",
            src: Some("(λx. z) ((λw. w w) (λw. w w))"),
        },
        Self {
            id: "dup",
            label: "Duplicated work",
            src: Some("(λx. x x x) ((λy. y) a)"),
        },
        Self {
            id: "under",
            label: "Under a binder",
            src: Some("λz. (λx. x) ((λy. y) z)"),
        },
        Self {
            id: "bench",
            label: "Workbench term",
            src: None,
        },
    ];

    pub fn find(id: &str) -> Option<Race> {
        Self::ALL.into_iter().find(|r| r.id == id)
    }

    pub fn id(&self) -> &'static str {
        self.id
    }

    pub fn label(&self) -> &'static str {
        self.label
    }

    pub fn src(&self) -> Option<&'static str> {
        self.src
    }
}

impl std::str::FromStr for Race {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::find(s).ok_or(())
    }
}
