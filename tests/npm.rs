//! `node_semver::npm` against npm's `semver` 7.7.1. The fixture holds
//! `[input, valid(input), [major, minor, patch, prerelease] of new SemVer(input),
//! validRange(input), coerce(input)?.version]` for hand-picked edge cases and
//! seeded random version and range strings.

use node_semver::npm::{self, Identifier};
use serde_json::Value;

#[test]
fn matches_npm_semver() {
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/npm-semver-7.7.1.json")).unwrap();
    let mut failures = Vec::new();
    for row in &rows {
        let input = row[0].as_str().unwrap();
        let parsed = npm::parse(input).map(|version| {
            let prerelease: Vec<Value> = version
                .prerelease
                .iter()
                .map(|id| match id {
                    Identifier::Numeric(number) => Value::from(*number),
                    Identifier::Alphanumeric(text) => Value::from(text.clone()),
                })
                .collect();
            serde_json::json!([version.major, version.minor, version.patch, prerelease])
        });
        let actual = [
            npm::valid(input).map_or(Value::Null, Value::from),
            parsed.unwrap_or(Value::Null),
            npm::valid_range(input).map_or(Value::Null, Value::from),
            npm::coerce(input).map_or(Value::Null, |version| Value::from(version.version)),
        ];
        for (index, (expected, actual)) in
            row.as_array().unwrap()[1..].iter().zip(&actual).enumerate()
        {
            if expected != actual {
                let name = ["valid", "SemVer", "validRange", "coerce"][index];
                failures.push(format!("{name}({input:?}): npm {expected}, ours {actual}"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} rows differ:\n{}",
        failures.len(),
        rows.len(),
        failures.join("\n")
    );
}
