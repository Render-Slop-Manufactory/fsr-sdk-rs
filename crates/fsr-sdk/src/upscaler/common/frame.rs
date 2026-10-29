// SPDX-License-Identifier: MPL-2.0

//! Portable recording intent; no backend objects, native calls or temporal progression.
use super::{
    CameraParameters, ColorEncodingHint, Dimensions, FrameTimeMillis, JitterOffsetPixels,
    NativeMotionVectorScale, PreExposure, ReactiveMaskOptions, Sharpening,
};

/// Complete native motion scaling choice. Changing it replaces the prior choice.
///
/// UV uses the actual active motion extent (render/display according to the
/// creation profile), Pixels uses `(1, 1)`, and Native forwards finite component
/// bits literally. No pixel conversion, dormant fallback or automatic NDC
/// conversion occurs. Actual-provider numerical/access support remains required.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[non_exhaustive]
pub enum MotionVectorScale {
    /// Current-to-previous top-down UV displacement of the active motion region.
    #[default]
    Uv,
    /// Current-to-previous top-down pixel displacement.
    Pixels,
    /// Literal checked multipliers, including signed zero/subnormal components.
    Native(NativeMotionVectorScale),
}

/// History intent for this recording; it never advances automatically.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum HistoryAction {
    /// Continue history after the renderer has established ordered execution.
    Continue,
    /// Request a history reset for this frame; no old recording is repaired.
    Reset,
}

/// One native colour hint and its context-pairing permission.
///
/// This is signal metadata, not conversion or a provider-support certificate.
/// A mismatch permission bypasses only the wrapper's creation/hint pairing rule.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ColorEncodingRequest {
    hint: ColorEncodingHint,
    allow_mismatch: bool,
}
impl ColorEncodingRequest {
    /// Request the hint with ordinary context pairing checks.
    pub fn checked(hint: ColorEncodingHint) -> Self {
        Self {
            hint,
            allow_mismatch: false,
        }
    }
    /// Explicitly permit this hint on a mismatched context. Other checks remain.
    pub fn allow_context_mismatch(hint: ColorEncodingHint) -> Self {
        Self {
            hint,
            allow_mismatch: true,
        }
    }
    /// Native encoding hint, forwarded without a signal transformation.
    pub fn hint(self) -> ColorEncodingHint {
        self.hint
    }
    /// Whether only the creation/hint pairing check is bypassed.
    pub fn allows_context_mismatch(self) -> bool {
        self.allow_mismatch
    }
}

