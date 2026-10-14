//! `classes/range.js` and `classes/comparator.js`, as far as `validRange`
//! goes: parsing a range into comparator sets and formatting it back.

use super::{js, re::re, version};

/// `<0.0.0-0`: the comparator no version satisfies.
const NULL_SET: &str = "<0.0.0-0";

/// `semver.validRange(text)`: the normalized range, `"*"` for any version,
/// or `None` where `new Range(text)` throws.
pub fn valid_range(text: &str) -> Option<String> {
    let raw = js::collapse_spaces(js::trim(text));
    let mut sets = raw
        .split("||")
        .map(|part| parse_range(js::trim(part)))
        .collect::<Option<Vec<_>>>()?;
    sets.retain(|set| !set.is_empty());
    if sets.is_empty() {
        return None;
    }
    if sets.len() > 1 {
        let first = sets[0].clone();
        sets.retain(|set| set[0] != NULL_SET);
        if sets.is_empty() {
            sets = vec![first];
        } else if let Some(any) = sets.iter().find(|set| set.len() == 1 && set[0].is_empty()) {
            sets = vec![any.clone()];
        }
    }
    let formatted = sets
        .iter()
        .map(|set| {
            set.iter()
                .map(|value| js::trim(value))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("||");
    Some(if formatted.is_empty() {
        "*".to_string()
    } else {
        formatted
    })
}

/// `Range.parseRange`: the comparator values of one `||` alternative.
fn parse_range(range: &str) -> Option<Vec<String>> {
    let re = re();
    let range = js::replace(range, &re.hyphen_range, false, hyphen_replace);
    let range = js::replace(&range, &re.comparator_trim, true, |groups| {
        groups[1..=3]
            .iter()
            .map(|group| group.unwrap_or_default())
            .collect()
    });
    let range = js::replace(&range, &re.tilde_trim, true, |groups| {
        format!("{}~", groups[1].unwrap_or_default())
    });
    let range = js::replace(&range, &re.caret_trim, true, |groups| {
        format!("{}^", groups[1].unwrap_or_default())
    });
    let parsed: Vec<String> = range.split(' ').map(parse_comparator).collect();
    let joined = parsed.join(" ");
    let values = js::split_spaces(&joined)
        .into_iter()
        .map(|comp| {
            comparator(&js::replace(js::trim(comp), &re.gte0, false, |_| {
                String::new()
            }))
        })
        .collect::<Option<Vec<_>>>()?;
    let mut unique: Vec<String> = Vec::new();
    for value in values {
        if value == NULL_SET {
            return Some(vec![value]);
        }
        if !unique.contains(&value) {
            unique.push(value);
        }
    }
    if unique.len() > 1 {
        unique.retain(|value| !value.is_empty());
    }
    Some(unique)
}

/// `new Comparator(text).value`: `""` for any version.
fn comparator(text: &str) -> Option<String> {
    let comp = js::split_spaces(js::trim(text)).join(" ");
    let groups = js::match_groups(&comp, &re().comparator)?;
    let operator = match groups[1].unwrap_or_default() {
        "=" => "",
        operator => operator,
    };
    match groups[2].filter(|version| !version.is_empty()) {
        None => Some(String::new()),
        Some(version) => Some(format!("{operator}{}", version::parse(version)?.version)),
    }
}

/// `parseComparator`.
fn parse_comparator(comp: &str) -> String {
    let comp = each(js::trim(comp), replace_caret);
    let comp = each(js::trim(&comp), replace_tilde);
    let comp = each(&comp, replace_xrange);
    js::replace(js::trim(&comp), &re().star, false, |_| String::new())
}

/// `comp.split(/\s+/).map(f).join(' ')`.
fn each(comp: &str, f: fn(&str) -> String) -> String {
    js::split_spaces(comp)
        .into_iter()
        .map(f)
        .collect::<Vec<_>>()
        .join(" ")
}

/// `isX`.
fn is_x(id: Option<&str>) -> bool {
    match id {
        None | Some("") | Some("*") => true,
        Some(id) => id.eq_ignore_ascii_case("x"),
    }
}

fn group<'a>(groups: &[Option<&'a str>], index: usize) -> &'a str {
    groups[index].unwrap_or_default()
}

/// `replaceTilde`.
fn replace_tilde(comp: &str) -> String {
    js::replace(comp, &re().tilde, false, |groups| {
        let (major, minor, patch, pre) = (groups[1], groups[2], groups[3], groups[4]);
        let (mj, mn, pt) = (group(groups, 1), group(groups, 2), group(groups, 3));
        if is_x(major) {
            String::new()
        } else if is_x(minor) {
            format!(">={mj}.0.0 <{}.0.0-0", js::increment(mj))
        } else if is_x(patch) {
            format!(">={mj}.{mn}.0 <{mj}.{}.0-0", js::increment(mn))
        } else if let Some(pre) = pre.filter(|pre| !pre.is_empty()) {
            format!(">={mj}.{mn}.{pt}-{pre} <{mj}.{}.0-0", js::increment(mn))
        } else {
            format!(">={mj}.{mn}.{pt} <{mj}.{}.0-0", js::increment(mn))
        }
    })
}

