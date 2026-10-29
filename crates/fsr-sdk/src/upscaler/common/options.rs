// SPDX-License-Identifier: MPL-2.0

//! Creation options, native flags and context-relative input checks.

#[cfg(any(test, all(windows, feature = "dx12")))]
use super::ColorEncodingHint;
use super::{
    DepthConvention, Dimensions, ExposureMode, InputColorSpace, MotionVectorJitter,
    MotionVectorResolution,
};
#[cfg(any(test, all(windows, feature = "dx12")))]
use crate::error::{DispatchValidation, Error};

/// Allocation maxima, colour, depth, motion and exposure profile for an upscaler.
///
/// Textures retain these allocation extents. Dynamic resolution optionally permits
/// smaller active render and output regions inside those allocations. The maximum
/// output allocation stays fixed; the active output region may vary.
/// All profile selections are fixed at creation. Linear input is the default;
/// non-linear input and HDR are independent opt-ins.
#[derive(Debug, Clone)]
pub struct UpscalerOptions {
    max_render_size: Dimensions,
    max_upscale_size: Dimensions,
    depth_convention: DepthConvention,
    motion_vector_resolution: MotionVectorResolution,
    motion_vector_jitter: MotionVectorJitter,
    exposure_mode: ExposureMode,
    dynamic_resolution: bool,
    high_dynamic_range: bool,
    input_color_space: InputColorSpace,
    debug_checking: bool,
    debug_visualization: bool,
}

impl UpscalerOptions {
    /// Sets the checked render and output maxima for the fixed upscaler profile.
    ///
    /// Defaults are standard finite depth and render-resolution jitter-free motion.
    /// Exposure defaults to explicit; HDR and dynamic resolution are not enabled.
    /// Clone for explicit reuse.
    pub fn new(max_render_size: Dimensions, max_upscale_size: Dimensions) -> Self {
        Self {
            max_render_size,
            max_upscale_size,
            depth_convention: DepthConvention::default(),
            motion_vector_resolution: MotionVectorResolution::default(),
            motion_vector_jitter: MotionVectorJitter::default(),
            exposure_mode: ExposureMode::default(),
            dynamic_resolution: false,
            high_dynamic_range: false,
            input_color_space: InputColorSpace::default(),
            debug_checking: false,
            debug_visualization: false,
        }
    }

    /// Whether native input checking is requested. Does not install a message sink.
    pub fn debug_checking(&self) -> bool {
        self.debug_checking
    }
    /// Request native input checking, independently of global message filtering.
    pub fn set_debug_checking(&mut self, value: bool) {
        self.debug_checking = value;
    }
    /// Whether creation permits native debug-visualization resources.
    pub fn debug_visualization(&self) -> bool {
        self.debug_visualization
    }
    /// Request visualization resources; native memory use may increase.
    /// Use matching options for creation and memory estimation.
    pub fn set_debug_visualization(&mut self, value: bool) {
        self.debug_visualization = value;
    }

    /// Maximum render width/height in pixels, also the required colour/depth
    /// extent and the motion extent when its resolution is [`MotionVectorResolution::Render`].
    pub fn max_render_size(&self) -> Dimensions {
        self.max_render_size
    }

    /// Maximum output width/height in pixels, also the required output allocation
    /// extent. The active output region may be smaller with dynamic resolution;
    /// see [`FrameParameters::set_active_output_size`](super::FrameParameters::set_active_output_size).
    pub fn max_upscale_size(&self) -> Dimensions {
        self.max_upscale_size
    }

    /// Device-depth convention to use when creating a context.
    pub fn depth_convention(&self) -> DepthConvention {
        self.depth_convention
    }

    /// Select standard/reversed finite/infinite device depth before context creation.
    /// Changing options does not reconfigure an already created context.
    pub fn set_depth_convention(&mut self, value: DepthConvention) {
        self.depth_convention = value;
    }

    /// Required motion-vector texture resolution.
    pub fn motion_vector_resolution(&self) -> MotionVectorResolution {
        self.motion_vector_resolution
    }

    /// Select render or display motion texture extents before context creation.
    /// Pixel motion uses pixels of its active region; UV motion uses that region's
    /// normalized UVs, independently of allocation dimensions.
    pub fn set_motion_vector_resolution(&mut self, value: MotionVectorResolution) {
        self.motion_vector_resolution = value;
    }

