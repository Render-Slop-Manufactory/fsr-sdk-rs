// SPDX-License-Identifier: MPL-2.0

//! Experimental acquisition boundary. Never added to the production checkout.
#![allow(missing_docs)]
use super::{LoadError, RuntimeState};
use crate::composition_spike::dx12::{ReactiveMaskResources, UpscaleResources};
use crate::upscaler::dx12::interop::{CreationCallbacks, MessageCallback};
use crate::upscaler::{
    DebugLevel, ID3D12Device, ID3D12GraphicsCommandList, JitterOffsetPixels, TuningParameter,
    TuningValue,
};
use crate::{
    composition_spike::{Config, FrameParameters, ReactiveMaskParameters},
    error::Error,
    query::*,
};
#[cfg(feature = "movable")]
use std::sync::Arc;
use std::{cell::Cell, marker::PhantomData, path::Path, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderSelection {
    Default,
    Explicit(ProviderId),
}
impl ProviderSelection {
    fn optional(self) -> Option<ProviderId> {
        match self {
            Self::Default => None,
            Self::Explicit(id) => Some(id),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeOperation {
    Upscale,
    ReactiveMask,
    Tuning,
    LegacyDebug,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalCause {
    Native {
        operation: NativeOperation,
        code: u32,
    },
    RecordingAbandoned,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextUsability {
    Usable,
    Unusable(TerminalCause),
}
#[derive(Debug)]
pub enum SpikeError {
    Core(Error),
    Unusable(TerminalCause),
}
impl From<Error> for SpikeError {
    fn from(value: Error) -> Self {
        Self::Core(value)
    }
}
impl std::fmt::Display for SpikeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for SpikeError {}

// Concrete expansions preserve the different storage and auto traits. No public
// ownership generic or conversion is introduced by the experiment.
macro_rules! family {
    ($runtime:ident, $setup:ident, $context:ident, $retention:ident, $legacy_runtime:ty, $legacy_context:ty, $bridge:expr) => {
        pub struct $runtime {
            pub(crate) state: $retention<RuntimeState>,
            _not_sync: PhantomData<Cell<()>>,
        }
        pub struct $setup<'runtime, 'device> {
            runtime: &'runtime mut $runtime,
            device: &'device ID3D12Device,
            selection: ProviderSelection,
        }
        pub struct $context {
            inner: $legacy_context,
            status: ContextUsability,
        }
        impl $runtime {
            /// # Safety
            /// Establish the draft's loader-only ABI, initialization, partial
            /// failure, final release and lasting family/thread coordination duties.
            pub unsafe fn load(path: impl AsRef<Path>) -> Result<Self, LoadError> {
                // SAFETY: This private operation loads only the library; it does
                // not invoke effect operations or manufacture an effect owner.
                let state = unsafe { RuntimeState::load_unchecked(path) }?;
                Ok(Self {
                    state: $retention::new(state),
                    _not_sync: PhantomData,
                })
            }
            /// # Safety
            /// Establish every lasting upscaler assumption allocated to setup
            /// in the draft, for all actual providers and derived owners. Movable
            /// use covers every destination thread, not just this borrow's scope.
            pub unsafe fn upscaler_dx12<'r, 'd>(
                &'r mut self,
                device: &'d ID3D12Device,
            ) -> $setup<'r, 'd> {
                $setup {
                    runtime: self,
                    device,
                    selection: ProviderSelection::Default,
                }
            }
        }
        impl $setup<'_, '_> {
            fn bridge(&self) -> $legacy_runtime {
                ($bridge)(self.runtime.state.clone())
            }
            pub fn selection(&self) -> ProviderSelection {
                self.selection
            }
            pub fn with_provider(mut self, selection: ProviderSelection) -> Self {
                self.selection = selection;
                self
            }
            pub fn providers(&mut self) -> Result<Vec<ProviderInfo>, Error> {
                self.bridge().upscaler_providers_dx12(self.device)
            }
            pub fn ratio(&mut self, mode: QualityMode) -> Result<f32, Error> {
                match self.selection {
                    ProviderSelection::Default => {
                        self.bridge().upscaler_ratio_dx12(self.device, mode)
                    }
                    ProviderSelection::Explicit(id) => self
                        .bridge()
                        .upscaler_ratio_for_provider_dx12(self.device, id, mode),
                }
            }
            pub fn render_size_for_quality(
                &mut self,
                output: crate::upscaler::Dimensions,
                mode: QualityMode,
            ) -> Result<crate::upscaler::Dimensions, Error> {
                match self.selection {
                    ProviderSelection::Default => self
                        .bridge()
                        .upscaler_render_size_for_quality_dx12(self.device, output, mode),
                    ProviderSelection::Explicit(id) => self
                        .bridge()
                        .upscaler_render_size_for_quality_for_provider_dx12(
                            self.device,
                            id,
                            output,
                            mode,
                        ),
                }
            }
            pub fn jitter_phase_count(
                &mut self,
                render: crate::upscaler::Dimensions,
                output: crate::upscaler::Dimensions,
            ) -> Result<JitterPhaseCount, Error> {
                match self.selection {
                    ProviderSelection::Default => {
                        self.bridge()
                            .upscaler_jitter_phase_count_dx12(self.device, render, output)
                    }
                    ProviderSelection::Explicit(id) => {
                        self.bridge().upscaler_jitter_phase_count_for_provider_dx12(
                            self.device,
                            id,
                            render,
                            output,
                        )
                    }
                }
            }
            pub fn jitter_offset(
                &mut self,
                index: i32,
                phases: JitterPhaseCount,
            ) -> Result<JitterOffsetPixels, Error> {
                match self.selection {
                    ProviderSelection::Default => {
                        self.bridge()
                            .upscaler_jitter_offset_dx12(self.device, index, phases)
                    }
                    ProviderSelection::Explicit(id) => self
                        .bridge()
                        .upscaler_jitter_offset_for_provider_dx12(self.device, id, index, phases),
                }
            }
            pub fn resource_requirements(&mut self) -> Result<ResourceRequirements, Error> {
                self.bridge()
                    .upscaler_resource_requirements_dx12(self.device, self.selection.optional())
            }
            pub fn estimate_memory(&mut self, config: &Config) -> Result<GpuMemoryUsage, Error> {
                self.bridge().estimate_upscaler_memory_dx12(
                    self.device,
                    &config.selected(self.selection.optional()),
                )
            }
            pub fn create(&mut self, config: &Config) -> Result<$context, Error> {
                let inner = <$legacy_context>::new(
                    &self.bridge(),
                    self.device,
                    config.selected(self.selection.optional()),
                )?;
                Ok($context {
                    inner,
                    status: ContextUsability::Usable,
                })
            }
            /// # Safety
            /// D040's full lasting external callback/allocator contract applies.
            pub unsafe fn create_with_callbacks(
                &mut self,
                config: &Config,
                callbacks: CreationCallbacks,
            ) -> Result<$context, Error> {
                // SAFETY: Setup establishes effect trust; installer establishes
                // lasting callback duties. The production owner retains storage.
                let inner = unsafe {
                    <$legacy_context>::new_with_callbacks(
                        &self.bridge(),
                        self.device,
                        config.selected(self.selection.optional()),
                        callbacks,
                    )
                }?;
                Ok($context {
                    inner,
                    status: ContextUsability::Usable,
                })
            }
            /// # Safety
            /// D040 global installation/coordination duties cover all affected
            /// users, including siblings and users outside this setup.
            pub unsafe fn configure_debug(
                &mut self,
                level: DebugLevel,
                callback: Option<MessageCallback>,
            ) -> Result<(), Error> {
                // SAFETY: Caller covers lasting global effects; setup supplies effect trust.
                unsafe { self.bridge().configure_upscaler_debug(level, callback) }
            }
            /// # Safety
            /// Same lasting global duties; no fallback from the modern route.
            pub unsafe fn configure_global_debug_legacy(
                &mut self,
                level: DebugLevel,
                callback: Option<MessageCallback>,
            ) -> Result<(), Error> {
                // SAFETY: Caller supplies the distinct legacy/global contract.
                unsafe { self.bridge().configure_global_debug_legacy(level, callback) }
            }
        }
        impl $context {
            fn check(&self) -> Result<(), SpikeError> {
                match self.status {
                    ContextUsability::Usable => Ok(()),
                    ContextUsability::Unusable(cause) => Err(SpikeError::Unusable(cause)),
                }
            }
            fn mutated(
                &mut self,
                operation: NativeOperation,
                result: Result<(), Error>,
            ) -> Result<(), SpikeError> {
                if let Err(Error::Native { code, .. }) = result {
                    self.status =
                        ContextUsability::Unusable(TerminalCause::Native { operation, code });
                }
                result.map_err(SpikeError::Core)
            }
            pub fn usability(&self) -> ContextUsability {
                self.status
            }
            pub fn mark_recording_abandoned(&mut self) {
                if self.status == ContextUsability::Usable {
                    self.status = ContextUsability::Unusable(TerminalCause::RecordingAbandoned);
                }
            }
            pub fn validate_upscale_inputs(
                &self,
                list: &ID3D12GraphicsCommandList,
                resources: &UpscaleResources<'_>,
                frame: &FrameParameters,
            ) -> Result<(), SpikeError> {
                self.check()?;
                self.inner
                    .validate_dispatch_inputs(&resources.assemble(list, frame))
                    .map_err(Into::into)
            }
            /// # Safety
            /// The production dispatch contract applies unchanged, including
            /// provider access, states/lifetimes, once-only ordered submission,
            /// failure/abandonment invalidation and completion before Configure/Drop.
            pub unsafe fn record(
                &mut self,
                list: &ID3D12GraphicsCommandList,
                resources: &UpscaleResources<'_>,
                frame: &FrameParameters,
            ) -> Result<(), SpikeError> {
                self.check()?;
                // SAFETY: Caller establishes the unchanged recording duties.
                let result = unsafe { self.inner.dispatch(resources.assemble(list, frame)) };
                self.mutated(NativeOperation::Upscale, result)
            }
            pub fn validate_reactive_mask_inputs(
                &self,
                list: &ID3D12GraphicsCommandList,
                resources: &ReactiveMaskResources<'_>,
                parameters: &ReactiveMaskParameters,
            ) -> Result<(), SpikeError> {
                self.check()?;
                self.inner
                    .validate_generate_reactive_mask(&resources.assemble(list, parameters))
                    .map_err(Into::into)
            }
            /// # Safety
            /// The existing generation contract applies, including output
            /// ordering/transition before consumption and terminal failed lists.
            pub unsafe fn record_reactive_mask(
                &mut self,
                list: &ID3D12GraphicsCommandList,
                resources: &ReactiveMaskResources<'_>,
                parameters: &ReactiveMaskParameters,
            ) -> Result<(), SpikeError> {
                self.check()?;
                // SAFETY: Caller establishes the unchanged generation duties.
                let result = unsafe {
                    self.inner
                        .generate_reactive_mask(resources.assemble(list, parameters))
                };
                self.mutated(NativeOperation::ReactiveMask, result)
            }
            // The request data is deliberately the baseline pair: U09 is outside
            // this U07/early-U08 experiment, but Configure's trust edge is exercised.
            pub fn request_tuning(
                &mut self,
                parameter: TuningParameter,
                value: TuningValue,
            ) -> Result<(), SpikeError> {
                self.check()?;
                let result = self.inner.request_tuning(parameter, value);
                self.mutated(NativeOperation::Tuning, result)
            }
            pub fn provider(&mut self) -> Result<ProviderInfo, SpikeError> {
                self.check()?;
                self.inner.provider().map_err(Into::into)
            }
            pub fn resource_requirements(&mut self) -> Result<ResourceRequirements, SpikeError> {
                self.check()?;
                self.inner.resource_requirements().map_err(Into::into)
            }
            pub fn gpu_memory_usage(&mut self) -> Result<GpuMemoryUsage, SpikeError> {
                self.check()?;
                self.inner.gpu_memory_usage().map_err(Into::into)
            }
            pub fn upscale_ratio(&mut self, mode: QualityMode) -> Result<f32, SpikeError> {
                self.check()?;
                self.inner.upscale_ratio(mode).map_err(Into::into)
            }
            pub fn render_size_for_quality(
                &mut self,
                output: crate::upscaler::Dimensions,
                mode: QualityMode,
            ) -> Result<crate::upscaler::Dimensions, SpikeError> {
                self.check()?;
                self.inner
                    .render_size_for_quality(output, mode)
                    .map_err(Into::into)
            }
            pub fn jitter_phase_count(
                &mut self,
                render: crate::upscaler::Dimensions,
                output: crate::upscaler::Dimensions,
            ) -> Result<JitterPhaseCount, SpikeError> {
                self.check()?;
                self.inner
                    .jitter_phase_count(render, output)
                    .map_err(Into::into)
            }
            pub fn jitter_offset(
                &mut self,
                index: i32,
                phases: JitterPhaseCount,
            ) -> Result<JitterOffsetPixels, SpikeError> {
                self.check()?;
                self.inner.jitter_offset(index, phases).map_err(Into::into)
            }
            /// # Safety
            /// D040's live legacy installation and global coordination duties apply.
            pub unsafe fn configure_global_debug_legacy(
                &mut self,
                level: DebugLevel,
                callback: Option<MessageCallback>,
            ) -> Result<(), SpikeError> {
                self.check()?;
                // SAFETY: Caller supplies live/global callback obligations.
                let result = unsafe { self.inner.configure_global_debug_legacy(level, callback) };
                self.mutated(NativeOperation::LegacyDebug, result)
            }
            pub fn destroy(self) -> Result<(), Error> {
                self.inner.destroy()
            }
        }
    };
}
family!(
    Runtime,
    UpscalerSetup,
    Upscaler,
    Rc,
    super::Runtime,
    crate::upscaler::dx12::Upscaler,
    |state| super::Runtime { state }
);
#[cfg(feature = "movable")]
family!(
    MovableRuntime,
    MovableUpscalerSetup,
    MovableUpscaler,
    Arc,
    super::MovableRuntime,
    crate::upscaler::dx12::MovableUpscaler,
    super::MovableRuntime::spike_from_state
);
