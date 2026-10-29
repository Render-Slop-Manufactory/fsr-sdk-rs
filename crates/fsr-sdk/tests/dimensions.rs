// SPDX-License-Identifier: MPL-2.0

#[macro_use]
#[path = "fixtures/errors.rs"]
mod test_errors;

use fsr_sdk::{
    error::{DispatchScalar, DispatchValidation, Error},
    upscaler::{
        CameraParameters, Dimensions, FrameTimeMillis, JitterOffsetPixels, PreExposure,
        UpscalerOptions, common,
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
    let factor: PreExposure = common::PreExposure::new(0.25).unwrap();
    assert_eq!(factor.value(), 0.25);
}

#[test]
fn pre_exposure_preserves_positive_finite_bits_and_rejects_invalid_values() {
    assert_eq!(PreExposure::default().value(), 1.0);
    for value in [
        f32::from_bits(1),
        f32::MIN_POSITIVE,
        0.25,
        1.0,
        4.0,
        f32::MAX,
    ] {
        assert_eq!(
            PreExposure::new(value).unwrap().value().to_bits(),
            value.to_bits()
        );
    }
    for value in [
        0.0,
        -0.0,
        -f32::from_bits(1),
        -1.0,
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
    ] {
        assert_result!(
            PreExposure::new(value),
            Err(Error::Input(fsr_sdk::error::InputError::Dispatch(
                DispatchValidation::Scalar(DispatchScalar::PreExposure)
            )))
        );
    }
    fn traits<T: Copy + Send + Sync + std::fmt::Debug + PartialEq>() {}
    traits::<PreExposure>();
}

#[cfg(all(windows, feature = "dx12"))]
#[test]
fn legacy_and_dx12_module_paths_accept_the_same_types() {
    use fsr_sdk::upscaler::{
        ID3D12GraphicsCommandList, ID3D12Resource, UpscaleResources, Upscaler, dx12,
    };

    fn create(
        setup: &mut dx12::UpscalerSetup<'_, '_>,
        options: &common::UpscalerOptions,
    ) -> Result<Upscaler, Error> {
        setup.create(options)
    }
    let _ = create;
    let _: fn(Upscaler) -> dx12::Upscaler = |owner| owner;
    let _: for<'a> fn(dx12::UpscaleResources<'a>) -> UpscaleResources<'a> = |frame| frame;
    let _: for<'a> fn(UpscaleResources<'a>) -> dx12::UpscaleResources<'a> = |frame| frame;
    let _: Option<ID3D12Resource> = None::<dx12::ID3D12Resource>;
    let _: Option<ID3D12GraphicsCommandList> = None::<dx12::ID3D12GraphicsCommandList>;

    #[cfg(feature = "movable")]
    {
        use fsr_sdk::upscaler::MovableUpscaler;
        fn create(
            setup: &mut dx12::MovableUpscalerSetup<'_, '_>,
            options: &common::UpscalerOptions,
        ) -> Result<MovableUpscaler, Error> {
            setup.create(options)
        }
        let _ = create;
        let _: fn(MovableUpscaler) -> dx12::MovableUpscaler = |owner| owner;
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
        assert_result!(
            Dimensions::new(width, height),
            Err(Error::Input(
                fsr_sdk::error::InputError::InvalidDimensions { width, height }
            ))
        );
    }
    let maximum = Dimensions::new(u32::MAX, u32::MAX).unwrap();
    assert_eq!((maximum.width(), maximum.height()), (u32::MAX, u32::MAX));
}

#[test]
fn color_selections_are_portable_and_default_to_unchanged_flags() {
    use fsr_sdk::upscaler::{ColorEncodingHint, InputColorSpace};
    let size = Dimensions::new(320, 180).unwrap();
    let mut options = UpscalerOptions::new(size, size);
    assert_eq!(InputColorSpace::default(), InputColorSpace::Linear);
    assert_eq!(ColorEncodingHint::default(), ColorEncodingHint::Unspecified);
    assert_eq!(options.input_color_space(), common::InputColorSpace::Linear);
    let original = options.clone();
    options.set_input_color_space(common::InputColorSpace::NonLinear);
    assert_eq!(
        options.clone().input_color_space(),
        InputColorSpace::NonLinear
    );
    assert_eq!(original.input_color_space(), InputColorSpace::Linear);
    assert!(!options.high_dynamic_range());
    assert!(!options.dynamic_resolution());
    assert_eq!(options.max_render_size(), size);
    assert_eq!(options.max_upscale_size(), size);
    fn traits<T: Copy + Eq + Default + std::fmt::Debug + Send + Sync>() {}
    traits::<InputColorSpace>();
    traits::<ColorEncodingHint>();
}

