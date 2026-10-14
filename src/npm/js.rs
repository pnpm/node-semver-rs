//! The JavaScript built-ins npm's `semver` relies on.

use regress::Regex;

/// `WhiteSpace` and `LineTerminator` of ECMAScript: what `\s` and `trim()` match.
pub(crate) fn is_space(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
    )
}

/// `String.prototype.trim`.
pub(crate) fn trim(text: &str) -> &str {
    text.trim_matches(is_space)
}

/// `text.split(/\s+/)`: empty leading and trailing pieces included.
pub(crate) fn split_spaces(text: &str) -> Vec<&str> {
    let mut pieces = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((index, c)) = chars.next() {
        if !is_space(c) {
            continue;
        }
        pieces.push(&text[start..index]);
        let mut end = index + c.len_utf8();
        while let Some(&(next, c)) = chars.peek() {
            if !is_space(c) {
                break;
            }
            end = next + c.len_utf8();
            chars.next();
        }
        start = end;
    }
    pieces.push(&text[start..]);
    pieces
}

/// `text.replace(/\s+/g, ' ')`.
pub(crate) fn collapse_spaces(text: &str) -> String {
    split_spaces(text).join(" ")
}

/// The length JavaScript reports: UTF-16 code units.
pub(crate) fn length(text: &str) -> usize {
    text.chars().map(char::len_utf16).sum()
}

/// `text.replace(regex, fn)` for the first match, or every match when `global`.
/// The function receives the match and its groups; an unmatched group is `None`.
pub(crate) fn replace(
    text: &str,
    regex: &Regex,
    global: bool,
    mut replacement: impl FnMut(&[Option<&str>]) -> String,
) -> String {
    let mut output = String::with_capacity(text.len());
    let mut copied = 0;
    let mut start = 0;
    while start <= text.len() {
        let Some(found) = regex.find_from(text, start).next() else {
            break;
        };
        let range = found.range();
        let groups: Vec<Option<&str>> = (0..found.captures.len() + 1)
            .map(|index| found.group(index).map(|group| &text[group]))
            .collect();
        output.push_str(&text[copied..range.start]);
        output.push_str(&replacement(&groups));
        copied = range.end;
        if !global {
            break;
        }
        start = if range.is_empty() {
            text[range.end..]
                .chars()
                .next()
                .map_or(range.end + 1, |c| range.end + c.len_utf8())
        } else {
            range.end
        };
    }
    output.push_str(&text[copied..]);
    output
}

/// The groups of `text.match(regex)`, or `None`.
pub(crate) fn match_groups<'t>(text: &'t str, regex: &Regex) -> Option<Vec<Option<&'t str>>> {
    let found = regex.find(text)?;
    Some(
        (0..found.captures.len() + 1)
            .map(|index| found.group(index).map(|group| &text[group]))
            .collect(),
    )
}

/// `+digits` for a string of ASCII digits.
pub(crate) fn number(digits: &str) -> f64 {
    digits.parse().unwrap_or(f64::NAN)
}

/// `` `${+digits + 1}` ``.
pub(crate) fn increment(digits: &str) -> String {
    number_to_string(number(digits) + 1.0)
}

/// `Number.prototype.toString` of a non-negative integer.
pub(crate) fn number_to_string(value: f64) -> String {
    // Shortest round-trip digits, as JavaScript chooses them.
    let scientific = format!("{value:e}");
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
    let exponent: i32 = exponent.parse().unwrap_or(0);
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    if exponent >= 21 {
        let (first, rest) = digits.split_at(1);
        let fraction = if rest.is_empty() {
            String::new()
        } else {
            format!(".{rest}")
        };
        return format!("{first}{fraction}e+{exponent}");
    }
    let width = exponent as usize + 1;
    if digits.len() >= width {
        return digits[..width].to_string();
    }
    format!("{digits}{}", "0".repeat(width - digits.len()))
}
