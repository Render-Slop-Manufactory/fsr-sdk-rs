// SPDX-License-Identifier: MPL-2.0

//! Narrow Windows/DX12 dispatch profile for SDK v2.3.0.
use super::Upscaler;
use crate::error::{
    DispatchScalar, DispatchValidation, Error, InspectionObject, InspectionOperation, Operation,
    ResourceProperty, ResourceRole,
};
use fsr_sdk_sys::{
    api::{
        FFX_API_RESOURCE_FLAGS_NONE, FFX_API_RESOURCE_STATE_COMPUTE_READ,
        FFX_API_RESOURCE_STATE_UNORDERED_ACCESS, FFX_API_RESOURCE_TYPE_TEXTURE2D,
        FFX_API_RESOURCE_USAGE_READ_ONLY, FFX_API_RESOURCE_USAGE_UAV, FFX_API_RETURN_OK,
        FFX_API_SURFACE_FORMAT_R16G16_FLOAT, FFX_API_SURFACE_FORMAT_R16G16B16A16_FLOAT,
        FFX_API_SURFACE_FORMAT_R32_FLOAT, FfxApiDimensions2D, FfxApiFloatCoords2D, FfxApiResource,
        FfxApiResourceDepth, FfxApiResourceDescription, FfxApiResourceHeight, FfxApiResourceWidth,
        ffxApiHeader,
    },
    upscale::{FFX_API_DISPATCH_DESC_TYPE_UPSCALE, ffxDispatchDescUpscale},
};
use std::{ffi::c_void, ptr::null_mut};
use windows::{
    Win32::Graphics::{
        Direct3D12::{
            D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_RESOURCE_DESC,
            D3D12_RESOURCE_DIMENSION_TEXTURE2D, D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE, ID3D12Device, ID3D12GraphicsCommandList,
            ID3D12Resource,
        },
        Dxgi::Common::{
            DXGI_FORMAT, DXGI_FORMAT_R16G16_FLOAT, DXGI_FORMAT_R16G16B16A16_FLOAT,
            DXGI_FORMAT_R32_FLOAT,
        },
    },
    core::{IUnknown, Interface},
};

/// Current-frame projection jitter actually applied to rendering, in pixels.
///
/// X and Y describe horizontal and vertical subpixel offsets, not UV or NDC
/// translations. Report the same offset used for both color and depth projection.
/// The wrapper passes both components unchanged: signs must match the renderer's
/// projection conversion, not an assumed sign for a jitter generator. Do not
/// infer jitter signs from the motion vectors' top-down UV convention.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JitterOffsetPixels {
    x: f32,
    y: f32,
}

impl JitterOffsetPixels {
    /// Accepts any two finite pixel offsets, including zero and negative values.
    /// No magnitude limit or jitter sequence is imposed. Non-finite components
    /// return [`Error::InvalidDispatch`] with the corresponding scalar reason.
    pub fn new(x: f32, y: f32) -> Result<Self, Error> {
        if !x.is_finite() {
            return Err(invalid_scalar(DispatchScalar::JitterX));
        }
        if !y.is_finite() {
            return Err(invalid_scalar(DispatchScalar::JitterY));
        }
        Ok(Self { x, y })
    }
    /// Horizontal applied jitter in pixels, with the supplied sign unchanged.
    pub fn x(self) -> f32 {
        self.x
    }
    /// Vertical applied jitter in pixels, with the supplied sign unchanged.
    pub fn y(self) -> f32 {
        self.y
    }
}

/// Elapsed render-frame time in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameTimeMillis(f32);

impl FrameTimeMillis {
    /// Accepts a positive finite elapsed frame time in milliseconds.
    /// Zero, negative, NaN or infinite values return [`Error::InvalidDispatch`].
    pub fn new(milliseconds: f32) -> Result<Self, Error> {
        if !milliseconds.is_finite() || milliseconds <= 0.0 {
            return Err(invalid_scalar(DispatchScalar::FrameTimeMillis));
        }
        Ok(Self(milliseconds))
    }
    /// Elapsed frame time in milliseconds; no seconds conversion is performed.
    pub fn milliseconds(self) -> f32 {
        self.0
    }
}

/// Finite camera parameters for ordinary device depth, without reversed/infinite depth.
///
/// Near and far share the renderer's view-space distance unit. The conversion
/// factor expresses metres per view-space unit; it does not change depth encoding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraParameters {
    near: f32,
    far: f32,
    vertical_fov_radians: f32,
    view_space_to_meters: f32,
}

