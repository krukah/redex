/// The HTML and SVG elements the widgets draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    Span,
    Button,
    Div,
    P,
    B,
    I,
    Code,
    Small,
    Kbd,
    Li,
    Ol,
    Header,
    Textarea,
    Input,
    Select,
    Svg,
    G,
    Path,
    Circle,
    Rect,
    Line,
    Text,
    Title,
    Defs,
    Marker,
}

impl Tag {
    pub const SVG: &'static str = "http://www.w3.org/2000/svg";

    pub fn name(self) -> &'static str {
        match self {
            Self::Span => "span",
            Self::Button => "button",
            Self::Div => "div",
            Self::P => "p",
            Self::B => "b",
            Self::I => "i",
            Self::Code => "code",
            Self::Small => "small",
            Self::Kbd => "kbd",
            Self::Li => "li",
            Self::Ol => "ol",
            Self::Header => "header",
            Self::Textarea => "textarea",
            Self::Input => "input",
            Self::Select => "select",
            Self::Svg => "svg",
            Self::G => "g",
            Self::Path => "path",
            Self::Circle => "circle",
            Self::Rect => "rect",
            Self::Line => "line",
            Self::Text => "text",
            Self::Title => "title",
            Self::Defs => "defs",
            Self::Marker => "marker",
        }
    }

    /// An element of the SVG namespace.
    pub fn svg(self) -> bool {
        matches!(
            self,
            Self::Svg
                | Self::G
                | Self::Path
                | Self::Circle
                | Self::Rect
                | Self::Line
                | Self::Text
                | Self::Title
                | Self::Defs
                | Self::Marker
        )
    }
}