/// Portable values for one upscaling recording, independent of GPU bindings.
///
/// Supply checked jitter, elapsed time, camera and history intent explicitly.
/// Reusing/cloning this value repeats its current values, including Reset.
/// Update jitter/time/camera/history each frame as required by the renderer;
/// clear an initial Reset to Continue when continuing history. No automatic
/// jitter sequence, elapsed time, history progression, GPU wait or native call.
/// Optional defaults: UV motion, sharpening disabled, pre-exposure 1, maximum
/// render/output extents, checked Unspecified colour hint and debug view off.
#[derive(Debug, Clone)]
pub struct FrameParameters {
    jitter: JitterOffsetPixels,
    frame_time: FrameTimeMillis,
    camera: CameraParameters,
    history_action: HistoryAction,
    sharpening: Sharpening,
    pre_exposure: PreExposure,
    motion_vector_scale: MotionVectorScale,
    active_render_size: Option<Dimensions>,
    active_output_size: Option<Dimensions>,
    color_encoding: ColorEncodingRequest,
    draw_debug_view: bool,
}
impl FrameParameters {
    /// Assemble checked required values with the documented optional defaults.
    pub fn new(
        jitter: JitterOffsetPixels,
        frame_time: FrameTimeMillis,
        camera: CameraParameters,
        history_action: HistoryAction,
    ) -> Self {
        Self {
            jitter,
            frame_time,
            camera,
            history_action,
            sharpening: Sharpening::Disabled,
            pre_exposure: PreExposure::default(),
            motion_vector_scale: MotionVectorScale::Uv,
            active_render_size: None,
            active_output_size: None,
            color_encoding: ColorEncodingRequest::default(),
            draw_debug_view: false,
        }
    }
    /// Renderer-applied projection jitter in pixels; not motion texture units.
    pub fn jitter(&self) -> JitterOffsetPixels {
        self.jitter
    }
    /// Update jitter, without inspection, recording or history progression.
    pub fn set_jitter(&mut self, value: JitterOffsetPixels) {
        self.jitter = value;
    }
    /// Elapsed frame time in milliseconds; no clock is read.
    pub fn frame_time(&self) -> FrameTimeMillis {
        self.frame_time
    }
    /// Update frame time, without inspection, recording or history progression.
    pub fn set_frame_time(&mut self, value: FrameTimeMillis) {
        self.frame_time = value;
    }
    /// Camera/depth convention; checked against the receiving context.
    pub fn camera(&self) -> CameraParameters {
        self.camera
    }
    /// Update camera, without inspection, recording or history progression.
    pub fn set_camera(&mut self, value: CameraParameters) {
        self.camera = value;
    }
    /// History intent; remains unchanged until explicitly updated.
    pub fn history_action(&self) -> HistoryAction {
        self.history_action
    }
    /// Update history action, without inspection, recording or history progression.
    pub fn set_history_action(&mut self, value: HistoryAction) {
        self.history_action = value;
    }
    /// Positive finite factor already applied to scene colour; no pixels are multiplied.
    pub fn pre_exposure(&self) -> PreExposure {
        self.pre_exposure
    }
    /// Update pre-exposure, without inspection, recording or history progression.
    pub fn set_pre_exposure(&mut self, value: PreExposure) {
        self.pre_exposure = value;
    }
    /// Complete effective motion request; replaces Native, Pixels or UV without fallback storage.
    pub fn motion_vector_scale(&self) -> MotionVectorScale {
        self.motion_vector_scale
    }
    /// Update motion vector scale, without inspection, recording or history progression.
    pub fn set_motion_vector_scale(&mut self, value: MotionVectorScale) {
        self.motion_vector_scale = value;
    }
    /// Top-left active render region; None uses context maximum. Allocations stay at maximum.
    pub fn active_render_size(&self) -> Option<Dimensions> {
        self.active_render_size
    }
    /// Request a render region. Smaller sizes require dynamic-resolution creation;
    /// oversized axes reject before COM inspection. This updates no allocation,
    /// native state or history; inputs must contain matching active image content.
    pub fn set_active_render_size(&mut self, value: Option<Dimensions>) {
        self.active_render_size = value;
    }
    /// Top-left active output region; None uses context maximum. Padding may be written.
    pub fn active_output_size(&self) -> Option<Dimensions> {
        self.active_output_size
    }
    /// Request an output region. Smaller sizes require dynamic-resolution creation;
    /// oversized axes reject before COM inspection. Output/display-motion
    /// allocations stay at maximum. Provider support/access remains conditional;
    /// padding can be written and no history reset or wait occurs.
    pub fn set_active_output_size(&mut self, value: Option<Dimensions>) {
        self.active_output_size = value;
    }
    /// Complete colour request, including context-pairing permission; no signal conversion.
    pub fn color_encoding(&self) -> ColorEncodingRequest {
        self.color_encoding
    }
    /// Update colour encoding, without inspection, recording or history progression.
    pub fn set_color_encoding(&mut self, value: ColorEncodingRequest) {
        self.color_encoding = value;
    }
    /// Native debug-view request, independent of creation diagnostics.
    pub fn draw_debug_view(&self) -> bool {
        self.draw_debug_view
    }
    /// Update draw debug view, without inspection, recording or history progression.
    pub fn set_draw_debug_view(&mut self, value: bool) {
        self.draw_debug_view = value;
    }
    /// Current native sharpening request, disabled by default.
    pub fn sharpening(&self) -> &Sharpening {
        &self.sharpening
    }
    /// Replace native sharpening intent; enabled zero is distinct from disabled.
    pub fn set_sharpening(&mut self, value: Sharpening) {
        self.sharpening = value;
    }
}

/// Portable reactive-mask generation region and direct native controls.
///
/// Reuse does not advance a frame. The active top-left region is checked against
/// the receiving context's maximum, independently of its dynamic-resolution flag.
/// Actual-provider support, signal meaning and access remain recording duties.
#[derive(Debug, Clone)]
pub struct ReactiveMaskParameters {
    render_size: Dimensions,
    options: ReactiveMaskOptions,
}
impl ReactiveMaskParameters {
    /// Assemble an explicit region and checked native controls; no GPU work.
    pub fn new(render_size: Dimensions, options: ReactiveMaskOptions) -> Self {
        Self {
            render_size,
            options,
        }
    }
    /// Active top-left region, independent of allocation dimensions.
    pub fn render_size(&self) -> Dimensions {
        self.render_size
    }
    /// Replace the region without inspecting allocations or resetting history.
    pub fn set_render_size(&mut self, value: Dimensions) {
        self.render_size = value;
    }
    /// Current direct native controls.
    pub fn options(&self) -> &ReactiveMaskOptions {
        &self.options
    }
    /// Replace the complete native controls; no clamping or automatic policy.
    pub fn set_options(&mut self, value: ReactiveMaskOptions) {
        self.options = value;
    }
}
