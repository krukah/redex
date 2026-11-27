use syntax::*;

/// A start term for the strategy race. `bench` has no source: it races whatever the workbench
/// started from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Race {
    id: &'static str,
    label: &'static str,
    text: Option<&'static str>,
}

impl Race {
    pub const ALL: [Race; 4] = [
        Self {
            id: "lazy",
            label: "Discarded loop",
            text: Some("(λx. z) ((λw. w w) (λw. w w))"),
        },
        Self {
            id: "dup",
            label: "Duplicated work",
            text: Some("(λx. x x x) ((λy. y) a)"),
        },
        Self {
            id: "under",
            label: "Under a binder",
            text: Some("λz. (λx. x) ((λy. y) z)"),
        },
        Self {
            id: "bench",
            label: "Workbench term",
            text: None,
        },
    ];

    pub fn find(id: &str) -> Option<Race> {
        Self::ALL.into_iter().find(|r| r.id() == id)
    }

    pub fn id(&self) -> &'static str {
        self.id
    }

    pub fn label(&self) -> &'static str {
        self.label
    }

    fn text(&self) -> Option<&'static str> {
        self.text
    }

    pub fn src(&self) -> Option<Source> {
        self.text().map(Source::from)
    }
}

impl std::str::FromStr for Race {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::find(s).ok_or(())
    }
}
