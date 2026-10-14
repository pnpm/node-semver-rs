//! `internal/re.js`: the token sources, composed exactly as npm composes them,
//! compiled in their "safe" form (`makeSafeRegex`), which is what every
//! function this module ports matches with.

use regress::Regex;
use std::sync::OnceLock;

const MAX_LENGTH: usize = 256;
const MAX_SAFE_COMPONENT_LENGTH: usize = 16;
const MAX_SAFE_BUILD_LENGTH: usize = MAX_LENGTH - 6;
const LETTERDASHNUMBER: &str = "[a-zA-Z0-9-]";

pub(crate) struct Re {
    pub(crate) full: Regex,
    pub(crate) xrange: Regex,
    pub(crate) coerce: Regex,
    pub(crate) tilde_trim: Regex,
    pub(crate) tilde: Regex,
    pub(crate) caret_trim: Regex,
    pub(crate) caret: Regex,
    pub(crate) comparator: Regex,
    pub(crate) comparator_trim: Regex,
    pub(crate) hyphen_range: Regex,
    pub(crate) star: Regex,
    pub(crate) gte0: Regex,
}

pub(crate) fn re() -> &'static Re {
    static RE: OnceLock<Re> = OnceLock::new();
    RE.get_or_init(build)
}

/// `makeSafeRegex`: bound the greedy tokens npm bounds.
fn safe(value: &str) -> Regex {
    let mut value = value.to_string();
    for (token, max) in [
        (r"\s", 1),
        (r"\d", MAX_LENGTH),
        (LETTERDASHNUMBER, MAX_SAFE_BUILD_LENGTH),
    ] {
        value = value
            .replace(&format!("{token}*"), &format!("{token}{{0,{max}}}"))
            .replace(&format!("{token}+"), &format!("{token}{{1,{max}}}"));
    }
    Regex::with_flags(&value, "").expect("npm semver's regular expressions compile")
}

fn build() -> Re {
    let numeric = r"0|[1-9]\d*";
    let numeric_loose = r"\d+";
    let non_numeric = format!(r"\d*[a-zA-Z-]{LETTERDASHNUMBER}*");
    let main = format!(r"({numeric})\.({numeric})\.({numeric})");
    let main_loose = format!(r"({numeric_loose})\.({numeric_loose})\.({numeric_loose})");
    let prerelease_identifier = format!("(?:{numeric}|{non_numeric})");
    let prerelease_identifier_loose = format!("(?:{numeric_loose}|{non_numeric})");
    let prerelease = format!(r"(?:-({prerelease_identifier}(?:\.{prerelease_identifier})*))");
    let prerelease_loose =
        format!(r"(?:-?({prerelease_identifier_loose}(?:\.{prerelease_identifier_loose})*))");
    let build_identifier = format!("{LETTERDASHNUMBER}+");
    let build = format!(r"(?:\+({build_identifier}(?:\.{build_identifier})*))");
    let full_plain = format!("v?{main}{prerelease}?{build}?");
    let loose_plain = format!(r"[v=\s]*{main_loose}{prerelease_loose}?{build}?");
    let gtlt = "((?:<|>)?=?)";
    let xrange_identifier = format!(r"{numeric}|x|X|\*");
    let xrange_plain = format!(
        r"[v=\s]*({xrange_identifier})(?:\.({xrange_identifier})(?:\.({xrange_identifier})(?:{prerelease})?{build}?)?)?"
    );
    let max = MAX_SAFE_COMPONENT_LENGTH;
    let coerce_plain =
        format!(r"(^|[^\d])(\d{{1,{max}}})(?:\.(\d{{1,{max}}}))?(?:\.(\d{{1,{max}}}))?");
    let lone_tilde = "(?:~>?)";
    let lone_caret = r"(?:\^)";
    Re {
        full: safe(&format!("^{full_plain}$")),
        xrange: safe(&format!(r"^{gtlt}\s*{xrange_plain}$")),
        coerce: safe(&format!(r"{coerce_plain}(?:$|[^\d])")),
        tilde_trim: safe(&format!(r"(\s*){lone_tilde}\s+")),
        tilde: safe(&format!("^{lone_tilde}{xrange_plain}$")),
        caret_trim: safe(&format!(r"(\s*){lone_caret}\s+")),
        caret: safe(&format!("^{lone_caret}{xrange_plain}$")),
        comparator: safe(&format!(r"^{gtlt}\s*({full_plain})$|^$")),
        comparator_trim: safe(&format!(r"(\s*){gtlt}\s*({loose_plain}|{xrange_plain})")),
        hyphen_range: safe(&format!(r"^\s*({xrange_plain})\s+-\s+({xrange_plain})\s*$")),
        star: safe(r"(<|>)?=?\s*\*"),
        gte0: safe(r"^\s*>=\s*0\.0\.0\s*$"),
    }
}