    /// Whether the input motion vectors contain projection jitter.
    pub fn motion_vector_jitter(&self) -> MotionVectorJitter {
        self.motion_vector_jitter
    }

    /// Select the input's existing jitter convention before context creation.
    /// With [`MotionVectorJitter::Included`], the SDK cancels jitter. The wrapper
    /// forwards applied pixel jitter unchanged and never rewrites texture content.
    pub fn set_motion_vector_jitter(&mut self, value: MotionVectorJitter) {
        self.motion_vector_jitter = value;
    }

    /// Exposure source for the context; explicit by default.
    pub fn exposure_mode(&self) -> ExposureMode {
        self.exposure_mode
    }

    /// Select caller-supplied or SDK-computed exposure before creation.
    /// Automatic contexts query the selected provider's requirements before
    /// becoming usable. Changing options never reconfigures an existing context.
    pub fn set_exposure_mode(&mut self, value: ExposureMode) {
        self.exposure_mode = value;
    }

    /// Whether active render/output sizes may vary within maxima; false by default.
    pub fn dynamic_resolution(&self) -> bool {
        self.dynamic_resolution
    }

    /// Enable the SDK's dynamic-resolution flag before context creation.
    /// Texture allocations remain at maxima; active render/output regions may vary.
    /// This does not reconfigure an existing context or request a history reset.
    pub fn set_dynamic_resolution(&mut self, enabled: bool) {
        self.dynamic_resolution = enabled;
    }

    /// Whether input colour has high dynamic range; false by default.
    pub fn high_dynamic_range(&self) -> bool {
        self.high_dynamic_range
    }

    /// Describe HDR input before context creation, independently of its encoding.
    ///
    /// In the default linear profile, the renderer supplies pre-tonemap RGBA16F
    /// colour with matching pre-exposure; output remains in the input signal
    /// domain. Non-linear interpretation depends on the actual provider. This flag does
    /// not convert pixels, select an exposure mode or configure an HDR display.
    /// The trusted provider must support the selected signal and profile.
    /// Changing options never reconfigures an existing context or resets history.
    pub fn set_high_dynamic_range(&mut self, enabled: bool) {
        self.high_dynamic_range = enabled;
    }

    /// Native input classification selected before creation; Linear by default.
    pub fn input_color_space(&self) -> InputColorSpace {
        self.input_color_space
    }

    /// Select the native input classification without transforming texture content.
    ///
    /// Changing options does not reconfigure an existing context. Per-frame
    /// encoding hints are separate. The trusted provider must support the chosen
    /// colour/exposure profile; resource queries establish no colour capability.
    pub fn set_input_color_space(&mut self, value: InputColorSpace) {
        self.input_color_space = value;
    }

    #[cfg(any(test, all(windows, feature = "dx12")))]
    pub(crate) fn check_color_encoding_hint(
        &self,
        hint: ColorEncodingHint,
        allow_context_mismatch: bool,
    ) -> Result<(), Error> {
        if hint != ColorEncodingHint::Unspecified
            && self.input_color_space != InputColorSpace::NonLinear
            && !allow_context_mismatch
        {
            return Err(Error::Input(crate::error::InputError::Dispatch(
                DispatchValidation::ColorEncodingContext,
            )));
        }
        Ok(())
    }

    #[cfg(any(test, all(windows, feature = "dx12")))]
    pub(crate) fn check_active_render_size(
        &self,
        requested: Option<Dimensions>,
    ) -> Result<Dimensions, Error> {
        let active = requested.unwrap_or(self.max_render_size);
        if active.width() > self.max_render_size.width()
            || active.height() > self.max_render_size.height()
            || (!self.dynamic_resolution && active != self.max_render_size)
        {
            return Err(Error::Input(crate::error::InputError::Dispatch(
                DispatchValidation::ActiveRenderSize,
            )));
        }
        Ok(active)
    }

