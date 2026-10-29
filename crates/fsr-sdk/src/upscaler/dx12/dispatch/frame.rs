// SPDX-License-Identifier: MPL-2.0

//! Named borrowed DX12 resources; assembling them performs no COM/native work.
use crate::{
    error::{Error, ResourceRole},
    upscaler::FrameParameters,
};
use windows::Win32::Graphics::Direct3D12::{ID3D12GraphicsCommandList, ID3D12Resource};

/// Required exposure binding: an explicit texture or the context's automatic mode.
/// Omission from a builder is an error, never an implicit automatic request.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum ExposureInput<'a> {
    /// Shader-readable 1x1 exposure multiplier for an explicit-exposure context.
    Texture(&'a ID3D12Resource),
    /// Omit the texture for a context created with automatic exposure.
    Automatic,
}
impl<'a> ExposureInput<'a> {
    /// Inspect the texture choice without inspection or independent COM retention.
    pub fn texture(self) -> Option<&'a ID3D12Resource> {
        match self {
            Self::Texture(value) => Some(value),
            Self::Automatic => None,
        }
    }
}

/// Borrowed textures for one DX12 upscaler recording.
///
/// Construction and cloning only assemble borrows: they perform no COM inspection,
/// native calls or independent COM retention. Resource compatibility is checked
/// by optional `validate_upscale_inputs` and mandatory `record` inspection;
/// contents, states, storage and GPU lifetimes remain caller
/// obligations described on [`Upscaler::record`](super::Upscaler::record).
///
/// Colour and output independently accept `R16G16B16A16_FLOAT`,
/// `R32G32B32A32_FLOAT`, `R11G11B10_FLOAT`, `R10G10B10A2_UNORM`,
/// `R8G8B8A8_UNORM`, `R8G8B8A8_SNORM`, `B8G8R8A8_UNORM`,
/// `R8G8B8A8_UNORM_SRGB`, `B8G8R8A8_UNORM_SRGB` or `R9G9B9E5_SHAREDEXP`.
/// Colour additionally accepts load-only `R32G32B32_FLOAT`; it cannot have UAV
/// capability or serve as output. The selected provider must support load-only
/// input. Device bits do not certify provider support for any role.
///
/// Typeless colour allocations `R32G32B32A32_TYPELESS`, `R16G16B16A16_TYPELESS`,
/// `R32G32B32_TYPELESS`, `R10G10B10A2_TYPELESS`, `R8G8B8A8_TYPELESS` and
/// `B8G8R8A8_TYPELESS` use fixed FLOAT views for float families and UNORM views
/// otherwise. RGB32 typeless is also input-only. No caller view override exists.
/// Motion accepts RG16F/RG32F typed or typeless allocations, with FLOAT views.
/// Explicit exposure accepts R32F or R32 typeless without depth/stencil capability.
/// Supplied masks and generator output accept R8 UNORM or R8 typeless, with
/// UNORM views. Typeless does not select another signal encoding.
///
/// Depth accepts `R32_FLOAT`, `R16_FLOAT`, `R16_UNORM` and ordinary
/// `R32_TYPELESS`/`R16_TYPELESS` FLOAT views. R16 typeless must not have
/// depth/stencil capability: the SDK cannot select its D16 UNORM interpretation.
/// Depth-buffer allocations additionally accept `R32_TYPELESS`, `D32_FLOAT`,
/// `R32G8X24_TYPELESS`, `D32_FLOAT_S8X24_UINT`, `D16_UNORM`,
/// `R24G8_TYPELESS` and `D24_UNORM_S8_UINT`, with depth/stencil capability,
/// no UAV or shader-read denial. Fixed depth SRVs are R32F for D32,
/// `R32_FLOAT_X8X24_TYPELESS` for D32S8, R16 UNORM for D16 and
/// `R24_UNORM_X8_TYPELESS` for D24; stencil is ignored. Typed depth-buffer
/// allocations require fully typed casting support. Callers transition both
/// depth/stencil planes to shader-read and own precision/range suitability.
///
/// Typed sRGB allocations use sRGB SRVs (RGB decoding, unchanged alpha) and,
/// when UAV-capable, same-family UNORM UAVs. The latter require fully typed
/// casting support even for read-only inputs. Output stores do not automatically
/// sRGB-encode. BGRA views expose logical RGB. Typeless 8-bit colour uses UNORM,
/// never sRGB. Classification/hints remain explicit and independent of storage.
///
/// Additional formats require role-specific device support, including optional
/// typed UAV stores for shared-exponent/BGRA output. RGB-only formats have no
/// alpha; unsigned formats cannot represent negative values. Normalized storage
/// quantizes/clamps to its range. The renderer supplies compatible signals and
/// establishes actual-provider support. The wrapper performs no software format
/// or signal conversion and promises no alpha preservation or tone mapping.
#[derive(Debug, Clone)]
pub struct UpscaleResources<'a> {
    color: &'a ID3D12Resource,
    depth: &'a ID3D12Resource,
    motion_vectors: &'a ID3D12Resource,
    exposure: ExposureInput<'a>,
    reactive_mask: Option<&'a ID3D12Resource>,
    transparency_and_composition_mask: Option<&'a ID3D12Resource>,
    output: &'a ID3D12Resource,
}

