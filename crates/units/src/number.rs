/// Numbers printed the way JavaScript prints them: the format the fixtures record.
pub struct Number;

impl Number {
    /// `String(x)`.
    pub fn show(x: f64) -> String {
        match x {
            _ if x.is_nan() => "NaN".to_string(),
            _ if x.is_infinite() => (if x > 0.0 { "Infinity" } else { "-Infinity" }).to_string(),
            _ if x == 0.0 => "0".to_string(),
            _ if x.abs() >= 1e21 || x.abs() < 1e-6 => Self::exponent(x),
            _ => format!("{x}"),
        }
    }

    /// `x.toFixed(digits)`: the exact binary value, an exact tie rounded away from zero.
    pub fn fixed(x: f64, digits: usize) -> String {
        let more = digits + 1;
        let tie = format!("{:.more$}", x.abs());
        let rounded = if tie.ends_with('5') && format!("{:.80}", x.abs()).trim_end_matches('0') == tie {
            Self::bump(&tie[..tie.len() - 1])
        } else {
            format!("{:.digits$}", x.abs())
        };
        let rounded = rounded.trim_end_matches('.').to_string();
        if x < 0.0 { format!("-{rounded}") } else { rounded }
    }

    /// Adds one unit in the last place of a decimal string. A tie only happens below 2^53, so the
    /// digits fit a `u128`.
    fn bump(s: &str) -> String {
        let (int, frac) = s.split_once('.').unwrap_or((s, ""));
        let n = format!("{int}{frac}").parse::<u128>().unwrap_or(0) + 1;
        let digits = format!("{n:0width$}", width = int.len() + frac.len());
        if frac.is_empty() {
            digits
        } else {
            let (whole, part) = digits.split_at(digits.len() - frac.len());
            format!("{whole}.{part}")
        }
    }

    fn exponent(x: f64) -> String {
        let s = format!("{x:e}");
        match s.split_once('e') {
            Some((m, e)) if e.starts_with('-') => format!("{m}e{e}"),
            Some((m, e)) => format!("{m}e+{e}"),
            None => s,
        }
    }
}
