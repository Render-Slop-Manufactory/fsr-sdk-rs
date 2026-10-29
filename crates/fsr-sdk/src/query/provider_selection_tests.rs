// SPDX-License-Identifier: MPL-2.0

// Construct test-local metadata within its owning privacy boundary. Resolver
// production code uses only public getters, on every host and without DX12.
use super::{ProviderId, ProviderInfo, ProviderSelection};
use crate::utils::provider::{FsrChoice, SelectionError, select_provider};

fn provider(id: u64, name: &str) -> ProviderInfo {
    ProviderInfo {
        id: ProviderId(id),
        name: name.into(),
    }
}

#[test]
fn auto_ignores_empty_malformed_and_ambiguous_snapshots() {
    for providers in [
        vec![],
        vec![provider(1, "4.1.1")],
        vec![provider(1, "selected-fixture"), provider(2, "4.1.1-extra")],
        vec![
            provider(1, "4.1.1"),
            provider(1, "4.1.1"),
            provider(2, "4.2.0"),
        ],
    ] {
        assert_eq!(
            select_provider(&providers, FsrChoice::Auto),
            Ok(ProviderSelection::Default)
        );
    }
}

#[test]
fn unique_generations_return_the_core_setup_selection() {
    let mut providers = vec![
        provider(9, "4.1.1-extra"),
        provider(2, "2.3.4"),
        provider(3, "3.1.5"),
        provider(4, "04.01.001"),
        provider(5, "selected-fixture"),
    ];
    for _ in 0..providers.len() {
        for (choice, id) in [
            (FsrChoice::Fsr2, 2),
            (FsrChoice::Fsr3, 3),
            (FsrChoice::Fsr4, 4),
        ] {
            let selected = select_provider(&providers, choice).unwrap();
            assert_eq!(selected, ProviderSelection::Explicit(ProviderId(id)));
            assert_eq!(
                select_provider(&providers, FsrChoice::Auto).unwrap(),
                ProviderSelection::Default
            );
        }
        providers.rotate_left(1);
    }
}

#[test]
fn absent_generation_never_falls_back_or_infers_support() {
    for providers in [
        vec![],
        vec![provider(2, "2.3.4"), provider(3, "3.1.5")],
        vec![provider(4, "FSR 4.1.1"), provider(5, "4.1.1\u{fffd}")],
    ] {
        assert_eq!(
            select_provider(&providers, FsrChoice::Fsr4),
            Err(SelectionError::NoRecognizedMatch {
                requested: FsrChoice::Fsr4
            })
        );
    }
    for choice in [FsrChoice::Fsr2, FsrChoice::Fsr3, FsrChoice::Fsr4] {
        assert_eq!(
            select_provider(&[], choice),
            Err(SelectionError::NoRecognizedMatch { requested: choice })
        );
    }
}

#[test]
fn every_matching_entry_counts_without_deduplication_or_version_ranking() {
    for (choice, major) in [
        (FsrChoice::Fsr2, 2),
        (FsrChoice::Fsr3, 3),
        (FsrChoice::Fsr4, 4),
    ] {
        for ids in [[8, 8, 8], [8, 9, 10]] {
            let mut providers = vec![
                provider(ids[0], &format!("{major}.0.0")),
                provider(ids[1], &format!("{major}.4294967295.4294967295")),
                provider(ids[2], &format!("{major}.1.1")),
                provider(11, "14.1.1"),
            ];
            for _ in 0..providers.len() {
                assert_eq!(
                    select_provider(&providers, choice),
                    Err(SelectionError::Ambiguous {
                        requested: choice,
                        matches: 3
                    })
                );
                providers.reverse();
                assert_eq!(
                    select_provider(&providers, choice),
                    Err(SelectionError::Ambiguous {
                        requested: choice,
                        matches: 3
                    })
                );
                providers.rotate_left(1);
            }
        }
    }
    let duplicate = provider(4, "4.1.1");
    assert_eq!(
        select_provider(&[duplicate.clone(), duplicate], FsrChoice::Fsr4),
        Err(SelectionError::Ambiguous {
            requested: FsrChoice::Fsr4,
            matches: 2
        })
    );
}

#[test]
fn diagnostics_describe_snapshot_resolution_and_support_standard_errors() {
    use std::error::Error;
    let missing = select_provider(&[], FsrChoice::Fsr3).unwrap_err();
    assert_eq!(
        missing.to_string(),
        "no provider name in the supplied snapshot was recognised as Fsr3"
    );
    let ambiguous = SelectionError::Ambiguous {
        requested: FsrChoice::Fsr4,
        matches: 2,
    };
    assert_eq!(
        ambiguous.to_string(),
        "2 provider entries in the supplied snapshot matched Fsr4"
    );
    assert!(missing.source().is_none());
    let boxed: Box<dyn Error + Send + Sync> = missing.into();
    assert!(boxed.downcast_ref::<SelectionError>().is_some());
}
