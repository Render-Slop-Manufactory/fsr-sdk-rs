// SPDX-License-Identifier: MPL-2.0

use super::{Upscaler, UpscalerCore, resources::*};
use crate::{
    context::RuntimeRetention,
    error::{DispatchValidation, Error, InspectionObject, InspectionOperation, ResourceRole},
    upscaler::{
        ReactiveMaskParameters,
        reactive_mask::{build_descriptor, check_region},
    },
};
use fsr_sdk_sys::upscale::ffxDispatchDescUpscaleGenerateReactiveMask;
use windows::{
    Win32::Graphics::{Direct3D12::*, Dxgi::Common::*},
    core::{IUnknown, Interface},
};

/// Named borrowed resources for standalone reactive-mask generation.
///
/// Both colours are maximum-render-size textures of the same frame and signal,
/// differing in translucent content. They may be the same resource. Each
/// independently accepts the colour inputs on [`super::UpscaleResources`],
/// including its fixed typeless views, SNORM/shared-exponent RGB and load-only
/// RGB32F. RGB32F inputs cannot have UAV capability. sRGB views decode RGB;
/// the renderer supplies comparable provider-facing signals across both inputs.
/// UAV-capable typed sRGB inputs require fully typed casting and a legal
/// same-family UNORM UAV, without typed UAV shader access requirements.
/// The caller owns quantization and actual-provider support for this operation;
/// main-upscaling format support does not establish generator support.
/// Output is a distinct, maximum-render-size R8 UNORM or R8 typeless texture
/// with fixed UNORM view, shader-read and UAV capability. The typeless output
/// additionally requires device load/view/store support. All textures are
/// single-sample/mip/slice Texture2D on the context device; the DIRECT list belongs
/// to that device too. No software format/signal conversion occurs.
/// `ReactiveMaskParameters::render_size` selects a nonzero top-left region within those allocations,
/// independently of the main upscaler's dynamic-resolution setting.
///
/// Construction/cloning only copies borrows. It retains no COM
/// objects, allocates no GPU resources and establishes no GPU completion.
#[derive(Debug, Clone)]
pub struct ReactiveMaskResources<'a> {
    color_opaque_only: &'a ID3D12Resource,
    color_pre_upscale: &'a ID3D12Resource,
    output: &'a ID3D12Resource,
}
impl<'a> ReactiveMaskResources<'a> {
    /// Start named assembly with all three required roles unset.
    pub fn builder() -> ReactiveMaskResourcesBuilder<'a> {
        ReactiveMaskResourcesBuilder::default()
    }
    /// Original opaque-only colour borrow.
    pub fn color_opaque_only(&self) -> &'a ID3D12Resource {
        self.color_opaque_only
    }
    /// Original colour including translucent content.
    pub fn color_pre_upscale(&self) -> &'a ID3D12Resource {
        self.color_pre_upscale
    }
    /// Original generated-mask destination borrow.
    pub fn output(&self) -> &'a ID3D12Resource {
        self.output
    }
}
/// Named generator bindings; presence checks only, with no COM/native work.
#[derive(Debug, Clone, Default)]
#[must_use = "resource builders must be used to assemble bindings and build the bundle"]
pub struct ReactiveMaskResourcesBuilder<'a> {
    color_opaque_only: Option<&'a ID3D12Resource>,
    color_pre_upscale: Option<&'a ID3D12Resource>,
    output: Option<&'a ID3D12Resource>,
}
impl<'a> ReactiveMaskResourcesBuilder<'a> {
    /// Bind opaque-only colour; replaces an earlier binding.
    pub fn color_opaque_only(mut self, value: &'a ID3D12Resource) -> Self {
        self.color_opaque_only = Some(value);
        self
    }
    /// Bind pre-upscale colour; replaces an earlier binding.
    pub fn color_pre_upscale(mut self, value: &'a ID3D12Resource) -> Self {
        self.color_pre_upscale = Some(value);
        self
    }
    /// Bind output; replaces an earlier binding.
    pub fn output(mut self, value: &'a ID3D12Resource) -> Self {
        self.output = Some(value);
        self
    }
    /// Check opaque colour, pre-upscale colour and mask output presence in order.
    pub fn build(self) -> Result<ReactiveMaskResources<'a>, Error> {
        Ok(ReactiveMaskResources {
            color_opaque_only: self.color_opaque_only.ok_or(Error::Input(
                crate::error::InputError::MissingBinding {
                    role: ResourceRole::OpaqueOnlyColor,
                },
            ))?,
            color_pre_upscale: self.color_pre_upscale.ok_or(Error::Input(
                crate::error::InputError::MissingBinding {
                    role: ResourceRole::Color,
                },
            ))?,
            output: self
                .output
                .ok_or(Error::Input(crate::error::InputError::MissingBinding {
                    role: ResourceRole::ReactiveMaskOutput,
                }))?,
        })
    }
}
/// Private borrowed generator inspection view; no duplicated parameter storage.
struct ReactiveMaskInputs<'a> {
    command_list: &'a ID3D12GraphicsCommandList,
    resources: &'a ReactiveMaskResources<'a>,
    parameters: &'a crate::upscaler::ReactiveMaskParameters,
}
impl<'a> ReactiveMaskInputs<'a> {
    fn command_list(&self) -> &'a ID3D12GraphicsCommandList {
        self.command_list
    }
    fn color_opaque_only(&self) -> &'a ID3D12Resource {
        self.resources.color_opaque_only()
    }
    fn color_pre_upscale(&self) -> &'a ID3D12Resource {
        self.resources.color_pre_upscale()
    }
    fn output(&self) -> &'a ID3D12Resource {
        self.resources.output()
    }
}
impl std::ops::Deref for ReactiveMaskInputs<'_> {
    type Target = crate::upscaler::ReactiveMaskParameters;
    fn deref(&self) -> &Self::Target {
        self.parameters
    }
}