impl CameraParameters {
    /// Checks the camera domain before native dispatch.
    ///
    /// `near` must be positive and finite, `far` finite and greater than `near`,
    /// vertical FOV finite and strictly between zero and pi radians, and the
    /// metres-per-view-space-unit factor positive and finite. Violations return
    /// [`Error::InvalidDispatch`] identifying the scalar; no native call occurs.
    pub fn new(
        near: f32,
        far: f32,
        vertical_fov_radians: f32,
        view_space_to_meters: f32,
    ) -> Result<Self, Error> {
        if !near.is_finite() || near <= 0.0 {
            return Err(invalid_scalar(DispatchScalar::CameraNear));
        }
        if !far.is_finite() || far <= near {
            return Err(invalid_scalar(DispatchScalar::CameraFar));
        }
        if !vertical_fov_radians.is_finite()
            || vertical_fov_radians <= 0.0
            || vertical_fov_radians >= std::f32::consts::PI
        {
            return Err(invalid_scalar(DispatchScalar::VerticalFovRadians));
        }
        if !view_space_to_meters.is_finite() || view_space_to_meters <= 0.0 {
            return Err(invalid_scalar(DispatchScalar::ViewSpaceToMeters));
        }
        Ok(Self {
            near,
            far,
            vertical_fov_radians,
            view_space_to_meters,
        })
    }
    /// Near-plane distance in the renderer's view-space units.
    pub fn near(self) -> f32 {
        self.near
    }
    /// Far-plane distance in the same view-space units as the near plane.
    pub fn far(self) -> f32 {
        self.far
    }
    /// Vertical field of view in radians, not degrees or horizontal FOV.
    pub fn vertical_fov_radians(self) -> f32 {
        self.vertical_fov_radians
    }
    /// Metres corresponding to one view-space distance unit.
    pub fn view_space_to_meters(self) -> f32 {
        self.view_space_to_meters
    }
}

/// Borrowed inputs for one DX12 upscaler recording.
///
/// Textures must be distinct single-sample, single-mip, single-slice Texture2D
/// resources on the context device. Color is linear, non-HDR, pre-tonemap,
/// non-pre-exposed RGBA16F; depth is ordinary finite device depth (near 0,
/// far 1) in R32F; jitter-free motion vectors are current-to-previous,
/// top-down UV displacement in RG16F at render size. Exposure is a 1×1 R32F
/// texture with a positive finite multiplier also used by subsequent tonemapping.
/// Output is a distinct UAV-capable RGBA16F texture. Inputs match the context's
/// maximum render size; output matches its maximum upscale size. The caller
/// applies `jitter` to color and depth projection and reports that same pixel
/// offset here. Pulse `reset` for the first frame and after cuts or history
/// discontinuities; it does not recover a poisoned context. Use the camera's
/// near/far distances in the same units and a vertical FOV in radians. The
/// caller supplies content consistent with these semantics; the wrapper can inspect
/// descriptions but not pixel values or actual resource states.
///
/// # Shared resource and lifetime requirements
///
/// Every input must enter in `D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE`;
/// output must enter in `D3D12_RESOURCE_STATE_UNORDERED_ACCESS`. The borrow only
/// covers the CPU call. All five textures, their backing storage, the command
/// list/allocator and the upscaler must remain valid for recorded GPU use through
/// completion, with resources resident and backing storage non-overlapping during
/// dispatch. See [`Upscaler::dispatch`] for submission, failure and teardown rules.
pub struct UpscaleDispatch<'a> {
    /// Open, recording DIRECT list on the context device. Dispatch records work;
    /// it does not close, submit or wait. Keep its allocator valid under the
    /// shared lifetime requirements above.
    pub command_list: &'a ID3D12GraphicsCommandList,
    /// Render-size `R16G16B16A16_FLOAT` linear, non-HDR color before tonemapping,
    /// without pre-exposure. Extent equals `max_render_size`; it shares the depth
    /// texture's jittered projection and the input state/lifetime rules above.
    pub color: &'a ID3D12Resource,
    /// Render-size shader-readable `R32_FLOAT` ordinary finite device depth
    /// (near toward zero, far toward one), not linearized, reversed or infinite
    /// depth. Uses the color projection and the shared input state/lifetime rules.
    pub depth: &'a ID3D12Resource,
    /// Render-size `R16G16_FLOAT` current-to-previous normalized UV displacement,
    /// with projection jitter removed. Axes are +X right and +Y down. The wrapper
    /// multiplies UV displacement by render width/height for native pixel scale;
    /// do not supply raw NDC or pixel displacement. Shared input state/lifetime
    /// rules apply; the wrapper cannot inspect vector content.
    pub motion_vectors: &'a ID3D12Resource,
    /// Shader-readable 1×1 `R32_FLOAT` positive finite exposure multiplier, also
    /// used by subsequent tonemapping. Explicit exposure is required; auto
    /// exposure is not enabled. The recorded GPU proof used `1.0`. Shared input
    /// state/lifetime rules apply; pixel validity is the caller's responsibility.
    pub exposure: &'a ID3D12Resource,
    /// Distinct `R16G16B16A16_FLOAT` output at `max_upscale_size`, with UAV
    /// capability and initial `UNORDERED_ACCESS` state. Shared storage, residency
    /// and lifetime rules apply; a distinct COM object does not prove non-aliasing.
    pub output: &'a ID3D12Resource,
    /// Pixel offset actually applied to this frame's color and depth projection.
    pub jitter: JitterOffsetPixels,
    /// Positive finite elapsed render-frame time in milliseconds.
    pub frame_time: FrameTimeMillis,
    /// Finite camera values matching the renderer's ordinary-depth projection.
    pub camera: CameraParameters,
    /// Reset temporal history for the first frame and after camera cuts/history
    /// discontinuities. This is not a fence or recovery from dispatch poison.
    pub reset: bool,
}

