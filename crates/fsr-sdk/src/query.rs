// SPDX-License-Identifier: MPL-2.0

//! Owned results of the selected upscaler queries.
//!
//! These types are available without platform or backend features. Native query
//! methods require Windows and `dx12`. Names and opaque IDs are diagnostic
//! metadata; they establish no capability, semantic version or persistent identity.

use fsr_sdk_sys::upscale::{
    FFX_API_QUERY_RESOURCE_INPUT_COLOR, FFX_API_QUERY_RESOURCE_INPUT_DEPTH,
    FFX_API_QUERY_RESOURCE_INPUT_EXPOSURE, FFX_API_QUERY_RESOURCE_INPUT_MV,
    FFX_API_QUERY_RESOURCE_INPUT_REACTIVEMASK,
    FFX_API_QUERY_RESOURCE_INPUT_TRANSPARENCYCOMPOSITION,
};

/// SDK quality-mode names passed to the selected provider's utility queries.
///
/// No ratio, rounding formula or image-quality guarantee is encoded here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum QualityMode {
    /// Native-resolution antialiasing.
    NativeAa,
    /// SDK Quality mode.
    Quality,
    /// SDK Balanced mode.
    Balanced,
    /// SDK Performance mode.
    Performance,
    /// SDK Ultra Performance mode.
    UltraPerformance,
}

impl QualityMode {
    #[cfg(any(test, all(windows, feature = "dx12")))]
    pub(crate) fn native_value(self) -> u32 {
        use fsr_sdk_sys::upscale::*;
        match self {
            Self::NativeAa => FFX_UPSCALE_QUALITY_MODE_NATIVEAA,
            Self::Quality => FFX_UPSCALE_QUALITY_MODE_QUALITY,
            Self::Balanced => FFX_UPSCALE_QUALITY_MODE_BALANCED,
            Self::Performance => FFX_UPSCALE_QUALITY_MODE_PERFORMANCE,
            Self::UltraPerformance => FFX_UPSCALE_QUALITY_MODE_ULTRA_PERFORMANCE,
        }
    }
}

/// Positive SDK jitter sequence length, representable as a native `i32`.
///
/// This validates the count only, not sequence quality or provider compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JitterPhaseCount(std::num::NonZeroI32);

impl JitterPhaseCount {
    /// Accept a positive length; zero and negative values return `Error::Input`.
    pub fn new(count: i32) -> Result<Self, crate::error::Error> {
        if count <= 0 {
            return Err(crate::error::Error::Input(crate::error::InputError::Query(
                crate::error::QueryValidation::JitterPhaseCount,
            )));
        }
        Ok(Self(
            std::num::NonZeroI32::new(count).expect("positive count"),
        ))
    }

    /// Sequence length, preserved exactly as reported or supplied.
    pub fn count(self) -> i32 {
        self.0.get()
    }
}

/// Opaque provider identity, also constructible as ordinary diagnostic data.
///
/// Numeric construction establishes no provenance, membership, capability or persistent identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProviderId(u64);

impl ProviderId {
    /// Retain every numeric bit, including zero; this does not select a default.
    pub const fn from_u64(value: u64) -> Self {
        Self(value)
    }
    /// Losslessly recover the stored numeric value, without interpreting it.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}
impl From<u64> for ProviderId {
    fn from(value: u64) -> Self {
        Self::from_u64(value)
    }
}
impl From<ProviderId> for u64 {
    fn from(value: ProviderId) -> Self {
        value.as_u64()
    }
}

/// Provider request bound to an upscaler setup, not a reservation or capability.
///
/// The default is resolved by the SDK on each native invocation. Explicit IDs
/// are checked against fresh runtime/device enumeration; no fallback is used.
/// Neither route certifies provider support or a persistent native identity.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProviderSelection {
    /// Request the SDK's default at this invocation.
    #[default]
    Default,
    /// Request precisely this freshly checked ID.
    Explicit(ProviderId),
}

impl ProviderSelection {
    #[cfg(any(test, all(windows, feature = "dx12")))]
    pub(crate) fn optional(self) -> Option<ProviderId> {
        match self {
            Self::Default => None,
            Self::Explicit(id) => Some(id),
        }
    }
}

/// Owned provider ID/name metadata, supplied by a caller or copied from a native query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderInfo {
    id: ProviderId,
    name: String,
}

