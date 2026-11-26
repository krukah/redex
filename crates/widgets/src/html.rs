/// Markup helpers shared by the widgets.
pub struct Html;

impl Html {
    /// Escapes `&`, `<`, `>` and `"`.
    pub fn esc(s: &str) -> String {
        s.chars().fold(String::with_capacity(s.len()), |mut out, c| {
            match c {
                '&' => out.push_str("&amp;"),
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                '"' => out.push_str("&quot;"),
                _ => out.push(c),
            }
            out
        })
    }
}