fn invalid_scalar(scalar: DispatchScalar) -> Error {
    Error::InvalidDispatch {
        reason: DispatchValidation::Scalar(scalar),
    }
}

fn invalid_resource(role: ResourceRole, property: ResourceProperty) -> Error {
    Error::InvalidDispatch {
        reason: DispatchValidation::Resource { role, property },
    }
}

fn inspection(
    object: InspectionObject,
    operation: InspectionOperation,
    error: windows::core::Error,
) -> Error {
    Error::Dx12Inspection {
        object,
        operation,
        hresult: error.code().0,
    }
}

fn check_identity(
    role: ResourceRole,
    identity: *mut c_void,
    prior: &[*mut c_void],
) -> Result<(), Error> {
    if prior.contains(&identity) {
        Err(invalid_resource(role, ResourceProperty::DistinctIdentity))
    } else {
        Ok(())
    }
}

fn check_extents(
    resources: [(ResourceRole, (u32, u32)); 5],
    render_size: (u32, u32),
    output_size: (u32, u32),
) -> Result<(), Error> {
    for ((role, actual), expected) in
        resources
            .into_iter()
            .zip([render_size, render_size, render_size, (1, 1), output_size])
    {
        if actual != expected {
            return Err(invalid_resource(role, ResourceProperty::Extent));
        }
    }
    Ok(())
}

struct Converted {
    native: FfxApiResource,
    size: (u32, u32),
}

fn format(format: DXGI_FORMAT) -> Option<u32> {
    match format {
        DXGI_FORMAT_R16G16B16A16_FLOAT => Some(FFX_API_SURFACE_FORMAT_R16G16B16A16_FLOAT),
        DXGI_FORMAT_R16G16_FLOAT => Some(FFX_API_SURFACE_FORMAT_R16G16_FLOAT),
        DXGI_FORMAT_R32_FLOAT => Some(FFX_API_SURFACE_FORMAT_R32_FLOAT),
        _ => None,
    }
}