impl ProviderInfo {
    /// Assemble ordinary metadata; no runtime membership or capability is implied.
    /// The supplied name is owned verbatim, including empty or duplicate names.
    pub fn new(id: ProviderId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }

    /// Stored opaque ID; construction alone establishes no native provenance.
    pub fn id(&self) -> ProviderId {
        self.id
    }

    /// Stored display name. Native queries replace invalid UTF-8 bytes with U+FFFD;
    /// caller-supplied Rust strings are retained verbatim.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Provider-specific GPU memory accounting or a pre-create estimate.
///
/// Values are not a reservation, a device budget or a guarantee of later
/// allocation. Zero is permitted. No relation between the two values is imposed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuMemoryUsage {
    total_bytes: u64,
    aliasable_bytes: u64,
}

impl GpuMemoryUsage {
    /// Native total usage or estimated usage, in bytes.
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// Native aliasable usage or estimated usage, in bytes.
    pub fn aliasable_bytes(&self) -> u64 {
        self.aliasable_bytes
    }
}

/// Resource-input mask retaining every native bit, including unknown bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceFlags(u64);

impl ResourceFlags {
    /// An empty resource mask.
    pub const fn empty() -> Self {
        Self(0)
    }
    /// Retain all native bits, including bits unknown to this SDK baseline.
    pub const fn from_bits_retain(bits: u64) -> Self {
        Self(bits)
    }

    /// Scene colour input.
    pub const COLOR: Self = Self(FFX_API_QUERY_RESOURCE_INPUT_COLOR as u64);
    /// Depth input.
    pub const DEPTH: Self = Self(FFX_API_QUERY_RESOURCE_INPUT_DEPTH as u64);
    /// Motion-vector input.
    pub const MOTION_VECTORS: Self = Self(FFX_API_QUERY_RESOURCE_INPUT_MV as u64);
    /// Exposure requirement, satisfied by an explicit texture or automatic context mode.
    pub const EXPOSURE: Self = Self(FFX_API_QUERY_RESOURCE_INPUT_EXPOSURE as u64);
    /// Reactive-mask input.
    pub const REACTIVE_MASK: Self = Self(FFX_API_QUERY_RESOURCE_INPUT_REACTIVEMASK as u64);
    /// Transparency/composition-mask input.
    pub const TRANSPARENCY_AND_COMPOSITION_MASK: Self =
        Self(FFX_API_QUERY_RESOURCE_INPUT_TRANSPARENCYCOMPOSITION as u64);

    const KNOWN_BITS: u64 = Self::COLOR.0
        | Self::DEPTH.0
        | Self::MOTION_VECTORS.0
        | Self::EXPOSURE.0
        | Self::REACTIVE_MASK.0
        | Self::TRANSPARENCY_AND_COMPOSITION_MASK.0;

    /// Complete stored mask, including unknown bits.
    pub fn bits(self) -> u64 {
        self.0
    }

    /// Whether every bit in `resources` is present.
    pub fn contains(self, resources: Self) -> bool {
        self.0 & resources.0 == resources.0
    }

    /// Bits whose meanings are unknown to this wrapper's SDK baseline.
    pub fn unknown_bits(self) -> u64 {
        self.0 & !Self::KNOWN_BITS
    }
}

/// Provider-specific required/optional inputs from a live or pre-create query.
///
/// Requirements describe effect correctness and consumption of optional inputs.
/// A mask absent from both bitfields is not thereby forbidden as a supplied input.
/// They do not expand the wrapper's supported dispatch profile. Unknown and
/// overlapping bits are retained without imposing an undocumented semantic rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceRequirements {
    required: ResourceFlags,
    optional: ResourceFlags,
}

impl ResourceRequirements {
    /// Inputs reported as required for effect correctness.
    pub fn required(&self) -> ResourceFlags {
        self.required
    }

    /// Inputs reported as consumed if supplied, but optional.
    pub fn optional(&self) -> ResourceFlags {
        self.optional
    }
}

#[cfg(any(test, all(windows, feature = "dx12")))]
pub(crate) mod native;

#[cfg(all(test, feature = "utils"))]
mod provider_selection_tests;

impl std::ops::BitOr for ResourceFlags {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl std::ops::BitOrAssign for ResourceFlags {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}
impl std::ops::BitAnd for ResourceFlags {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}
impl std::ops::BitAndAssign for ResourceFlags {
    fn bitand_assign(&mut self, other: Self) {
        self.0 &= other.0;
    }
}