impl Upscaler {
    /// Inspect generator metadata without recording or calling FidelityFX.
    ///
    /// Uses the recording path's complete validation. Existing poison rejects
    /// before COM inspection; validation never changes eligibility or retains
    /// independent COM references. It may run before opening/transitions of the
    /// list. It cannot check actual GPU states, backing-storage overlap, pixels,
    /// residency, submission/completion or native support. Recording always
    /// inspects again; this optional preflight certifies no provider capability.
    pub fn validate_reactive_mask_inputs(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        resources: &ReactiveMaskResources<'_>,
        parameters: &ReactiveMaskParameters,
    ) -> Result<(), Error> {
        self.core
            .validate_reactive_mask_inputs(command_list, resources, parameters)
    }

    /// Record standalone reactive-mask generation on this context's actual provider.
    ///
    /// Success acknowledges recording only. No main upscale, submission, wait,
    /// history reset or barrier insertion occurs. Resource requirement bits do
    /// not establish generator support and are not used to gate this operation.
    ///
    /// # Input conventions
    ///
    /// Within the provider's supported numerical and access domain, both colours
    /// must match frame, projection/jitter, exposure and encoding, differing only
    /// in the intended translucent contribution. No colour conversion is performed.
    /// Match the consuming upscaler's frame and active region. Restore
    /// application compute bindings/heaps after recording as needed.
    /// These conventions do not establish actual-provider support or access safety.
    ///
    /// # Safety
    ///
    /// Establish the actual provider's support and safe signal/scalar/extent
    /// domain under the runtime acquisition contract.
    /// The DIRECT list must be open, with both colours in
    /// `D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE` and output in
    /// `D3D12_RESOURCE_STATE_UNORDERED_ACCESS`. Output backing storage must not
    /// overlap either input; canonical COM identity checks cannot prove this.
    /// Keep resources/storage resident, list/allocator, context and runtime
    /// valid through GPU completion. Avoid conflicting accesses, premature reuse,
    /// allocator reset and resubmission. Serialize native access and cleanup in
    /// the runtime's common non-reentrant coordination domain.
    ///
    /// Submit each successful recording once in recording order with all other
    /// operations on this context. The trusted provider must restore declared
    /// boundary states on success; this is acquisition trust, not a verified
    /// generator guarantee.
    /// Before consuming output as a supplied reactive mask, order UAV writes
    /// and transition it to shader-read (a UAV barrier then transition is a
    /// conservative sequence).
    /// Across queues, use producer-fence/consumer-wait dependencies. Restore UAV
    /// state for the next generation only after prior readers finish.
    ///
    /// Before any later safe Configure, complete submitted work from both GPU
    /// operations and leave no executable unsubmitted recording on this context.
    /// Moving an owner does not discharge these lasting duties. Before destroy,
    /// Drop or unwinding cleanup, complete earlier submitted work and invalidate
    /// failed/abandoned recordings. A CPU return never proves GPU completion.
    ///
    /// Any native non-OK (including unsupported) preserves
    /// `Operation::GenerateReactiveMask` and its code, poisoning generation,
    /// upscaling, live queries and Configure.
    /// Never submit that failed list: partial commands and unspecified post-states
    /// are possible. Invalidate it, finish prior work and destroy the context.
    /// Abandoning a successful recording likewise requires terminal cleanup.
    ///
    /// ## Native assumptions and uncertainty
    ///
    /// Acquisition's synchronous descriptor/resource non-retention and terminal
    /// destroyability assumptions apply. They protect temporary CPU pointer and
    /// object lifetimes; earlier GPU work may still reference backing storage.
    /// The general Rust/host-memory consequences of GPU-only state, aliasing or
    /// ordering violations are unresolved. All lifetime, residency and
    /// synchronization rules above remain mandatory native safety obligations.
    ///
    /// # Errors
    ///
    /// Native non-OK returns preserve `Operation::GenerateReactiveMask` and the
    /// exact code; generation, upscaling, live queries and Configure become poisoned.
    /// Validation/inspection errors add no generation commands, leave usable
    /// contexts usable and do not undo earlier successful commands. There is
    /// no two-operation transaction, fallback, retry or safe capability probe.
    pub unsafe fn record_reactive_mask(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
        resources: &ReactiveMaskResources<'_>,
        parameters: &ReactiveMaskParameters,
    ) -> Result<(), Error> {
        // SAFETY: Caller establishes provider compatibility and all recording,
        // GPU lifetime/state/ordering and terminal cleanup obligations above.
        unsafe {
            self.core
                .record_reactive_mask(command_list, resources, parameters)
        }
    }
}

