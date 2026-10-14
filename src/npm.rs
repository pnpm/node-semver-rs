//! npm's `semver` 7.7.1 in its default (strict) mode, as the JavaScript
//! package computes it: [`valid`], [`parse`] (`new SemVer()`), [`valid_range`]
//! and [`coerce`].
//!
//! The rest of this crate parses ranges loosely into a normalized interval
//! model, which is right for resolution but cannot answer what npm's functions
//! return as text: `validRange("^0.0")` is `"<0.1.0-0"`, `valid("1.2.3-01")` is
//! `null`. This module ports the package's code paths one to one. Its regular
//! expressions are built from the same sources as `internal/re.js`, including
//! the length limits npm substitutes into them, and are run with `regress`, a
//! JavaScript regex engine, so matching, `\s` and `String.prototype.trim` follow
//! JavaScript.

mod js;
mod range;
mod re;
mod version;

pub use range::{min_version, range_set, valid_range, Comparator, MinVersionError};
pub use version::{coerce, compare, parse, parse_loose, valid, Identifier, Version};
