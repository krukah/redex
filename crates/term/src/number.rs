/// Numbers printed the way JavaScript prints them, for output that must match the JS widgets.
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

    /// Adds one unit in the last place of a decimal string.
    fn bump(s: &str) -> String {
        let mut digits = s.chars().rev().collect::<Vec<char>>();
        let carry = digits.iter_mut().filter(|c| c.is_ascii_digit()).try_for_each(|c| {
            if *c == '9' {
                *c = '0';
                Ok(())
            } else {
                *c = (*c as u8 + 1) as char;
                Err(())
            }
        });
        if carry.is_ok() {
            digits.push('1');
        }
        digits.into_iter().rev().collect()
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
