/// JavaScript's regex classes, so lines and names split exactly as the JS engine splits them.
pub struct Line;

impl Line {
    /// `\s`.
    pub fn space(c: char) -> bool {
        matches!(
            c,
            '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
                ..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}'
        )
    }

    /// A character `.` does not match.
    pub fn breaks(c: char) -> bool {
        matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
    }

    /// `String.prototype.trim`.
    pub fn trim(s: &str) -> &str {
        s.trim_matches(Self::space)
    }

    /// `^\s*--\s?(.*)$`: the note on a comment line.
    pub fn note(line: &str) -> Option<&str> {
        let rest = line.trim_start_matches(Self::space).strip_prefix("--")?;
        let tail = rest
            .chars()
            .next()
            .filter(|c| Self::space(*c))
            .map_or(rest, |c| &rest[c.len_utf8()..]);
        match (tail.chars().any(Self::breaks), rest.chars().any(Self::breaks)) {
            (false, _) => Some(tail),
            (true, false) => Some(rest),
            (true, true) => None,
        }
    }

    /// `^\s*def\s+(\S+?)\s*:=(.*)$`: the name and source of a definition line.
    pub fn def(line: &str) -> Option<(&str, &str)> {
        let rest = line.trim_start_matches(Self::space).strip_prefix("def")?;
        let name = rest.trim_start_matches(Self::space);
        (name.len() < rest.len()).then_some(())?;
        name.char_indices()
            .take_while(|(_, c)| !Self::space(*c))
            .map(|(i, c)| i + c.len_utf8())
            .find_map(|end| {
                name[end..]
                    .trim_start_matches(Self::space)
                    .strip_prefix(":=")
                    .filter(|src| !src.chars().any(Self::breaks))
                    .map(|src| (&name[..end], src))
            })
    }
}