impl<'a> UpscaleResources<'a> {
    /// Start named assembly with every required binding unset.
    pub fn builder() -> UpscaleResourcesBuilder<'a> {
        UpscaleResourcesBuilder::default()
    }
    /// Render-size scene colour in one of this bundle's accepted formats.
    /// In the default linear profile, RGB is
    /// multiplied by the frame's [`PreExposure`](crate::upscaler::PreExposure) factor before dispatch.
    /// Non-linear signals require a provider-established encoding/exposure contract.
    /// HDR content requires the
    /// context's [`UpscalerOptions::high_dynamic_range`](crate::upscaler::UpscalerOptions::high_dynamic_range) option.
    /// Must share the depth projection and enter shader-readable as documented
    /// on [`Upscaler::record`](super::Upscaler::record).
    pub fn color(&self) -> &'a ID3D12Resource {
        self.color
    }

    /// Render-size device depth in an accepted format, matching the context's
    /// [`DepthConvention`](crate::upscaler::DepthConvention).
    /// Linearized depth is outside the supported profile.
    /// Stencil is ignored. The renderer owns DSV production and must transition
    /// the whole depth/stencil resource to shader-read before dispatch.
    pub fn depth(&self) -> &'a ID3D12Resource {
        self.depth
    }

    /// RG16F or RG32F motion (typed or fixed-view typeless allocations) at the context's selected render/display
    /// resolution, containing or excluding jitter according to its options.
    /// For UV/pixel scaling, axes are +X right/+Y down and values use this frame's
    /// [`MotionVectorScale`](crate::upscaler::MotionVectorScale): UV by default, or pixels. With literal Native scaling,
    /// stored components must encode the actual provider's current-to-previous
    /// displacement after native scaling. NDC is not converted automatically.
    /// RG32F also requires device SRV/view support; that is no provider certificate.
    pub fn motion_vectors(&self) -> &'a ID3D12Resource {
        self.motion_vectors
    }

    /// Shader-readable 1×1 R32F positive finite exposure multiplier, using a typed
    /// or fixed-view typeless allocation, also used by subsequent tonemapping.
    /// The recorded GPU profile used `1.0`.
    ///
    /// Returns `ExposureInput::Automatic` for automatic exposure. Internal SDK exposure is not
    /// exposed to the renderer as a texture or a tonemapping value.
    pub fn exposure(&self) -> ExposureInput<'a> {
        self.exposure
    }

    /// Optional R8 UNORM (typed or typeless) reactive mask controlling reliance on pixel history.
    pub fn reactive_mask(&self) -> Option<&'a ID3D12Resource> {
        self.reactive_mask
    }

    /// Supply or omit the renderer's reactive mask, without inspecting it.
    ///
    /// The texture must match the maximum render allocation, with jitter-aligned
    /// content in the same top-left active region as colour/depth. UNORM values
    /// represent reactivity in `[0, 1]`; no wrapper scaling or clamping occurs.
    /// Dispatch checks provider requirements and metadata. Keep the distinct,
    /// shader-readable resource alive through GPU completion, like other inputs.
    pub fn set_reactive_mask(&mut self, value: Option<&'a ID3D12Resource>) {
        self.reactive_mask = value;
    }

    /// Optional R8 UNORM (typed or typeless) transparency/composition mask for history protection.
    pub fn transparency_and_composition_mask(&self) -> Option<&'a ID3D12Resource> {
        self.transparency_and_composition_mask
    }

    /// Supply or omit the renderer's transparency/composition mask.
    ///
    /// Allocation, active-region, jitter, state and lifetime requirements match
    /// [`Self::set_reactive_mask`]. Values describe the renderer's history
    /// protection input, independently of reactivity. This setter only stores
    /// the borrow; it generates no pixels and performs no native call.
    pub fn set_transparency_and_composition_mask(&mut self, value: Option<&'a ID3D12Resource>) {
        self.transparency_and_composition_mask = value;
    }

    /// Distinct destination in an accepted colour format at maximum output size,
    /// with UAV capability and initial `UNORDERED_ACCESS` state.
    /// Distinct COM identity does not prove non-overlapping backing storage.
    pub fn output(&self) -> &'a ID3D12Resource {
        self.output
    }
}

