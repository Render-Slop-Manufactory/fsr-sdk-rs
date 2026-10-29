// SPDX-License-Identifier: MPL-2.0

//! Test equipment convenience only; no release API or alternate mapping/defaults.
//! Native and controlled workloads retain their existing frame factories while
//! all construction and recording go through production builders/parameters.
use super::super::*;
use crate::upscaler::{
    CameraParameters, FrameParameters, FrameTimeMillis, HistoryAction, JitterOffsetPixels,
    ReactiveMaskOptions, ReactiveMaskParameters,
};

#[derive(Debug, Clone)]
pub(crate) struct UpscaleTestInputs<'a> {
    pub(crate) command_list: &'a ID3D12GraphicsCommandList,
    pub(crate) resources: UpscaleResources<'a>,
    pub(crate) parameters: FrameParameters,
}
impl<'a> UpscaleTestInputs<'a> {
    pub(crate) fn new(
        command_list: &'a ID3D12GraphicsCommandList,
        resources: UpscaleResources<'a>,
        jitter: JitterOffsetPixels,
        time: FrameTimeMillis,
        camera: CameraParameters,
        reset: bool,
    ) -> Self {
        Self {
            command_list,
            resources,
            parameters: FrameParameters::new(
                jitter,
                time,
                camera,
                if reset {
                    HistoryAction::Reset
                } else {
                    HistoryAction::Continue
                },
            ),
        }
    }
    pub(crate) fn command_list(&self) -> &'a ID3D12GraphicsCommandList {
        self.command_list
    }
    pub(crate) fn resources(&self) -> &UpscaleResources<'a> {
        &self.resources
    }
    pub(crate) fn reset(&self) -> bool {
        self.parameters.history_action() == HistoryAction::Reset
    }
}
impl std::ops::Deref for UpscaleTestInputs<'_> {
    type Target = FrameParameters;
    fn deref(&self) -> &Self::Target {
        &self.parameters
    }
}
impl std::ops::DerefMut for UpscaleTestInputs<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parameters
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ReactiveMaskTestInputs<'a> {
    pub(crate) command_list: &'a ID3D12GraphicsCommandList,
    pub(crate) resources: ReactiveMaskResources<'a>,
    pub(crate) parameters: ReactiveMaskParameters,
}
impl<'a> ReactiveMaskTestInputs<'a> {
    pub(crate) fn new(
        command_list: &'a ID3D12GraphicsCommandList,
        color_opaque_only: &'a ID3D12Resource,
        color_pre_upscale: &'a ID3D12Resource,
        output: &'a ID3D12Resource,
        render_size: crate::upscaler::Dimensions,
        options: ReactiveMaskOptions,
    ) -> Self {
        Self {
            command_list,
            resources: ReactiveMaskResources::builder()
                .color_opaque_only(color_opaque_only)
                .color_pre_upscale(color_pre_upscale)
                .output(output)
                .build()
                .unwrap(),
            parameters: ReactiveMaskParameters::new(render_size, options),
        }
    }
}

impl<'a> ReactiveMaskTestInputs<'a> {
    pub(crate) fn command_list(&self) -> &'a ID3D12GraphicsCommandList {
        self.command_list
    }
    pub(crate) fn color_opaque_only(&self) -> &'a ID3D12Resource {
        self.resources.color_opaque_only()
    }
    pub(crate) fn color_pre_upscale(&self) -> &'a ID3D12Resource {
        self.resources.color_pre_upscale()
    }
    pub(crate) fn output(&self) -> &'a ID3D12Resource {
        self.resources.output()
    }
}
impl std::ops::Deref for ReactiveMaskTestInputs<'_> {
    type Target = ReactiveMaskParameters;
    fn deref(&self) -> &Self::Target {
        &self.parameters
    }
}
