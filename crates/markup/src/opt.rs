/// A checkbox's option.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opt {
    Arcs,
    Db,
    Eta,
}

impl std::fmt::Display for Opt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Arcs => "arcs",
            Self::Db => "db",
            Self::Eta => "eta",
        })
    }
}
