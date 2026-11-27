use wasm_bindgen::JsCast;
use web_sys::Element;

/// A CSS property a tween animates or a widget sets inline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prop {
    Opacity,
    Transform,
    Width,
    Height,
    StrokeDashoffset,
    Left,
    Top,
    Flex,
    Visibility,
}

impl Prop {
    /// The property's name in a WAAPI keyframe.
    pub fn key(self) -> &'static str {
        match self {
            Self::StrokeDashoffset => "strokeDashoffset",
            _ => self.css(),
        }
    }

    /// The property's name in a stylesheet.
    pub fn css(self) -> &'static str {
        match self {
            Self::Opacity => "opacity",
            Self::Transform => "transform",
            Self::Width => "width",
            Self::Height => "height",
            Self::StrokeDashoffset => "stroke-dashoffset",
            Self::Left => "left",
            Self::Top => "top",
            Self::Flex => "flex",
            Self::Visibility => "visibility",
        }
    }

    /// Sets the property inline on an element, HTML or SVG.
    pub fn set<T>(self, el: &Element, value: &T)
    where
        T: std::fmt::Display,
    {
        if let Ok(style) =
            js_sys::Reflect::get(el, &"style".into()).and_then(JsCast::dyn_into::<web_sys::CssStyleDeclaration>)
        {
            let _ = style.set_property(self.css(), &value.to_string());
        }
    }
}
