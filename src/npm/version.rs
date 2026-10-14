//! `classes/semver.js` (construction, `format` and `compare`),
//! `functions/parse.js`, `functions/valid.js`, `functions/coerce.js` and
//! `functions/compare.js`.

use super::{js, re::re};
use std::{cmp::Ordering, fmt};

const MAX_LENGTH: usize = 256;
const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

/// A prerelease identifier: numeric ones below `MAX_SAFE_INTEGER` become numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identifier {
    Numeric(u64),
    Alphanumeric(String),
}

impl fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Numeric(number) => write!(formatter, "{number}"),
            Self::Alphanumeric(text) => formatter.write_str(text),
        }
    }
}

/// `new SemVer(text)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub prerelease: Vec<Identifier>,
    pub build: Vec<String>,
    /// `version.version`: the canonical text, without build metadata.
    pub version: String,
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.version)
    }
}

/// `semver.parse(text)`: `None` where `new SemVer(text)` throws.
pub fn parse(text: &str) -> Option<Version> {
    parse_with(text, false)
}

/// `semver.parse(text, { loose: true })`: also `v1.2.3`, `=1.2.3`, `01.2.3`
/// and `1.2.3beta`.
pub fn parse_loose(text: &str) -> Option<Version> {
    parse_with(text, true)
}

fn parse_with(text: &str, loose: bool) -> Option<Version> {
    if js::length(text) > MAX_LENGTH {
        return None;
    }
    let regex = if loose { &re().loose } else { &re().full };
    let groups = js::match_groups(js::trim(text), regex)?;
    let component = |index: usize| {
        let value = js::number(groups[index]?);
        (0.0..=MAX_SAFE_INTEGER)
            .contains(&value)
            .then_some(value as u64)
    };
    let (major, minor, patch) = (component(1)?, component(2)?, component(3)?);
    let prerelease: Vec<Identifier> = groups[4]
        .map(|ids| ids.split('.').map(identifier).collect())
        .unwrap_or_default();
    let build = groups[5]
        .map(|ids| ids.split('.').map(str::to_string).collect())
        .unwrap_or_default();
    let mut version = format!("{major}.{minor}.{patch}");
    if !prerelease.is_empty() {
        let ids: Vec<String> = prerelease.iter().map(ToString::to_string).collect();
        version.push('-');
        version.push_str(&ids.join("."));
    }
    Some(Version {
        major,
        minor,
        patch,
        prerelease,
        build,
        version,
    })
}

fn identifier(id: &str) -> Identifier {
    if !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit()) {
        let number = js::number(id);
        if (0.0..MAX_SAFE_INTEGER).contains(&number) {
            return Identifier::Numeric(number as u64);
        }
    }
    Identifier::Alphanumeric(id.to_string())
}

impl Version {
    /// `version.compare(other)`: main version, then prerelease; build
    /// metadata is ignored.
    pub fn compare(&self, other: &Self) -> Ordering {
        if self.version == other.version {
            return Ordering::Equal;
        }
        let main =
            (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch));
        main.then_with(|| compare_prerelease(&self.prerelease, &other.prerelease))
    }
}

/// `comparePre`: a version without a prerelease is greater.
fn compare_prerelease(left: &[Identifier], right: &[Identifier]) -> Ordering {
    match (left.is_empty(), right.is_empty()) {
        (true, true) => return Ordering::Equal,
        (false, true) => return Ordering::Less,
        (true, false) => return Ordering::Greater,
        (false, false) => {}
    }
    for index in 0.. {
        match (left.get(index), right.get(index)) {
            (None, None) => return Ordering::Equal,
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            (Some(a), Some(b)) if a == b => {}
            (Some(a), Some(b)) => return compare_identifiers(a, b),
        }
    }
    Ordering::Equal
}

/// `compareIdentifiers`: numeric identifiers compare as numbers and sort
/// before alphanumeric ones, which compare as strings.
fn compare_identifiers(left: &Identifier, right: &Identifier) -> Ordering {
    let numeric = |id: &Identifier| match id {
        Identifier::Numeric(number) => Some(*number as f64),
        Identifier::Alphanumeric(text) => (!text.is_empty()
            && text.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| js::number(text)),
    };
    match (numeric(left), numeric(right)) {
        (Some(a), Some(b)) => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => {
            let utf16 = |id: &Identifier| id.to_string().encode_utf16().collect::<Vec<_>>();
            utf16(left).cmp(&utf16(right))
        }
    }
}

/// `semver.compare(left, right, { loose })`: `None` where either side does not parse.
pub fn compare(left: &str, right: &str, loose: bool) -> Option<Ordering> {
    Some(parse_with(left, loose)?.compare(&parse_with(right, loose)?))
}

/// `semver.valid(text)`.
pub fn valid(text: &str) -> Option<String> {
    parse(text).map(|version| version.version)
}

/// `semver.coerce(text)`: the first version-like run of digits.
pub fn coerce(text: &str) -> Option<Version> {
    let groups = js::match_groups(text, &re().coerce)?;
    let major = groups[2]?;
    let minor = groups[3].unwrap_or("0");
    let patch = groups[4].unwrap_or("0");
    parse(&format!("{major}.{minor}.{patch}"))
}