    #[cfg(any(test, all(windows, feature = "dx12")))]
    pub(crate) fn check_active_output_size(
        &self,
        requested: Option<Dimensions>,
    ) -> Result<Dimensions, Error> {
        let active = requested.unwrap_or(self.max_upscale_size);
        if active.width() > self.max_upscale_size.width()
            || active.height() > self.max_upscale_size.height()
            || (!self.dynamic_resolution && active != self.max_upscale_size)
        {
            return Err(Error::Input(crate::error::InputError::Dispatch(
                DispatchValidation::ActiveOutputSize,
            )));
        }
        Ok(active)
    }

    #[cfg(any(test, all(windows, feature = "dx12")))]
    pub(crate) fn native_flags(&self) -> u32 {
        use fsr_sdk_sys::upscale::*;
        let mut flags = match self.depth_convention {
            DepthConvention::StandardFinite => 0,
            DepthConvention::ReversedFinite => FFX_UPSCALE_ENABLE_DEPTH_INVERTED,
            DepthConvention::StandardInfinite => FFX_UPSCALE_ENABLE_DEPTH_INFINITE,
            DepthConvention::ReversedInfinite => {
                FFX_UPSCALE_ENABLE_DEPTH_INVERTED | FFX_UPSCALE_ENABLE_DEPTH_INFINITE
            }
        };
        if self.motion_vector_resolution == MotionVectorResolution::Display {
            flags |= FFX_UPSCALE_ENABLE_DISPLAY_RESOLUTION_MOTION_VECTORS;
        }
        if self.motion_vector_jitter == MotionVectorJitter::Included {
            flags |= FFX_UPSCALE_ENABLE_MOTION_VECTORS_JITTER_CANCELLATION;
        }
        if self.exposure_mode == ExposureMode::Automatic {
            flags |= FFX_UPSCALE_ENABLE_AUTO_EXPOSURE;
        }
        if self.dynamic_resolution {
            flags |= FFX_UPSCALE_ENABLE_DYNAMIC_RESOLUTION;
        }
        if self.high_dynamic_range {
            flags |= FFX_UPSCALE_ENABLE_HIGH_DYNAMIC_RANGE;
        }
        if self.input_color_space == InputColorSpace::NonLinear {
            flags |= FFX_UPSCALE_ENABLE_NON_LINEAR_COLORSPACE;
        }
        if self.debug_checking {
            flags |= FFX_UPSCALE_ENABLE_DEBUG_CHECKING;
        }
        if self.debug_visualization {
            flags |= FFX_UPSCALE_ENABLE_DEBUG_VISUALIZATION;
        }
        flags
    }
}

#[cfg(any(test, all(windows, feature = "dx12")))]
pub(crate) fn check_resource_requirements(
    required: crate::query::ResourceFlags,
) -> Result<(), Error> {
    use crate::query::ResourceFlags;
    // Exposure is satisfied by an explicit texture or the automatic creation flag.
    // Other required inputs must fit the currently supported dispatch resources.
    let supported = ResourceFlags::COLOR.bits()
        | ResourceFlags::DEPTH.bits()
        | ResourceFlags::MOTION_VECTORS.bits()
        | ResourceFlags::EXPOSURE.bits()
        | ResourceFlags::REACTIVE_MASK.bits()
        | ResourceFlags::TRANSPARENCY_AND_COMPOSITION_MASK.bits();
    if required.bits() & !supported != 0 {
        return Err(Error::Input(
            crate::error::InputError::UnsupportedResourceRequirements { required },
        ));
    }
    Ok(())
}

#[cfg(any(test, all(windows, feature = "dx12")))]
pub(crate) fn check_mask_requirements(
    requirements: crate::query::ResourceRequirements,
    reactive: bool,
    composition: bool,
) -> Result<(), Error> {
    use crate::{error::ResourceRole, query::ResourceFlags};
    for (flag, role, present) in [
        (
            ResourceFlags::REACTIVE_MASK,
            ResourceRole::ReactiveMask,
            reactive,
        ),
        (
            ResourceFlags::TRANSPARENCY_AND_COMPOSITION_MASK,
            ResourceRole::TransparencyAndCompositionMask,
            composition,
        ),
    ] {
        // Optional bits report consumption, not permission to supply SDK-defined inputs.
        if requirements.required().contains(flag) && !present {
            return Err(Error::Input(crate::error::InputError::Dispatch(
                DispatchValidation::MissingRequiredResource(role),
            )));
        }
    }
    Ok(())
}
