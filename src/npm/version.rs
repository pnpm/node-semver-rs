//! `classes/semver.js` (construction and `format`), `functions/parse.js`,
//! `functions/valid.js` and `functions/coerce.js` without options.

use super::{js, re::re};
use std::fmt;

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
    if js::length(text) > MAX_LENGTH {
        return None;
    }
    let groups = js::match_groups(js::trim(text), &re().full)?;
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
