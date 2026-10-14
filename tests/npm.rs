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

/// `[left, right, compare(left, right, { loose: true, includePrerelease: true })]`
/// from semver 7.7.1, `null` where it throws.
#[test]
fn loose_compare_matches_npm_semver() {
    use std::cmp::Ordering;
    let rows: Vec<(String, String, Option<i8>)> =
        serde_json::from_str(include_str!("fixtures/npm-semver-7.7.1-compare-loose.json")).unwrap();
    let mut failures = Vec::new();
    for (left, right, expected) in &rows {
        let actual = npm::compare(left, right, true).map(|order| match order {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        });
        if actual != *expected {
            failures.push(format!(
                "compare({left:?}, {right:?}): npm {expected:?}, ours {actual:?}"
            ));
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

/// `[range, new Range(range).set as [operator, version|null], minVersion(range)]`
/// from semver 7.7.1; `minVersion` is `[version, major, minor, patch,
/// prerelease]`, `null`, or `"throw"`.
#[test]
fn range_sets_and_min_version_match_npm_semver() {
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/npm-semver-7.7.1-ranges.json")).unwrap();
    let mut failures = Vec::new();
    for row in &rows {
        let input = row[0].as_str().unwrap();
        let set = npm::range_set(input).map(|sets| {
            sets.iter()
                .map(|set| {
                    set.iter()
                        .map(|comparator| {
                            serde_json::json!([
                                comparator.operator,
                                comparator
                                    .version
                                    .as_ref()
                                    .map(|version| version.version.clone()),
                            ])
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        });
        let set = set.map_or(Value::Null, |set| serde_json::json!(set));
        if set != row[1] {
            failures.push(format!("set({input:?}): npm {}, ours {set}", row[1]));
        }
        let minimum = match npm::min_version(input) {
            Err(_) => Value::from("throw"),
            Ok(None) => Value::Null,
            Ok(Some(version)) => {
                let prerelease: Vec<Value> = version
                    .prerelease
                    .iter()
                    .map(|id| match id {
                        Identifier::Numeric(number) => Value::from(*number),
                        Identifier::Alphanumeric(text) => Value::from(text.clone()),
                    })
                    .collect();
                serde_json::json!([
                    version.version,
                    version.major,
                    version.minor,
                    version.patch,
                    prerelease
                ])
            }
        };
        if minimum != row[2] {
            failures.push(format!(
                "minVersion({input:?}): npm {}, ours {minimum}",
                row[2]
            ));
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
