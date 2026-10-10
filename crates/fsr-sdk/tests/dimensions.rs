// SPDX-License-Identifier: MPL-2.0

use fsr_sdk::{
    error::{DispatchScalar, DispatchValidation, Error},
    upscaler::{
        CameraParameters, Dimensions, FrameTimeMillis, JitterOffsetPixels, UpscalerOptions, common,
    },
};

#[test]
fn legacy_and_explicit_module_paths_accept_the_same_types() {
    let render: Dimensions = common::Dimensions::new(320, 180).unwrap();
    let output: common::Dimensions = Dimensions::new(640, 360).unwrap();
    let options: UpscalerOptions = common::UpscalerOptions::new(render, output);
    assert_eq!(options.max_render_size(), render);
    assert_eq!(options.max_upscale_size(), output);

    let jitter: JitterOffsetPixels = common::JitterOffsetPixels::new(0.25, -0.25).unwrap();
    let time: FrameTimeMillis = common::FrameTimeMillis::new(16.667).unwrap();
    let camera: CameraParameters = common::CameraParameters::new(0.1, 100.0, 1.0, 1.0).unwrap();
    assert_eq!((jitter.x(), jitter.y()), (0.25, -0.25));
    assert_eq!(time.milliseconds(), 16.667);
    assert_eq!(camera.far(), 100.0);
}

#[cfg(all(windows, feature = "dx12"))]
#[test]
fn legacy_and_dx12_module_paths_accept_the_same_types() {
    use fsr_sdk::{
        runtime::Runtime,
        upscaler::{
            ID3D12Device, ID3D12GraphicsCommandList, ID3D12Resource, UpscaleDispatch, Upscaler,
            dx12,
        },
    };

    let _: fn(&Runtime, &ID3D12Device, common::UpscalerOptions) -> Result<Upscaler, Error> =
        dx12::Upscaler::new;
    let _: fn(&Runtime, &dx12::ID3D12Device, UpscalerOptions) -> Result<dx12::Upscaler, Error> =
        Upscaler::new;
    let _: for<'a> fn(dx12::UpscaleDispatch<'a>) -> UpscaleDispatch<'a> = |frame| frame;
    let _: for<'a> fn(UpscaleDispatch<'a>) -> dx12::UpscaleDispatch<'a> = |frame| frame;
    let _: Option<ID3D12Resource> = None::<dx12::ID3D12Resource>;
    let _: Option<ID3D12GraphicsCommandList> = None::<dx12::ID3D12GraphicsCommandList>;

    #[cfg(feature = "movable")]
    {
        use fsr_sdk::{runtime::MovableRuntime, upscaler::MovableUpscaler};
        let _: fn(
            &MovableRuntime,
            &ID3D12Device,
            common::UpscalerOptions,
        ) -> Result<MovableUpscaler, Error> = dx12::MovableUpscaler::new;
        let _: fn(
            &MovableRuntime,
            &dx12::ID3D12Device,
            UpscalerOptions,
        ) -> Result<dx12::MovableUpscaler, Error> = MovableUpscaler::new;
    }
}

#[test]
fn renderer_can_recover_sizes_from_owned_configuration() {
    let options = UpscalerOptions::new(
        Dimensions::new(1280, 720).unwrap(),
        Dimensions::new(1920, 1080).unwrap(),
    );
    let reused = options.clone();
    let render = reused.max_render_size();
    let output = reused.max_upscale_size();
    assert_eq!((render.width(), render.height()), (1280, 720));
    assert_eq!((output.width(), output.height()), (1920, 1080));
    for (width, height) in [(0, 0), (0, 720), (1280, 0)] {
        assert_eq!(
            Dimensions::new(width, height),
            Err(Error::InvalidDimensions { width, height })
        );
    }
    let maximum = Dimensions::new(u32::MAX, u32::MAX).unwrap();
    assert_eq!((maximum.width(), maximum.height()), (u32::MAX, u32::MAX));
}

#[test]
fn common_scalar_domains_are_checked_without_native_apis() {
    let invalid_scalar = |scalar| Error::InvalidDispatch {
        reason: DispatchValidation::Scalar(scalar),
    };
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
