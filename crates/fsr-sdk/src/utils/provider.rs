// SPDX-License-Identifier: MPL-2.0

//! Resolve an upscaling generation using a provider display-name heuristic.
//!
//! Enable `utils` to use this portable, CPU-only resolver. Enumeration remains a
//! separate Windows/DX12 operation. Observed names include `2.3.4`, `3.1.5` and
//! `4.1.1`; AMD provides no naming, ordering or generation-compatibility guarantee.
//!
//! Recognition requires the entire name to contain exactly three nonempty ASCII
//! decimal components separated by full stops, each representable as a `u32`.
//! The major must be 2, 3 or 4. Leading zeros are accepted (`04.01.001` is FSR 4).
//! Signs, whitespace, Unicode digits, overflow, prefixes, suffixes and additional
//! components are ignored as unrecognised names, including lossy UTF-8 replacement
//! characters. Other naming formats require a separately reviewed rule.
//!
//! Explicit choices require exactly one matching entry. Repeated IDs still count
//! as separate entries; minor/patch versions and ID order are never ranked.
//! Failure to resolve a snapshot is a [`SelectionError`], not a native SDK error
//! or proof that the hardware lacks support. Inspect all entries when diagnosing:
//!
//! ```
//! use fsr_sdk::{
//!     query::{ProviderInfo, ProviderSelection},
//!     utils::provider::{FsrChoice, SelectionError, select_provider},
//! };
//!
//! fn resolve(providers: &[ProviderInfo]) -> Result<ProviderSelection, SelectionError> {
//!     for provider in providers {
//!         eprintln!("provider {:?}: {}", provider.id(), provider.name());
//!     }
//!     select_provider(providers, FsrChoice::Fsr4)
//! }
//! ```
//!
//! Obtain the snapshot with an acquired DX12 setup's `providers()`, then bind
//! the returned request with `setup = setup.with_provider(selection)`. Planning,
//! memory estimation and creation share that one selection. Native failures
//! retain the separate core error type and exact native codes.
//!
//! [`FsrChoice::Auto`] returns [`ProviderSelection::Default`] even for an empty
//! list and delegates to the SDK's then-default route. Callers may skip
//! enumeration and use Default directly. Explicit choices never fall back.
//!
//! Enumeration is bounded and need not be complete. Success classifies only the
//! supplied snapshot; IDs need not remain valid between calls. Core performs fresh
//! membership and post-creation live-identity checks, but cannot detect reuse of
//! an ID for another provider or prove which generation executes. This resolver
//! makes no native calls and certifies no provider/hardware compatibility.
//! Applications retain resource ownership, native trust, GPU coordination and
//! context recreation: a selection cannot switch a live upscaler.

use crate::query::{ProviderInfo, ProviderSelection};
use std::fmt;

/// Requested generation, interpreted only by the documented name heuristic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FsrChoice {
    /// Use the SDK's then-default route without inspecting the snapshot.
    Auto,
    /// Require exactly one recognised major-2 entry.
    Fsr2,
    /// Require exactly one recognised major-3 entry.
    Fsr3,
    /// Require exactly one recognised major-4 entry.
    Fsr4,
}

/// Failure to resolve the supplied snapshot, without a native support verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SelectionError {
    /// No entry was recognisable as the requested generation.
    NoRecognizedMatch {
        /// Explicit generation requested by the caller.
        requested: FsrChoice,
    },
    /// Multiple entries matched; no implicit version ranking was attempted.
    Ambiguous {
        /// Explicit generation requested by the caller.
        requested: FsrChoice,
        /// Exact matching-entry count, including repeated IDs.
        matches: usize,
    },
}

impl fmt::Display for SelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoRecognizedMatch { requested } => write!(
                f,
                "no provider name in the supplied snapshot was recognised as {requested:?}"
            ),
            Self::Ambiguous { requested, matches } => write!(
                f,
                "{matches} provider entries in the supplied snapshot matched {requested:?}"
            ),
        }
    }
}

impl std::error::Error for SelectionError {}

/// Resolve a choice to the core setup provider request.
///
/// `Auto` always returns `Ok(ProviderSelection::Default)`. An explicit generation
/// requires exactly one recognised entry; no match or multiple entries return a [`SelectionError`].
/// No fallback, version ranking, deduplication or native operation occurs.
/// Resolution and ambiguity counts are independent of input order. See the
/// [module documentation](self) for exact recognition rules and snapshot limits.
pub fn select_provider(
    providers: &[ProviderInfo],
    choice: FsrChoice,
) -> Result<ProviderSelection, SelectionError> {
    let major = match choice {
        FsrChoice::Auto => return Ok(ProviderSelection::Default),
        FsrChoice::Fsr2 => 2,
        FsrChoice::Fsr3 => 3,
        FsrChoice::Fsr4 => 4,
    };
    let mut matches = providers
        .iter()
        .filter(|provider| recognized_major(provider.name()) == Some(major));
    let first = matches
        .next()
        .ok_or(SelectionError::NoRecognizedMatch { requested: choice })?;
    let count = 1 + matches.count();
    if count != 1 {
        return Err(SelectionError::Ambiguous {
            requested: choice,
            matches: count,
        });
    }
    Ok(ProviderSelection::Explicit(first.id()))
}

fn recognized_major(name: &str) -> Option<u32> {
    fn component(text: &str) -> Option<u32> {
        if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        text.parse().ok()
    }
    let mut components = name.split('.');
    let major = component(components.next()?)?;
    component(components.next()?)?;
    component(components.next()?)?;
    (components.next().is_none() && matches!(major, 2..=4)).then_some(major)
}

#[cfg(test)]
mod tests {
    use super::recognized_major;

    #[test]
    fn strict_ascii_names_with_bounded_numeric_components() {
        for (name, expected) in [
            ("2.3.4", 2),
            ("3.1.5", 3),
            ("4.1.1", 4),
            ("04.01.001", 4),
            ("2.0.0", 2),
            ("3.4294967295.4294967295", 3),
            ("000000000000000000000004.000000000000.000000000000", 4),
        ] {
            assert_eq!(recognized_major(name), Some(expected), "{name:?}");
        }
        for name in [
            "",
            "4",
            "4.1",
            ".1.1",
            "4..1",
            "4.1.",
            "..",
            "4.1.1.",
            "4.1.1.2",
            "4.1.1-extra",
            "FSR 4.1.1",
            " 4.1.1",
            "4.1.1 ",
            "4. 1.1",
            "4.1.1\n",
            "+4.1.1",
            "4.+1.1",
            "4.1.+1",
            "-4.1.1",
            "4.-1.1",
            "4.1.-1",
            "４.1.1",
            "4.١.1",
            "4.1.１",
            "4．1.1",
            "4.1.1\u{fffd}",
            "4.1.1\0",
            "0.1.1",
            "1.1.1",
            "5.1.1",
            "14.1.1",
            "4294967296.1.1",
            "4.4294967296.1",
            "4.1.4294967296",
            "99999999999999999999999.1.1",
            "4.1e0.1",
            "4.0x1.1",
            "4.1_0.1",
        ] {
            assert_eq!(recognized_major(name), None, "{name:?}");
        }
    }
}