/// `replaceCaret` without `includePrerelease`.
fn replace_caret(comp: &str) -> String {
    js::replace(comp, &re().caret, false, |groups| {
        let (major, minor, patch, pre) = (groups[1], groups[2], groups[3], groups[4]);
        let (mj, mn, pt) = (group(groups, 1), group(groups, 2), group(groups, 3));
        let pre = pre
            .filter(|pre| !pre.is_empty())
            .map(|pre| format!("-{pre}"));
        if is_x(major) {
            String::new()
        } else if is_x(minor) {
            format!(">={mj}.0.0 <{}.0.0-0", js::increment(mj))
        } else if is_x(patch) {
            if mj == "0" {
                format!(">={mj}.{mn}.0 <{mj}.{}.0-0", js::increment(mn))
            } else {
                format!(">={mj}.{mn}.0 <{}.0.0-0", js::increment(mj))
            }
        } else {
            let lower = format!(">={mj}.{mn}.{pt}{}", pre.unwrap_or_default());
            if mj != "0" {
                format!("{lower} <{}.0.0-0", js::increment(mj))
            } else if mn != "0" {
                format!("{lower} <{mj}.{}.0-0", js::increment(mn))
            } else {
                format!("{lower} <{mj}.{mn}.{}-0", js::increment(pt))
            }
        }
    })
}

/// `replaceXRange` without `includePrerelease`.
fn replace_xrange(comp: &str) -> String {
    js::replace(js::trim(comp), &re().xrange, false, |groups| {
        let x_major = is_x(groups[2]);
        let x_minor = x_major || is_x(groups[3]);
        let any_x = x_minor || is_x(groups[4]);
        let mut gtlt = group(groups, 1);
        if gtlt == "=" && any_x {
            gtlt = "";
        }
        let (mut major, mut minor) = (group(groups, 2).to_string(), group(groups, 3).to_string());
        if x_major {
            return if gtlt == ">" || gtlt == "<" {
                NULL_SET.into()
            } else {
                "*".into()
            };
        }
        if !gtlt.is_empty() && any_x {
            if x_minor {
                minor = "0".into();
            }
            let patch = "0";
            let mut pre = "";
            if gtlt == ">" {
                gtlt = ">=";
                if x_minor {
                    major = js::increment(&major);
                    minor = "0".into();
                } else {
                    minor = js::increment(&minor);
                }
            } else if gtlt == "<=" {
                gtlt = "<";
                if x_minor {
                    major = js::increment(&major);
                } else {
                    minor = js::increment(&minor);
                }
            }
            if gtlt == "<" {
                pre = "-0";
            }
            return format!("{gtlt}{major}.{minor}.{patch}{pre}");
        }
        if x_minor {
            return format!(">={major}.0.0 <{}.0.0-0", js::increment(&major));
        }
        if any_x {
            return format!(">={major}.{minor}.0 <{major}.{}.0-0", js::increment(&minor));
        }
        group(groups, 0).to_string()
    })
}

/// `hyphenReplace(false)`: `1.2 - 3.4` to `>=1.2.0 <3.5.0-0`.
fn hyphen_replace(groups: &[Option<&str>]) -> String {
    // npm writes `>=${from}` whether or not `from` has a prerelease.
    let (from, f_major, f_minor, f_patch) = (group(groups, 1), groups[2], groups[3], groups[4]);
    let (to, t_major, t_minor, t_patch, t_pre) = (
        group(groups, 7),
        groups[8],
        groups[9],
        groups[10],
        groups[11],
    );
    let (fm, fn_) = (f_major.unwrap_or_default(), f_minor.unwrap_or_default());
    let from = if is_x(f_major) {
        String::new()
    } else if is_x(f_minor) {
        format!(">={fm}.0.0")
    } else if is_x(f_patch) {
        format!(">={fm}.{fn_}.0")
    } else {
        format!(">={from}")
    };
    let (tm, tn, tp) = (
        t_major.unwrap_or_default(),
        t_minor.unwrap_or_default(),
        t_patch.unwrap_or_default(),
    );
    let to = if is_x(t_major) {
        String::new()
    } else if is_x(t_minor) {
        format!("<{}.0.0-0", js::increment(tm))
    } else if is_x(t_patch) {
        format!("<{tm}.{}.0-0", js::increment(tn))
    } else if let Some(pre) = t_pre.filter(|pre| !pre.is_empty()) {
        format!("<={tm}.{tn}.{tp}-{pre}")
    } else {
        format!("<={to}")
    };
    js::trim(&format!("{from} {to}")).to_string()
}