#[test]
fn hdr_options_are_portable_opt_in_and_clone_independently() {
    let size = common::Dimensions::new(320, 180).unwrap();
    let mut options = common::UpscalerOptions::new(size, size);
    assert!(!options.high_dynamic_range());
    let original: UpscalerOptions = options.clone();
    options.set_high_dynamic_range(true);
    assert!(options.clone().high_dynamic_range());
    assert!(!original.high_dynamic_range());
    assert_eq!(
        options.exposure_mode(),
        fsr_sdk::upscaler::ExposureMode::Explicit
    );
    assert!(!options.dynamic_resolution());
    assert_eq!(options.max_render_size(), size);
    assert_eq!(options.max_upscale_size(), size);
    options.set_high_dynamic_range(false);
    assert!(!options.high_dynamic_range());
}

#[test]
fn dynamic_resolution_options_are_portable_and_preserve_clones() {
    let d = Dimensions::new(320, 180).unwrap();
    let mut options = UpscalerOptions::new(d, d);
    assert!(!options.dynamic_resolution());
    let fixed = options.clone();
    options.set_dynamic_resolution(true);
    assert!(options.clone().dynamic_resolution());
    assert!(!fixed.dynamic_resolution());
    assert_eq!(options.max_render_size(), d);
    assert_eq!(options.max_upscale_size(), d);
    options.set_dynamic_resolution(false);
    assert!(!options.dynamic_resolution());
}

#[test]
fn depth_and_motion_profiles_preserve_defaults_and_option_clones() {
    use fsr_sdk::upscaler::{
        DepthConvention, MotionVectorJitter, MotionVectorResolution, MotionVectorScale,
    };
    let mut options = UpscalerOptions::new(
        Dimensions::new(320, 180).unwrap(),
        Dimensions::new(640, 360).unwrap(),
    );
    assert_eq!(options.depth_convention(), DepthConvention::StandardFinite);
    assert_eq!(
        options.motion_vector_resolution(),
        MotionVectorResolution::Render
    );
    assert_eq!(options.motion_vector_jitter(), MotionVectorJitter::Excluded);
    let original = options.clone();
    options.set_motion_vector_resolution(MotionVectorResolution::Display);
    options.set_motion_vector_jitter(MotionVectorJitter::Included);
    assert_eq!(
        options.motion_vector_resolution(),
        MotionVectorResolution::Display
    );
    assert_eq!(options.motion_vector_jitter(), MotionVectorJitter::Included);
    assert_eq!(
        original.motion_vector_resolution(),
        MotionVectorResolution::Render
    );
    assert_eq!(
        original.motion_vector_jitter(),
        MotionVectorJitter::Excluded
    );
    options.set_depth_convention(DepthConvention::ReversedFinite);
    assert_eq!(options.depth_convention(), DepthConvention::ReversedFinite);
    assert_eq!(original.depth_convention(), DepthConvention::StandardFinite);
    assert_eq!(MotionVectorScale::default(), MotionVectorScale::Uv);
    fn traits<T: Copy + Send + Sync + std::fmt::Debug + PartialEq>() {}
    traits::<DepthConvention>();
    traits::<MotionVectorScale>();
    traits::<MotionVectorResolution>();
    traits::<MotionVectorJitter>();
    // Reversed depth still uses finite, ordered geometric camera distances.
    assert!(CameraParameters::new(0.1, 100.0, 1.0, 1.0).is_ok());
    assert!(CameraParameters::new(100.0, 0.1, 1.0, 1.0).is_err());
    assert!(CameraParameters::new(0.1, f32::INFINITY, 1.0, 1.0).is_err());
}

