// SPDX-License-Identifier: MPL-2.0

use super::{com::Scene, *};
use crate::error::{DispatchScalar, DispatchValidation, Operation, ResourceProperty, ResourceRole};
use std::cell::RefCell;

thread_local! {
    static CAPTURED: RefCell<Option<ffxDispatchDescUpscale>> = const { RefCell::new(None) };
}

unsafe extern "C" fn capture(header: *const ffxApiHeader) {
    // SAFETY: Registered only for the public dispatch calls below, which pass a
    // complete live descriptor. This raw ABI value owns no resources and has no
    // Drop implementation; its copied pointers are compared, never dereferenced.
    let descriptor = unsafe { std::ptr::read(header.cast::<ffxDispatchDescUpscale>()) };
    CAPTURED.with(|captured| *captured.borrow_mut() = Some(descriptor));
}

#[test]
fn public_frame_accessors_and_clone_only_borrow_inputs() {
    let scene = Scene::new();
    let counts = scene.reference_counts();
    let frame = scene.input();
    let resources = frame.resources().clone();
    assert_eq!(frame.command_list().as_raw(), scene.list.as_raw());
    assert_eq!(resources.color().as_raw(), scene.resources[0].as_raw());
    assert_eq!(resources.depth().as_raw(), scene.resources[1].as_raw());
    assert_eq!(
        resources.motion_vectors().as_raw(),
        scene.resources[2].as_raw()
    );
    assert_eq!(
        resources.exposure().unwrap().as_raw(),
        scene.resources[3].as_raw()
    );
    assert_eq!(resources.output().as_raw(), scene.resources[4].as_raw());
    assert_eq!(frame.jitter(), JitterOffsetPixels::new(0.0, 0.0).unwrap());
    assert_eq!(frame.frame_time(), FrameTimeMillis::new(16.0).unwrap());
    assert_eq!(
        frame.camera(),
        CameraParameters::new(0.1, 1000.0, 1.0, 1.0).unwrap()
    );
    assert!(frame.reset());
    assert!(matches!(frame.sharpening(), Sharpening::Disabled));
    let replacement = UpscaleDispatch::new(
        frame.command_list(),
        resources,
        frame.jitter(),
        frame.frame_time(),
        frame.camera(),
        false,
    );
    assert!(!replacement.reset());
    assert_eq!(scene.reference_counts(), counts);
}

#[test]
fn public_dispatch_maps_modes_exactly_on_one_context_and_preserves_failures() {
    let (runtime, observer) = fixture_runtime();
    let scene = Scene::new();
    let mut upscaler = Upscaler::new(&runtime, &scene.device, sizes()).unwrap();
    // SAFETY: Exact callback ABI of the controlled fixture. Only this test's
    // complete public dispatch descriptors are observed; no GPU work is recorded.
    let set_code = unsafe {
        observer
            .get::<extern "C" fn(u32)>(c"fixture_set_dispatch_code")
            .unwrap()
    };
    // SAFETY: The standalone fixture accepts this C-ABI descriptor callback.
    unsafe {
        observer
            .get::<extern "C" fn(Option<unsafe extern "C" fn(*const ffxApiHeader)>)>(
                c"fixture_set_dispatch_observer",
            )
            .unwrap()(Some(capture));
    }
    let calls = || {
        // SAFETY: Exact observation export of the controlled fixture.
        unsafe {
            observer
                .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
                .unwrap()()
            .calls
        }
    };
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.1, 1.1] {
        assert_eq!(
            NativeSharpness::new(value),
            Err(Error::InvalidDispatch {
                reason: DispatchValidation::Scalar(DispatchScalar::NativeSharpness),
            })
        );
    }
    assert_eq!(calls(), 0);

    for (index, setting) in [None, Some(0.0), Some(-0.0), Some(0.35), Some(1.0), None]
        .into_iter()
        .enumerate()
    {
        let base = scene.input();
        let mut frame = UpscaleDispatch::new(
            base.command_list(),
            base.resources().clone(),
            base.jitter(),
            base.frame_time(),
            base.camera(),
            index == 0,
        );
        let selection = setting.map_or(Sharpening::Disabled, |v| {
            Sharpening::Native(NativeSharpness::new(v).unwrap())
        });
        // Replacing an enabled selection with Disabled must also clear strength.
        frame.set_sharpening(Sharpening::Native(NativeSharpness::new(1.0).unwrap()));
        frame.set_sharpening(selection);
        let borrowed: &Sharpening = frame.sharpening();
        let owned = borrowed.clone();
        frame.set_sharpening(owned);
        assert_eq!(calls(), index as u32);
        let counts = scene.reference_counts();
        // SAFETY: Controlled COM/runtime fixtures record no GPU commands.
        assert_eq!(unsafe { upscaler.dispatch(frame) }, Ok(()));
        assert_eq!(scene.reference_counts(), counts);
        let descriptor = CAPTURED.with(|captured| captured.borrow_mut().take().unwrap());
        assert_eq!(descriptor.enableSharpening, setting.is_some());
        assert_eq!(
            descriptor.sharpness.to_bits(),
            setting.unwrap_or(0.0).to_bits()
        );
        assert_eq!(descriptor.reset, index == 0);
        assert_eq!(descriptor.commandList, scene.list.as_raw());
        for (native, resource) in [
            descriptor.color,
            descriptor.depth,
            descriptor.motionVectors,
            descriptor.exposure,
            descriptor.output,
        ]
        .into_iter()
        .zip(&scene.resources)
        {
            assert_eq!(native.resource, resource.as_raw());
        }
        assert_eq!(descriptor.flags, 0);
        assert_eq!(descriptor.preExposure, 1.0);
        assert_eq!(calls(), index as u32 + 1);
    }
    set_code(u32::MAX);
    let mut frame = scene.input();
    frame.set_sharpening(Sharpening::Native(NativeSharpness::new(0.35).unwrap()));
    // SAFETY: The controlled fixture records no GPU work, including on error.
    assert_eq!(
        unsafe { upscaler.dispatch(frame) },
        Err(Error::Native {
            operation: Operation::Dispatch,
            code: u32::MAX,
        })
    );
    assert_eq!(calls(), 7);
    scene
        .list_object()
        .get_device_failure
        .set(Some(super::com::GET_DEVICE_FAILURE));
    // SAFETY: Poison-first rejection prevents COM inspection and native dispatch.
    assert_eq!(
        unsafe { upscaler.dispatch(scene.input()) },
        Err(Error::DispatchPoisoned)
    );
    assert_eq!(calls(), 7);
    upscaler.destroy().unwrap();
    assert_eq!(fixture_snapshot(&observer).destroys, 1);
}