fn convert_desc(
    pointer: *mut c_void,
    desc: D3D12_RESOURCE_DESC,
    role: ResourceRole,
    expected_format: DXGI_FORMAT,
    output: bool,
) -> Result<Converted, Error> {
    if desc.Dimension != D3D12_RESOURCE_DIMENSION_TEXTURE2D
        || desc.DepthOrArraySize != 1
        || desc.MipLevels != 1
        || desc.SampleDesc.Count != 1
    {
        return Err(invalid_resource(role, ResourceProperty::TextureShape));
    }
    if desc.Format != expected_format {
        return Err(invalid_resource(role, ResourceProperty::Format));
    }
    let width =
        u32::try_from(desc.Width).map_err(|_| invalid_resource(role, ResourceProperty::Extent))?;
    if width == 0 || desc.Height == 0 {
        return Err(invalid_resource(role, ResourceProperty::Extent));
    }
    if !output && (desc.Flags.0 & D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE.0) != 0 {
        return Err(invalid_resource(role, ResourceProperty::ShaderReadable));
    }
    let has_uav = (desc.Flags.0 & D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS.0) != 0;
    if output && !has_uav {
        return Err(invalid_resource(role, ResourceProperty::UnorderedAccess));
    }
    let native = FfxApiResource {
        resource: pointer,
        description: FfxApiResourceDescription {
            r#type: FFX_API_RESOURCE_TYPE_TEXTURE2D,
            format: format(desc.Format)
                .ok_or_else(|| invalid_resource(role, ResourceProperty::Format))?,
            width: FfxApiResourceWidth { width },
            height: FfxApiResourceHeight {
                height: desc.Height,
            },
            depth: FfxApiResourceDepth {
                depth: u32::from(desc.DepthOrArraySize),
            },
            mipCount: u32::from(desc.MipLevels),
            flags: FFX_API_RESOURCE_FLAGS_NONE,
            usage: FFX_API_RESOURCE_USAGE_READ_ONLY
                | if has_uav {
                    FFX_API_RESOURCE_USAGE_UAV
                } else {
                    0
                },
        },
        state: if output {
            FFX_API_RESOURCE_STATE_UNORDERED_ACCESS
        } else {
            FFX_API_RESOURCE_STATE_COMPUTE_READ
        },
    };
    Ok(Converted {
        native,
        size: (width, desc.Height),
    })
}

fn convert(
    resource: &ID3D12Resource,
    role: ResourceRole,
    expected_format: DXGI_FORMAT,
    output: bool,
) -> Result<Converted, Error> {
    // SAFETY: A valid COM resource returns its immutable resource description.
    let desc = unsafe { resource.GetDesc() };
    convert_desc(resource.as_raw(), desc, role, expected_format, output)
}

#[cfg(test)]
pub(super) fn resource_for_parity(
    resource: &ID3D12Resource,
    expected_format: DXGI_FORMAT,
    output: bool,
) -> Result<FfxApiResource, Error> {
    Ok(convert(resource, ResourceRole::Color, expected_format, output)?.native)
}

fn null_resource() -> FfxApiResource {
    FfxApiResource {
        resource: null_mut(),
        description: FfxApiResourceDescription {
            r#type: 0,
            format: 0,
            width: FfxApiResourceWidth { width: 0 },
            height: FfxApiResourceHeight { height: 0 },
            depth: FfxApiResourceDepth { depth: 0 },
            mipCount: 0,
            flags: 0,
            usage: 0,
        },
        state: 0,
    }
}

fn build_descriptor(
    command_list: *mut c_void,
    resources: [Converted; 5],
    frame: (JitterOffsetPixels, FrameTimeMillis, CameraParameters, bool),
) -> ffxDispatchDescUpscale {
    let [color, depth, motion, exposure, output] = resources;
    let (jitter, frame_time, camera, reset) = frame;
    ffxDispatchDescUpscale {
        header: ffxApiHeader {
            r#type: FFX_API_DISPATCH_DESC_TYPE_UPSCALE,
            pNext: null_mut(),
        },
        commandList: command_list,
        color: color.native,
        depth: depth.native,
        motionVectors: motion.native,
        exposure: exposure.native,
        reactive: null_resource(),
        transparencyAndComposition: null_resource(),
        output: output.native,
        jitterOffset: FfxApiFloatCoords2D {
            x: jitter.x,
            y: jitter.y,
        },
        motionVectorScale: FfxApiFloatCoords2D {
            x: color.size.0 as f32,
            y: color.size.1 as f32,
        },
        renderSize: FfxApiDimensions2D {
            width: color.size.0,
            height: color.size.1,
        },
        upscaleSize: FfxApiDimensions2D {
            width: output.size.0,
            height: output.size.1,
        },
        enableSharpening: false,
        sharpness: 0.0,
        frameTimeDelta: frame_time.0,
        preExposure: 1.0,
        reset,
        cameraNear: camera.near,
        cameraFar: camera.far,
        cameraFovAngleVertical: camera.vertical_fov_radians,
        viewSpaceToMetersFactor: camera.view_space_to_meters,
        flags: 0,
    }
}