#[test]
fn common_scalar_domains_are_checked_without_native_apis() {
    let invalid_scalar = |scalar| {
        Error::Input(fsr_sdk::error::InputError::Dispatch(
            DispatchValidation::Scalar(scalar),
        ))
    };
    assert_result!(
        JitterOffsetPixels::new(f32::NAN, 0.0),
        Err(invalid_scalar(DispatchScalar::JitterX))
    );
    assert_result!(
        JitterOffsetPixels::new(0.0, f32::INFINITY),
        Err(invalid_scalar(DispatchScalar::JitterY))
    );
    for invalid in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert_result!(
            FrameTimeMillis::new(invalid),
            Err(invalid_scalar(DispatchScalar::FrameTimeMillis))
        );
    }
    assert_result!(
        CameraParameters::new(0.0, 100.0, 1.0, 1.0),
        Err(invalid_scalar(DispatchScalar::CameraNear))
    );
    assert_result!(
        CameraParameters::new(0.1, 0.1, 1.0, 1.0),
        Err(invalid_scalar(DispatchScalar::CameraFar))
    );
    assert_result!(
        CameraParameters::new(0.1, 100.0, std::f32::consts::PI, 1.0),
        Err(invalid_scalar(DispatchScalar::VerticalFovRadians))
    );
    assert_result!(
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
fn infinite_camera_checks_scalars_and_preserves_projection_identity() {
    use fsr_sdk::upscaler::DepthConvention;
    let camera = CameraParameters::new_infinite(0.1, 1.0, 2.0).unwrap();
    assert!(camera.has_infinite_far_plane());
    assert_eq!(
        (
            camera.near(),
            camera.far(),
            camera.vertical_fov_radians(),
            camera.view_space_to_meters()
        ),
        (0.1, f32::MAX, 1.0, 2.0)
    );
    let finite = CameraParameters::new(0.1, f32::MAX, 1.0, 2.0).unwrap();
    assert!(!finite.has_infinite_far_plane());
    assert_ne!(finite, camera);
    for (depth, infinite) in [
        (DepthConvention::StandardFinite, false),
        (DepthConvention::ReversedFinite, false),
        (DepthConvention::StandardInfinite, true),
        (DepthConvention::ReversedInfinite, true),
    ] {
        assert_eq!(depth.is_infinite(), infinite);
    }
    for near in [0.0, -0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_result!(
            CameraParameters::new_infinite(near, 1.0, 1.0),
            Err(Error::Input(fsr_sdk::error::InputError::Dispatch(
                DispatchValidation::Scalar(DispatchScalar::CameraNear)
            )))
        );
    }
    for fov in [0.0, -1.0, std::f32::consts::PI, f32::NAN, f32::INFINITY] {
        assert_result!(
            CameraParameters::new_infinite(0.1, fov, 1.0),
            Err(Error::Input(fsr_sdk::error::InputError::Dispatch(
                DispatchValidation::Scalar(DispatchScalar::VerticalFovRadians)
            )))
        );
    }
    for scale in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert_result!(
            CameraParameters::new_infinite(0.1, 1.0, scale),
            Err(Error::Input(fsr_sdk::error::InputError::Dispatch(
                DispatchValidation::Scalar(DispatchScalar::ViewSpaceToMeters)
            )))
        );
    }
}

#[test]
fn exposure_selection_defaults_to_explicit_and_clones_independently() {
    use fsr_sdk::upscaler::ExposureMode;
    let mut options = UpscalerOptions::new(
        Dimensions::new(320, 180).unwrap(),
        Dimensions::new(640, 360).unwrap(),
    );
    assert_eq!(ExposureMode::default(), ExposureMode::Explicit);
    assert_eq!(options.exposure_mode(), ExposureMode::Explicit);
    let original = options.clone();
    options.set_exposure_mode(ExposureMode::Automatic);
    assert_eq!(options.exposure_mode(), ExposureMode::Automatic);
    assert_eq!(original.exposure_mode(), ExposureMode::Explicit);
    fn traits<T: Copy + Eq + Send + Sync + std::fmt::Debug>() {}
    traits::<ExposureMode>();
}

#[test]
fn provider_selection_defaults_are_portable_without_native_apis() {
    use fsr_sdk::query::ProviderSelection;
    fn traits<T: Copy + Eq + Send + Sync + std::fmt::Debug>() {}
    traits::<ProviderSelection>();
    assert_eq!(ProviderSelection::default(), ProviderSelection::Default);
}
