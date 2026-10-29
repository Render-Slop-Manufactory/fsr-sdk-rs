// SPDX-License-Identifier: MPL-2.0

use fsr_sdk::query::{ProviderId, ProviderInfo, ProviderSelection, ResourceFlags};

#[test]
fn synthetic_provider_metadata_is_owned_and_every_id_bit_round_trips() {
    for value in [0, 1, 42, 1 << 63, u64::MAX] {
        let id = ProviderId::from_u64(value);
        assert_eq!(id.as_u64(), value);
        assert_eq!(u64::from(ProviderId::from(value)), value);
        assert_ne!(ProviderSelection::Explicit(id), ProviderSelection::Default);
        let mut name = String::from("4.1.1 *\0 synthetic");
        let metadata = ProviderInfo::new(id, name.clone());
        name.clear();
        assert_eq!(metadata.id(), id);
        assert_eq!(metadata.name(), "4.1.1 *\0 synthetic");
        assert_eq!(ProviderInfo::new(id, "").name(), "");
    }
}

#[test]
fn resource_mask_composition_preserves_unknown_bits_in_full_width_operations() {
    let unknown = 1u64 << 63;
    let all = ResourceFlags::from_bits_retain(u64::MAX);
    assert_eq!(all.bits(), u64::MAX);
    let mut mask = ResourceFlags::empty();
    mask |= ResourceFlags::COLOR | ResourceFlags::from_bits_retain(unknown);
    assert!(mask.contains(ResourceFlags::COLOR));
    assert_eq!(mask.unknown_bits(), unknown);
    assert_eq!((mask & all).bits(), mask.bits());
    mask &= ResourceFlags::from_bits_retain(unknown);
    assert_eq!(mask.bits(), unknown);
    assert!(!mask.contains(ResourceFlags::COLOR));
}

#[cfg(feature = "utils")]
#[test]
fn resolver_policy_can_be_tested_without_a_runtime_or_private_metadata_access() {
    use fsr_sdk::utils::provider::{FsrChoice, SelectionError, select_provider};
    let id = ProviderId::from_u64(u64::MAX);
    let providers = [ProviderInfo::new(id, "4.1.1")];
    assert_eq!(
        select_provider(&providers, FsrChoice::Fsr4).unwrap(),
        ProviderSelection::Explicit(id)
    );
    let ambiguous = [
        providers[0].clone(),
        ProviderInfo::new(ProviderId::from_u64(0), "4.1.2"),
    ];
    assert!(matches!(
        select_provider(&ambiguous, FsrChoice::Fsr4),
        Err(SelectionError::Ambiguous { matches: 2, .. })
    ));
}