impl Upscaler {
    /// Record one upscaler dispatch on an open DIRECT command list.
    ///
    /// All list/resources must belong to this context's DX12 device. The wrapper
    /// checks device identity, texture metadata, distinct COM identities and
    /// checked scalars before entering native code. It cannot inspect actual
    /// states, backing-heap aliasing, residency, pixel content or GPU completion.
    /// The supported formats and semantics are documented on [`UpscaleDispatch`].
    ///
    /// # Safety
    ///
    /// The list must be open and recording, with its allocator valid through
    /// submission and completion. Color, depth, motion and exposure must be in
    /// `D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE`; output must be in
    /// `D3D12_RESOURCE_STATE_UNORDERED_ACCESS`. Those states are declared to
    /// FidelityFX as compute-read and unordered-access respectively. Keep all
    /// five resources, their backing storage, list/allocator and this upscaler
    /// valid until the submitted GPU work completes, and maintain resource
    /// residency. Backing storage must not overlap during dispatch; the wrapper
    /// checks COM identity, not heap aliasing.
    /// Avoid conflicting access, premature input/output reuse, allocator reset
    /// or list resubmission. Submit each successful recording once and execute
    /// successive dispatches on this context in recording order; use queue/fence
    /// waits for cross-queue dependencies. A shared submission batch alone does
    /// not establish temporal execution order. A CPU return is not GPU completion.
    /// Before [`Self::destroy`] or `Drop`, complete all submitted work on this
    /// context and invalidate failed or abandoned recordings referencing it.
    ///
    /// On success the selected v2.3.0 baseline records restoration to the declared
    /// boundary states by the end of the recorded dispatch. The call may change
    /// command-list descriptor heaps and compute bindings; restore application
    /// bindings afterward.
    /// After a native error, no resource post-state is promised: never submit
    /// that list, invalidate its recording, complete earlier submitted work,
    /// then destroy the poisoned context. If an OK recording is abandoned,
    /// likewise stop dispatching with that context and invalidate it before
    /// destruction. The wrapper cannot observe abandonment. These obligations
    /// rely on bounded allocation trust in the signed v2.3.0 provider under
    /// [D011](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/blob/main/docs/adr/D011.md):
    /// the selected trusted runtime manages its transient allocations under
    /// these known preconditions. This is not an established allocator guarantee
    /// for arbitrary providers or a promised queue depth. Runtime calls remain
    /// confined to the acquiring thread and externally serialized as required by
    /// [`Runtime`](crate::runtime::Runtime).
    ///
    /// # Return value and errors
    ///
    /// `Ok(())` means only that native recording returned OK; it proves neither
    /// submission nor GPU completion or output quality. [`Error::InvalidDispatch`]
    /// and [`Error::Dx12Inspection`] occur before native dispatch, record no FSR
    /// commands in this call and leave a usable context unpoisoned. They do not
    /// undo the renderer's earlier commands. A non-OK native return preserves its
    /// code in [`Error::Native`] and poisons later dispatch, which then returns
    /// [`Error::DispatchPoisoned`] without FFI. Native faults need not return a
    /// Rust error. Teardown remains available under the lifetime rules above.
    pub unsafe fn dispatch(&mut self, dispatch: UpscaleDispatch<'_>) -> Result<(), Error> {
        // SAFETY: The caller accepted this method's GPU lifetime and state
        // obligations. Inspection runs before the native entry point.
        unsafe { self.dispatch_core(|this| this.inspect_dispatch(dispatch)) }
    }