impl<R: RuntimeRetention> UpscalerCore<R> {
    pub(in crate::upscaler::dx12) fn validate_reactive_mask_inputs(
        &self,
        command_list: &ID3D12GraphicsCommandList,
        resources: &ReactiveMaskResources<'_>,
        parameters: &ReactiveMaskParameters,
    ) -> Result<(), Error> {
        let dispatch = ReactiveMaskInputs {
            command_list,
            resources,
            parameters,
        };
        self.eligibility.check()?;
        self.inspect_reactive_mask(&dispatch).map(|_| ())
    }

    pub(in crate::upscaler::dx12) unsafe fn record_reactive_mask(
        &mut self,
        command_list: &ID3D12GraphicsCommandList,
        resources: &ReactiveMaskResources<'_>,
        parameters: &ReactiveMaskParameters,
    ) -> Result<(), Error> {
        let dispatch = ReactiveMaskInputs {
            command_list,
            resources,
            parameters,
        };
        let mut eligibility = self.eligibility;
        let result = crate::context_policy::dispatch(
            crate::error::Operation::GenerateReactiveMask,
            &mut eligibility,
            self,
            |this| this.inspect_reactive_mask(&dispatch),
            |this, desc| {
                // SAFETY: Complete temporary descriptor is valid through this
                // synchronous call; caller supplies native/GPU obligations.
                unsafe { this.owner.dispatch(&desc.header) }
            },
        );
        self.eligibility = eligibility;
        result
    }

    fn inspect_reactive_mask(
        &self,
        dispatch: &ReactiveMaskInputs<'_>,
    ) -> Result<ffxDispatchDescUpscaleGenerateReactiveMask, Error> {
        check_region(dispatch.render_size(), self.options.max_render_size())?;
        let list = dispatch.command_list();
        // SAFETY: Inspection of the live borrowed COM list.
        if unsafe { list.GetType() } != D3D12_COMMAND_LIST_TYPE_DIRECT {
            return Err(Error::Input(crate::error::InputError::Dispatch(
                DispatchValidation::CommandListType,
            )));
        }
        let roles = [
            ResourceRole::OpaqueOnlyColor,
            ResourceRole::Color,
            ResourceRole::ReactiveMaskOutput,
        ];
        let objects = [
            (roles[0], Some(dispatch.color_opaque_only())),
            (roles[1], Some(dispatch.color_pre_upscale())),
            (roles[2], Some(dispatch.output())),
        ];
        let mut identities: [Option<IUnknown>; 3] = std::array::from_fn(|_| None);
        for (index, (role, object)) in objects.iter().enumerate() {
            identities[index] = Some(object.unwrap().cast().map_err(|error| {
                inspection(
                    InspectionObject::Resource(*role),
                    InspectionOperation::QueryIdentity,
                    error,
                )
            })?);
        }
        // Read-only inputs may alias; output must have a distinct canonical identity.
        let pointers = identities
            .each_ref()
            .map(|id| id.as_ref().unwrap().as_raw());
        check_identity(
            ResourceRole::ReactiveMaskOutput,
            pointers[2],
            &pointers[..2],
        )?;
        let formats = [
            DXGI_FORMAT_R16G16B16A16_FLOAT,
            DXGI_FORMAT_R16G16B16A16_FLOAT,
            DXGI_FORMAT_R8_UNORM,
        ];
        let convert_at = |index: usize| {
            convert(
                objects[index].1.unwrap(),
                roles[index],
                formats[index],
                index == 2,
            )
        };
        let converted = [convert_at(0)?, convert_at(1)?, convert_at(2)?];
        let maximum = self.options.max_render_size();
        let size = (maximum.width(), maximum.height());
        check_extents(
            roles.into_iter().zip(converted.each_ref().map(|r| r.size)),
            size,
            size,
            size,
        )?;
        let _device_identities = inspect_device_membership(self.owner.device(), list, &objects)?;
        for (role, resource) in roles.into_iter().zip(&converted) {
            inspect_format_support(self.owner.device(), role, resource)?;
        }
        Ok(build_descriptor(
            list.as_raw(),
            converted.map(|r| r.native),
            dispatch.render_size(),
            dispatch.options(),
        ))
    }
}
