/// A CSS property a tween animates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prop {
    Opacity,
    Transform,
    Width,
    Height,
    StrokeDashoffset,
}

impl Prop {
    /// The property's name in a WAAPI keyframe.
    pub fn key(self) -> &'static str {
        match self {
            Self::Opacity => "opacity",
            Self::Transform => "transform",
            Self::Width => "width",
            Self::Height => "height",
            Self::StrokeDashoffset => "strokeDashoffset",
        }
    }
}
