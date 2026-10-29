// SPDX-License-Identifier: MPL-2.0

//! Experimental composition data; injected only into an isolated candidate.
#![allow(missing_docs)]

use crate::upscaler::common::{
    CameraParameters, ColorEncodingHint, Dimensions, ExposureMode, FrameTimeMillis,
    JitterOffsetPixels, NativeMotionVectorScale, PreExposure, ReactiveMaskOptions, Sharpening,
    UpscalerOptions,
};

#[derive(Debug, Clone)]
pub struct Config(UpscalerOptions);
impl Config {
    pub fn new(render: Dimensions, output: Dimensions) -> Self {
        Self(UpscalerOptions::new(render, output))
    }
    pub fn max_render_size(&self) -> Dimensions {
        self.0.max_render_size()
    }
    pub fn max_output_size(&self) -> Dimensions {
        self.0.max_upscale_size()
    }
    pub fn set_dynamic_resolution(&mut self, value: bool) {
        self.0.set_dynamic_resolution(value);
    }
    pub fn set_exposure_mode(&mut self, value: ExposureMode) {
        self.0.set_exposure_mode(value);
    }
    #[cfg(all(windows, feature = "dx12"))]
    pub(crate) fn selected(&self, provider: Option<crate::query::ProviderId>) -> UpscalerOptions {
        let mut options = self.0.clone();
        options.set_provider(provider);
        options
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotionVectorScale {
    Uv,
    Pixels,
    Native(NativeMotionVectorScale),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryAction {
    Continue,
    Reset,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorEncodingRequest {
    hint: ColorEncodingHint,
    allow_mismatch: bool,
}
impl ColorEncodingRequest {
    pub fn checked(hint: ColorEncodingHint) -> Self {
        Self {
            hint,
            allow_mismatch: false,
        }
    }
    pub fn allow_context_mismatch(hint: ColorEncodingHint) -> Self {
        Self {
            hint,
            allow_mismatch: true,
        }
    }
    pub fn hint(self) -> ColorEncodingHint {
        self.hint
    }
    pub fn allows_context_mismatch(self) -> bool {
        self.allow_mismatch
    }
}

#[derive(Debug, Clone)]
pub struct FrameParameters {
    jitter: JitterOffsetPixels,
    time: FrameTimeMillis,
    camera: CameraParameters,
    history: HistoryAction,
    sharpening: Sharpening,
    pre_exposure: PreExposure,
    motion: MotionVectorScale,
    render: Option<Dimensions>,
    output: Option<Dimensions>,
    encoding: ColorEncodingRequest,
    debug_view: bool,
}
impl FrameParameters {
    pub fn new(
        jitter: JitterOffsetPixels,
        time: FrameTimeMillis,
        camera: CameraParameters,
        history: HistoryAction,
    ) -> Self {
        Self {
            jitter,
            time,
            camera,
            history,
            sharpening: Sharpening::Disabled,
            pre_exposure: PreExposure::default(),
            motion: MotionVectorScale::Uv,
            render: None,
            output: None,
            encoding: ColorEncodingRequest::checked(ColorEncodingHint::Unspecified),
            debug_view: false,
        }
    }
    pub fn jitter(&self) -> JitterOffsetPixels {
        self.jitter
    }
    pub fn set_jitter(&mut self, value: JitterOffsetPixels) {
        self.jitter = value;
    }
    pub fn frame_time(&self) -> FrameTimeMillis {
        self.time
    }
    pub fn set_frame_time(&mut self, value: FrameTimeMillis) {
        self.time = value;
    }
    pub fn camera(&self) -> CameraParameters {
        self.camera
    }
    pub fn set_camera(&mut self, value: CameraParameters) {
        self.camera = value;
    }
    pub fn history_action(&self) -> HistoryAction {
        self.history
    }
    pub fn set_history_action(&mut self, value: HistoryAction) {
        self.history = value;
    }
    pub fn sharpening(&self) -> &Sharpening {
        &self.sharpening
    }
    pub fn set_sharpening(&mut self, value: Sharpening) {
        self.sharpening = value;
    }
    pub fn pre_exposure(&self) -> PreExposure {
        self.pre_exposure
    }
    pub fn set_pre_exposure(&mut self, value: PreExposure) {
        self.pre_exposure = value;
    }
    pub fn motion_scale(&self) -> MotionVectorScale {
        self.motion
    }
    pub fn set_motion_scale(&mut self, value: MotionVectorScale) {
        self.motion = value;
    }
    pub fn active_render_size(&self) -> Option<Dimensions> {
        self.render
    }
    pub fn set_active_render_size(&mut self, value: Option<Dimensions>) {
        self.render = value;
    }
    pub fn active_output_size(&self) -> Option<Dimensions> {
        self.output
    }
    pub fn set_active_output_size(&mut self, value: Option<Dimensions>) {
        self.output = value;
    }
    pub fn color_encoding(&self) -> ColorEncodingRequest {
        self.encoding
    }
    pub fn set_color_encoding(&mut self, value: ColorEncodingRequest) {
        self.encoding = value;
    }
    pub fn draw_debug_view(&self) -> bool {
        self.debug_view
    }
    pub fn set_draw_debug_view(&mut self, value: bool) {
        self.debug_view = value;
    }
}

#[derive(Debug, Clone)]
pub struct ReactiveMaskParameters {
    pub region: Dimensions,
    pub options: ReactiveMaskOptions,
}
impl ReactiveMaskParameters {
    pub fn new(region: Dimensions, options: ReactiveMaskOptions) -> Self {
        Self { region, options }
    }
}

#[cfg(all(windows, feature = "dx12"))]
pub mod dx12 {
    use super::*;
    use crate::upscaler::common::MotionVectorUnits;
    use crate::upscaler::dx12::{ID3D12GraphicsCommandList, ID3D12Resource, UpscaleDispatch};

    #[derive(Debug, Clone, Copy)]
    pub enum ExposureInput<'a> {
        Texture(&'a ID3D12Resource),
        Automatic,
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BindingRole {
        Color,
        Depth,
        MotionVectors,
        Exposure,
        Output,
    }
    #[derive(Debug, Clone)]
    pub struct UpscaleResources<'a> {
        color: &'a ID3D12Resource,
        depth: &'a ID3D12Resource,
        motion: &'a ID3D12Resource,
        exposure: ExposureInput<'a>,
        output: &'a ID3D12Resource,
        reactive: Option<&'a ID3D12Resource>,
        composition: Option<&'a ID3D12Resource>,
    }
    #[derive(Default)]
    pub struct ResourceBuilder<'a> {
        color: Option<&'a ID3D12Resource>,
        depth: Option<&'a ID3D12Resource>,
        motion: Option<&'a ID3D12Resource>,
        exposure: Option<ExposureInput<'a>>,
        output: Option<&'a ID3D12Resource>,
        reactive: Option<&'a ID3D12Resource>,
        composition: Option<&'a ID3D12Resource>,
    }
    impl<'a> ResourceBuilder<'a> {
        pub fn color(mut self, value: &'a ID3D12Resource) -> Self {
            self.color = Some(value);
            self
        }
        pub fn depth(mut self, value: &'a ID3D12Resource) -> Self {
            self.depth = Some(value);
            self
        }
        pub fn motion_vectors(mut self, value: &'a ID3D12Resource) -> Self {
            self.motion = Some(value);
            self
        }
        pub fn exposure(mut self, value: ExposureInput<'a>) -> Self {
            self.exposure = Some(value);
            self
        }
        pub fn output(mut self, value: &'a ID3D12Resource) -> Self {
            self.output = Some(value);
            self
        }
        pub fn reactive_mask(mut self, value: Option<&'a ID3D12Resource>) -> Self {
            self.reactive = value;
            self
        }
        pub fn transparency_and_composition_mask(
            mut self,
            value: Option<&'a ID3D12Resource>,
        ) -> Self {
            self.composition = value;
            self
        }
        pub fn build(self) -> Result<UpscaleResources<'a>, BindingRole> {
            Ok(UpscaleResources {
                color: self.color.ok_or(BindingRole::Color)?,
                depth: self.depth.ok_or(BindingRole::Depth)?,
                motion: self.motion.ok_or(BindingRole::MotionVectors)?,
                exposure: self.exposure.ok_or(BindingRole::Exposure)?,
                output: self.output.ok_or(BindingRole::Output)?,
                reactive: self.reactive,
                composition: self.composition,
            })
        }
    }
    impl<'a> UpscaleResources<'a> {
        pub fn builder() -> ResourceBuilder<'a> {
            ResourceBuilder::default()
        }
        pub(crate) fn assemble<'b>(
            &'b self,
            list: &'b ID3D12GraphicsCommandList,
            frame: &FrameParameters,
        ) -> UpscaleDispatch<'b> {
            let mut resources = match self.exposure {
                ExposureInput::Texture(exposure) => crate::upscaler::dx12::UpscaleResources::new(
                    self.color,
                    self.depth,
                    self.motion,
                    exposure,
                    self.output,
                ),
                ExposureInput::Automatic => {
                    crate::upscaler::dx12::UpscaleResources::without_exposure(
                        self.color,
                        self.depth,
                        self.motion,
                        self.output,
                    )
                }
            };
            resources.set_reactive_mask(self.reactive);
            resources.set_transparency_and_composition_mask(self.composition);
            let mut dispatch = UpscaleDispatch::new(
                list,
                resources,
                frame.jitter,
                frame.time,
                frame.camera,
                frame.history == HistoryAction::Reset,
            );
            dispatch.set_sharpening(frame.sharpening.clone());
            dispatch.set_pre_exposure(frame.pre_exposure);
            match frame.motion {
                MotionVectorScale::Uv => dispatch.set_motion_vector_units(MotionVectorUnits::Uv),
                MotionVectorScale::Pixels => {
                    dispatch.set_motion_vector_units(MotionVectorUnits::Pixels)
                }
                MotionVectorScale::Native(value) => {
                    dispatch.set_motion_vector_scale_override(Some(value))
                }
            }
            dispatch.set_active_render_size(frame.render);
            dispatch.set_active_output_size(frame.output);
            if frame.encoding.allow_mismatch {
                dispatch.set_color_encoding_hint_allow_context_mismatch(frame.encoding.hint);
            } else {
                dispatch.set_color_encoding_hint(frame.encoding.hint);
            }
            dispatch.set_draw_debug_view(frame.debug_view);
            dispatch
        }
    }

    #[derive(Debug, Clone)]
    pub struct ReactiveMaskResources<'a> {
        pub opaque: &'a ID3D12Resource,
        pub color: &'a ID3D12Resource,
        pub output: &'a ID3D12Resource,
    }
    impl<'a> ReactiveMaskResources<'a> {
        pub(crate) fn assemble<'b>(
            &'b self,
            list: &'b ID3D12GraphicsCommandList,
            parameters: &ReactiveMaskParameters,
        ) -> crate::upscaler::dx12::GenerateReactiveMaskDispatch<'b> {
            crate::upscaler::dx12::GenerateReactiveMaskDispatch::new(
                list,
                self.opaque,
                self.color,
                self.output,
                parameters.region,
                parameters.options.clone(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn setup_spike_portable_parameters_replace_effective_intent() -> Result<(), crate::error::Error>
    {
        let mut frame = FrameParameters::new(
            JitterOffsetPixels::new(0.25, -0.25)?,
            FrameTimeMillis::new(16.0)?,
            CameraParameters::new(0.1, 100.0, 1.0, 1.0)?,
            HistoryAction::Reset,
        );
        frame.set_motion_scale(MotionVectorScale::Native(NativeMotionVectorScale::new(
            -0.0,
            f32::from_bits(1),
        )?));
        frame.set_motion_scale(MotionVectorScale::Pixels);
        assert_eq!(frame.motion_scale(), MotionVectorScale::Pixels);
        frame.set_history_action(HistoryAction::Continue);
        frame.set_jitter(JitterOffsetPixels::new(-0.25, 0.25)?);
        frame.set_frame_time(FrameTimeMillis::new(20.0)?);
        assert_eq!(frame.history_action(), HistoryAction::Continue);
        assert_eq!(frame.jitter().x(), -0.25);
        assert_eq!(frame.frame_time().milliseconds(), 20.0);
        frame.set_color_encoding(ColorEncodingRequest::allow_context_mismatch(
            ColorEncodingHint::Srgb,
        ));
        frame.set_color_encoding(ColorEncodingRequest::checked(
            ColorEncodingHint::Unspecified,
        ));
        assert!(!frame.color_encoding().allows_context_mismatch());
        Ok(())
    }
}