/// Named assembly of required texture roles and explicit exposure intent.
/// Build checks presence only, in colour/depth/motion/exposure/output order.
/// It never inspects metadata, retains COM objects or establishes GPU lifetime.
#[derive(Debug, Clone, Default)]
#[must_use = "resource builders must be used to assemble bindings and build the bundle"]
pub struct UpscaleResourcesBuilder<'a> {
    color: Option<&'a ID3D12Resource>,
    depth: Option<&'a ID3D12Resource>,
    motion_vectors: Option<&'a ID3D12Resource>,
    exposure: Option<ExposureInput<'a>>,
    output: Option<&'a ID3D12Resource>,
    reactive_mask: Option<&'a ID3D12Resource>,
    transparency_and_composition_mask: Option<&'a ID3D12Resource>,
}
impl<'a> UpscaleResourcesBuilder<'a> {
    /// Bind the colour role; replaces an earlier binding.
    pub fn color(mut self, value: &'a ID3D12Resource) -> Self {
        self.color = Some(value);
        self
    }
    /// Bind the depth role; replaces an earlier binding.
    pub fn depth(mut self, value: &'a ID3D12Resource) -> Self {
        self.depth = Some(value);
        self
    }
    /// Bind the motion vectors role; replaces an earlier binding.
    pub fn motion_vectors(mut self, value: &'a ID3D12Resource) -> Self {
        self.motion_vectors = Some(value);
        self
    }
    /// Bind the exposure role; replaces an earlier binding.
    pub fn exposure(mut self, value: ExposureInput<'a>) -> Self {
        self.exposure = Some(value);
        self
    }
    /// Bind the output role; replaces an earlier binding.
    pub fn output(mut self, value: &'a ID3D12Resource) -> Self {
        self.output = Some(value);
        self
    }
    /// Bind or clear the optional reactive mask without inspection.
    pub fn reactive_mask(mut self, value: Option<&'a ID3D12Resource>) -> Self {
        self.reactive_mask = value;
        self
    }
    /// Bind or clear the optional transparency and composition mask without inspection.
    pub fn transparency_and_composition_mask(mut self, value: Option<&'a ID3D12Resource>) -> Self {
        self.transparency_and_composition_mask = value;
        self
    }
    /// Require every mandatory role. `Error::Input` is pure input rejection.
    pub fn build(self) -> Result<UpscaleResources<'a>, Error> {
        Ok(UpscaleResources {
            color: self
                .color
                .ok_or(Error::Input(crate::error::InputError::MissingBinding {
                    role: ResourceRole::Color,
                }))?,
            depth: self
                .depth
                .ok_or(Error::Input(crate::error::InputError::MissingBinding {
                    role: ResourceRole::Depth,
                }))?,
            motion_vectors: self.motion_vectors.ok_or(Error::Input(
                crate::error::InputError::MissingBinding {
                    role: ResourceRole::MotionVectors,
                },
            ))?,
            exposure: self.exposure.ok_or(Error::Input(
                crate::error::InputError::MissingBinding {
                    role: ResourceRole::Exposure,
                },
            ))?,
            output: self
                .output
                .ok_or(Error::Input(crate::error::InputError::MissingBinding {
                    role: ResourceRole::Output,
                }))?,
            reactive_mask: self.reactive_mask,
            transparency_and_composition_mask: self.transparency_and_composition_mask,
        })
    }
}

/// Private borrowed inspection view, with no copied parameters or defaults.
pub(super) struct UpscaleInputs<'a> {
    pub(super) command_list: &'a ID3D12GraphicsCommandList,
    pub(super) resources: &'a UpscaleResources<'a>,
    pub(super) parameters: &'a FrameParameters,
}
impl<'a> UpscaleInputs<'a> {
    pub(super) fn new(
        command_list: &'a ID3D12GraphicsCommandList,
        resources: &'a UpscaleResources<'a>,
        parameters: &'a FrameParameters,
    ) -> Self {
        Self {
            command_list,
            resources,
            parameters,
        }
    }
    pub(super) fn command_list(&self) -> &'a ID3D12GraphicsCommandList {
        self.command_list
    }
    pub(super) fn resources(&self) -> &'a UpscaleResources<'a> {
        self.resources
    }
}
impl std::ops::Deref for UpscaleInputs<'_> {
    type Target = FrameParameters;
    fn deref(&self) -> &Self::Target {
        self.parameters
    }
}