    fn inspect_dispatch(
        &self,
        dispatch: UpscaleDispatch<'_>,
    ) -> Result<ffxDispatchDescUpscale, Error> {
        let list = dispatch.command_list;
        // SAFETY: GetType inspects the live command-list COM object.
        if unsafe { list.GetType() } != D3D12_COMMAND_LIST_TYPE_DIRECT {
            return Err(Error::InvalidDispatch {
                reason: DispatchValidation::CommandListType,
            });
        }
        let objects = [
            (ResourceRole::Color, dispatch.color),
            (ResourceRole::Depth, dispatch.depth),
            (ResourceRole::MotionVectors, dispatch.motion_vectors),
            (ResourceRole::Exposure, dispatch.exposure),
            (ResourceRole::Output, dispatch.output),
        ];
        let mut identities: [Option<IUnknown>; 5] = std::array::from_fn(|_| None);
        let mut identity_pointers = [std::ptr::null_mut(); 5];
        for (index, (role, object)) in objects.into_iter().enumerate() {
            let identity: IUnknown = object.cast().map_err(|error| {
                inspection(
                    InspectionObject::Resource(role),
                    InspectionOperation::QueryIdentity,
                    error,
                )
            })?;
            check_identity(role, identity.as_raw(), &identity_pointers[..index])?;
            identity_pointers[index] = identity.as_raw();
            identities[index] = Some(identity);
        }
        let color = convert(
            dispatch.color,
            ResourceRole::Color,
            DXGI_FORMAT_R16G16B16A16_FLOAT,
            false,
        )?;
        let depth = convert(
            dispatch.depth,
            ResourceRole::Depth,
            DXGI_FORMAT_R32_FLOAT,
            false,
        )?;
        let motion = convert(
            dispatch.motion_vectors,
            ResourceRole::MotionVectors,
            DXGI_FORMAT_R16G16_FLOAT,
            false,
        )?;
        let exposure = convert(
            dispatch.exposure,
            ResourceRole::Exposure,
            DXGI_FORMAT_R32_FLOAT,
            false,
        )?;
        let output = convert(
            dispatch.output,
            ResourceRole::Output,
            DXGI_FORMAT_R16G16B16A16_FLOAT,
            true,
        )?;
        let render_size = (
            self.options.max_render_size.width.get(),
            self.options.max_render_size.height.get(),
        );
        let output_size = (
            self.options.max_upscale_size.width.get(),
            self.options.max_upscale_size.height.get(),
        );
        check_extents(
            [
                (ResourceRole::Color, color.size),
                (ResourceRole::Depth, depth.size),
                (ResourceRole::MotionVectors, motion.size),
                (ResourceRole::Exposure, exposure.size),
                (ResourceRole::Output, output.size),
            ],
            render_size,
            output_size,
        )?;

        let context_device: IUnknown = self.owner.device().cast().map_err(|error| {
            inspection(
                InspectionObject::ContextDevice,
                InspectionOperation::QueryIdentity,
                error,
            )
        })?;
        let mut list_device: Option<ID3D12Device> = None;
        // SAFETY: GetDevice writes a fresh COM reference into this output.
        unsafe { list.GetDevice(&mut list_device) }.map_err(|error| {
            inspection(
                InspectionObject::CommandList,
                InspectionOperation::GetDevice,
                error,
            )
        })?;
        let list_identity: IUnknown = list_device
            .ok_or(Error::InvalidDispatch {
                reason: DispatchValidation::CommandListDevice,
            })?
            .cast()
            .map_err(|error| {
                inspection(
                    InspectionObject::CommandList,
                    InspectionOperation::QueryIdentity,
                    error,
                )
            })?;
        if list_identity.as_raw() != context_device.as_raw() {
            return Err(Error::InvalidDispatch {
                reason: DispatchValidation::CommandListDevice,
            });
        }
        for (role, object) in objects {
            let mut device: Option<ID3D12Device> = None;
            // SAFETY: GetDevice writes a fresh COM reference into this output.
            unsafe { object.GetDevice(&mut device) }.map_err(|error| {
                inspection(
                    InspectionObject::Resource(role),
                    InspectionOperation::GetDevice,
                    error,
                )
            })?;
            let identity: IUnknown = device
                .ok_or_else(|| invalid_resource(role, ResourceProperty::Device))?
                .cast()
                .map_err(|error| {
                    inspection(
                        InspectionObject::Resource(role),
                        InspectionOperation::QueryIdentity,
                        error,
                    )
                })?;
            if identity.as_raw() != context_device.as_raw() {
                return Err(invalid_resource(role, ResourceProperty::Device));
            }
        }

        Ok(build_descriptor(
            list.as_raw(),
            [color, depth, motion, exposure, output],
            (
                dispatch.jitter,
                dispatch.frame_time,
                dispatch.camera,
                dispatch.reset,
            ),
        ))
    }