#[test]
fn resource_role_swaps_reject_before_ffi_and_preserve_compatible_pointer_mapping() {
    use windows::Win32::Graphics::Direct3D12::D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
    let (runtime, observer) = fixture_runtime();
    let scene = Scene::new();
    // Both RGBA16F roles allow UAV access so the unequal-size swap reaches
    // the extent check rather than failing earlier on output capabilities.
    let mut color = scene.resource_object(0).desc.get();
    color.Flags = D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
    scene.resource_object(0).desc.set(color);
    let mut upscaler = Upscaler::new(&runtime, &scene.device, sizes()).unwrap();
    for (roles, role, property) in [
        (
            [2, 1, 0, 3, 4],
            ResourceRole::Color,
            ResourceProperty::Format,
        ),
        (
            [0, 3, 2, 1, 4],
            ResourceRole::Depth,
            ResourceProperty::Extent,
        ),
        (
            [4, 1, 2, 3, 0],
            ResourceRole::Color,
            ResourceProperty::Extent,
        ),
    ] {
        let base = scene.input();
        let resources = UpscaleResources::new(
            &scene.resources[roles[0]],
            &scene.resources[roles[1]],
            &scene.resources[roles[2]],
            &scene.resources[roles[3]],
            &scene.resources[roles[4]],
        );
        let frame = UpscaleDispatch::new(
            base.command_list(),
            resources,
            base.jitter(),
            base.frame_time(),
            base.camera(),
            false,
        );
        // SAFETY: Controlled COM/runtime fixtures; rejected calls record no GPU work.
        assert_eq!(
            unsafe { upscaler.dispatch(frame) },
            Err(Error::InvalidDispatch {
                reason: DispatchValidation::Resource { role, property },
            })
        );
    }
    // SAFETY: Exact controlled-fixture export; it only counts calls.
    let observe = unsafe {
        observer
            .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
            .unwrap()
    };
    assert_eq!(observe().calls, 0);
    // SAFETY: Rejections leave this fixture context usable; no GPU work occurs.
    assert_eq!(unsafe { upscaler.dispatch(scene.input()) }, Ok(()));
    assert_eq!(observe().calls, 1);
    upscaler.destroy().unwrap();

    let mut output = scene.resource_object(4).desc.get();
    output.Width = color.Width;
    output.Height = color.Height;
    scene.resource_object(4).desc.set(output);
    let options = UpscalerOptions::new(
        Dimensions::new(1280, 720).unwrap(),
        Dimensions::new(1280, 720).unwrap(),
    );
    let base = scene.input();
    let swapped = UpscaleResources::new(
        &scene.resources[4],
        &scene.resources[1],
        &scene.resources[2],
        &scene.resources[3],
        &scene.resources[0],
    );
    let frame = UpscaleDispatch::new(
        base.command_list(),
        swapped,
        base.jitter(),
        base.frame_time(),
        base.camera(),
        false,
    );
    // Metadata alone cannot establish semantic roles; inspect only, never run
    // this deliberately swapped case against the AMD runtime or a GPU.
    let descriptor =
        super::super::dispatch::inspect_dispatch(&scene.device, &options, frame).unwrap();
    assert_eq!(descriptor.color.resource, scene.resources[4].as_raw());
    assert_eq!(descriptor.output.resource, scene.resources[0].as_raw());
}