    /// # Safety
    /// The inspected descriptor must satisfy this module's dispatch contract
    /// and the caller must uphold GPU lifetime, state and ordering obligations.
    pub(super) unsafe fn dispatch_core(
        &mut self,
        inspect: impl FnOnce(&Self) -> Result<ffxDispatchDescUpscale, Error>,
    ) -> Result<(), Error> {
        if self.dispatch_poisoned {
            return Err(Error::DispatchPoisoned);
        }
        let desc = inspect(self)?;
        // SAFETY: All descriptor storage lives through the call; the private
        // owner retains the context/runtime. The unsafe caller fulfills the
        // resource-state and post-return GPU obligations described above.
        let code = unsafe { self.owner.dispatch(&desc.header) };
        if code == FFX_API_RETURN_OK {
            Ok(())
        } else {
            self.dispatch_poisoned = true;
            Err(Error::Native {
                operation: Operation::Dispatch,
                code,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::Graphics::Direct3D12::D3D12_RESOURCE_FLAG_NONE;
    use windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC;

    fn desc(format: DXGI_FORMAT, width: u64, height: u32, output: bool) -> D3D12_RESOURCE_DESC {
        D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            Width: width,
            Height: height,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: format,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Flags: if output {
                D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS
            } else {
                D3D12_RESOURCE_FLAG_NONE
            },
            ..Default::default()
        }
    }

    #[test]
    fn narrow_conversion_and_descriptor() {
        let make = |pointer, format, width, height, output| {
            convert_desc(
                pointer as *mut c_void,
                desc(format, width, height, output),
                ResourceRole::Color,
                format,
                output,
            )
            .unwrap()
        };
        let color = make(1, DXGI_FORMAT_R16G16B16A16_FLOAT, 320, 180, false);
        let depth = make(2, DXGI_FORMAT_R32_FLOAT, 320, 180, false);
        let motion = make(3, DXGI_FORMAT_R16G16_FLOAT, 320, 180, false);
        let exposure = make(4, DXGI_FORMAT_R32_FLOAT, 1, 1, false);
        let output = make(5, DXGI_FORMAT_R16G16B16A16_FLOAT, 640, 360, true);
        let result = build_descriptor(
            6usize as *mut c_void,
            [color, depth, motion, exposure, output],
            (
                JitterOffsetPixels::new(0.25, -0.25).unwrap(),
                FrameTimeMillis::new(16.667).unwrap(),
                CameraParameters::new(0.1, 100.0, 1.0, 1.0).unwrap(),
                true,
            ),
        );
        assert_eq!(result.header.r#type, FFX_API_DISPATCH_DESC_TYPE_UPSCALE);
        assert!(result.header.pNext.is_null());
        assert_eq!(result.commandList as usize, 6);
        for (actual, expected) in [
            (result.color, 1),
            (result.depth, 2),
            (result.motionVectors, 3),
            (result.exposure, 4),
            (result.output, 5),
        ] {
            assert_eq!(actual.resource as usize, expected);
        }
        assert!(result.reactive.resource.is_null());
        assert!(result.transparencyAndComposition.resource.is_null());
        assert_eq!(
            result.color.description.format,
            FFX_API_SURFACE_FORMAT_R16G16B16A16_FLOAT
        );
        assert_eq!(
            result.depth.description.format,
            FFX_API_SURFACE_FORMAT_R32_FLOAT
        );
        assert_eq!(
            result.motionVectors.description.format,
            FFX_API_SURFACE_FORMAT_R16G16_FLOAT
        );
        assert_eq!(result.output.description.usage, FFX_API_RESOURCE_USAGE_UAV);
        assert_eq!(result.output.state, FFX_API_RESOURCE_STATE_UNORDERED_ACCESS);
        assert_eq!(result.color.state, FFX_API_RESOURCE_STATE_COMPUTE_READ);
        assert_eq!(
            (result.renderSize.width, result.renderSize.height),
            (320, 180)
        );
        assert_eq!(
            (result.upscaleSize.width, result.upscaleSize.height),
            (640, 360)
        );
        assert_eq!(
            (result.motionVectorScale.x, result.motionVectorScale.y),
            (320.0, 180.0)
        );
        assert_eq!(
            (result.jitterOffset.x, result.jitterOffset.y),
            (0.25, -0.25)
        );
        assert_eq!(result.frameTimeDelta, 16.667);
        assert_eq!(result.preExposure, 1.0);
        assert!(result.reset);
        assert!(!result.enableSharpening);
        assert_eq!(result.sharpness, 0.0);
        assert_eq!(result.flags, 0);
    }

    #[test]
    fn rejects_invalid_resources_and_frame_before_native_call() {
        let valid = desc(DXGI_FORMAT_R16G16B16A16_FLOAT, 640, 360, true);
        let no_uav = desc(DXGI_FORMAT_R16G16B16A16_FLOAT, 640, 360, false);
        let check = |resource_desc| {
            convert_desc(
                null_mut(),
                resource_desc,
                ResourceRole::Output,
                DXGI_FORMAT_R16G16B16A16_FLOAT,
                true,
            )
            .err()
            .unwrap()
        };
        assert_eq!(
            check(no_uav),
            invalid_resource(ResourceRole::Output, ResourceProperty::UnorderedAccess)
        );
        let mut multisampled = valid;
        multisampled.SampleDesc.Count = 2;
        assert_eq!(
            check(multisampled),
            invalid_resource(ResourceRole::Output, ResourceProperty::TextureShape)
        );
        let mut array = valid;
        array.DepthOrArraySize = 2;
        assert_eq!(
            check(array),
            invalid_resource(ResourceRole::Output, ResourceProperty::TextureShape)
        );
        let mut mips = valid;
        mips.MipLevels = 2;
        assert_eq!(
            check(mips),
            invalid_resource(ResourceRole::Output, ResourceProperty::TextureShape)
        );
        let mut wrong_format = valid;
        wrong_format.Format = DXGI_FORMAT_R32_FLOAT;
        assert_eq!(
            check(wrong_format),
            invalid_resource(ResourceRole::Output, ResourceProperty::Format)
        );
        let mut oversized = valid;
        oversized.Width = u64::from(u32::MAX) + 1;
        assert_eq!(
            check(oversized),
            invalid_resource(ResourceRole::Output, ResourceProperty::Extent)
        );
        let mut zero = valid;
        zero.Height = 0;
        assert_eq!(
            check(zero),
            invalid_resource(ResourceRole::Output, ResourceProperty::Extent)
        );
        let mut denied = desc(DXGI_FORMAT_R32_FLOAT, 320, 180, false);
        denied.Flags = D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE;
        assert_eq!(
            convert_desc(
                null_mut(),
                denied,
                ResourceRole::Depth,
                DXGI_FORMAT_R32_FLOAT,
                false
            )
            .err()
            .unwrap(),
            invalid_resource(ResourceRole::Depth, ResourceProperty::ShaderReadable)
        );
        assert_eq!(
            JitterOffsetPixels::new(f32::NAN, 0.0),
            Err(invalid_scalar(DispatchScalar::JitterX))
        );
        assert_eq!(
            JitterOffsetPixels::new(0.0, f32::INFINITY),
            Err(invalid_scalar(DispatchScalar::JitterY))
        );
        for invalid in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(
                FrameTimeMillis::new(invalid),
                Err(invalid_scalar(DispatchScalar::FrameTimeMillis))
            );
        }
        assert_eq!(
            CameraParameters::new(0.0, 100.0, 1.0, 1.0),
            Err(invalid_scalar(DispatchScalar::CameraNear))
        );
        assert_eq!(
            CameraParameters::new(0.1, 0.1, 1.0, 1.0),
            Err(invalid_scalar(DispatchScalar::CameraFar))
        );
        assert_eq!(
            CameraParameters::new(0.1, 100.0, std::f32::consts::PI, 1.0),
            Err(invalid_scalar(DispatchScalar::VerticalFovRadians))
        );
        assert_eq!(
            CameraParameters::new(0.1, 100.0, 1.0, 0.0),
            Err(invalid_scalar(DispatchScalar::ViewSpaceToMeters))
        );
        let jitter = JitterOffsetPixels::new(0.25, -0.25).unwrap();
        assert_eq!((jitter.x(), jitter.y()), (0.25, -0.25));
        assert_eq!(FrameTimeMillis::new(16.667).unwrap().milliseconds(), 16.667);
        let camera = CameraParameters::new(0.1, 100.0, 1.0, 1.0).unwrap();
        assert_eq!(
            (
                camera.near(),
                camera.far(),
                camera.vertical_fov_radians(),
                camera.view_space_to_meters()
            ),
            (0.1, 100.0, 1.0, 1.0)
        );
    }

    #[test]
    fn exact_extents_and_canonical_identity_reject_before_native_call() {
        let roles = [
            ResourceRole::Color,
            ResourceRole::Depth,
            ResourceRole::MotionVectors,
            ResourceRole::Exposure,
            ResourceRole::Output,
        ];
        let valid = [(320, 180), (320, 180), (320, 180), (1, 1), (640, 360)];
        let inputs =
            |sizes: [(u32, u32); 5]| std::array::from_fn(|index| (roles[index], sizes[index]));
        assert_eq!(check_extents(inputs(valid), (320, 180), (640, 360)), Ok(()));
        for index in 0..5 {
            let mut sizes = valid;
            sizes[index].0 -= 1;
            assert_eq!(
                check_extents(inputs(sizes), (320, 180), (640, 360)),
                Err(invalid_resource(roles[index], ResourceProperty::Extent))
            );
        }
        let mut first = 0u8;
        let mut second = 0u8;
        let pointers = [
            (&raw mut first).cast::<c_void>(),
            (&raw mut second).cast::<c_void>(),
        ];
        assert_eq!(
            check_identity(ResourceRole::Depth, pointers[1], &pointers[..1]),
            Ok(())
        );
        assert_eq!(
            check_identity(ResourceRole::Output, pointers[0], &pointers),
            Err(invalid_resource(
                ResourceRole::Output,
                ResourceProperty::DistinctIdentity
            ))
        );
    }
}
