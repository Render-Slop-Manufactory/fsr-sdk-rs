// SPDX-License-Identifier: MPL-2.0

use super::{super::*, support::*};

#[test]
fn public_constructor_forwards_checked_chain_and_owns_device() {
    let (mut runtime, observer) = fixture_runtime();
    let (device, fake) = fake_device();
    let weak = std::rc::Rc::downgrade(&runtime.state);
    let samples = [
        sizes(),
        UpscalerOptions::new(
            Dimensions::new(1, 2).unwrap(),
            Dimensions::new(3, 4).unwrap(),
        ),
        UpscalerOptions::new(
            Dimensions::new(2048, 1024).unwrap(),
            Dimensions::new(1024, 512).unwrap(),
        ),
    ];
    for (index, options) in samples.into_iter().enumerate() {
        let context = unsafe { runtime.upscaler_dx12(&device) }
            .create(&options)
            .unwrap();
        assert_eq!(fake.references.get(), 2);
        let snapshot = fixture_snapshot(&observer);
        assert_eq!(snapshot.creates, (index + 1) as u32);
        assert_eq!(snapshot.root_tag, FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE);
        assert_eq!(
            snapshot.version_tag,
            FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION
        );
        assert_eq!(
            snapshot.backend_tag,
            FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12
        );
        assert_eq!(snapshot.version, FFX_UPSCALER_VERSION);
        assert_eq!(snapshot.flags, 0);
        assert_eq!(snapshot.chain_valid, 1);
        assert_eq!(snapshot.device, device.as_raw() as usize);
        assert_eq!(snapshot.render_width, options.max_render_size().width());
        assert_eq!(snapshot.render_height, options.max_render_size().height());
        assert_eq!(snapshot.upscale_width, options.max_upscale_size().width());
        assert_eq!(snapshot.upscale_height, options.max_upscale_size().height());
        context.destroy().unwrap();
        assert_eq!(fake.references.get(), 1);
        assert_eq!(fixture_snapshot(&observer).destroys, (index + 1) as u32);
    }
    let a = unsafe { runtime.upscaler_dx12(&device) }
        .create(&sizes())
        .unwrap();
    let b = unsafe { runtime.upscaler_dx12(&device) }
        .create(&sizes())
        .unwrap();
    drop(runtime);
    drop(device);
    assert_eq!(fake.references.get(), 2);
    assert!(weak.upgrade().is_some());
    a.destroy().unwrap();
    assert_eq!(fake.references.get(), 1);
    assert!(weak.upgrade().is_some());
    drop(b);
    assert_eq!(fake.references.get(), 0);
    assert!(weak.upgrade().is_none());
}

#[test]
fn zero_is_rejected_without_entering_native_create() {
    let (_runtime, observer) = fixture_runtime();
    for pair in [(0, 0), (0, 1), (1, 0)] {
        assert!(Dimensions::new(pair.0, pair.1).is_err());
    }
    assert_eq!(fixture_snapshot(&observer).creates, 0);
}

#[test]
fn public_failure_paths_preserve_com_and_runtime_policy() {
    for (create_code, destroy_code, null_create) in [(u32::MAX, 0, 0), (0, 0, 1), (0, u32::MAX, 0)]
    {
        let (mut runtime, observer) = fixture_runtime();
        let (device, fake) = fake_device();
        let weak = std::rc::Rc::downgrade(&runtime.state);
        fixture_set(&observer, create_code, destroy_code, null_create);
        let result = unsafe { runtime.upscaler_dx12(&device) }.create(&sizes());
        if create_code != 0 {
            assert!(matches!(
                result,
                Err(Error::Native(crate::error::NativeError {
                    operation: crate::error::Operation::Create,
                    code: u32::MAX
                }))
            ));
            assert_eq!(fake.references.get(), 1);
            assert_eq!(fixture_snapshot(&observer).destroys, 0);
        } else if null_create != 0 {
            assert!(matches!(
                result,
                Err(Error::Invariant {
                    operation: _,
                    reason: _
                })
            ));
            assert_eq!(fake.references.get(), 2);
            assert_eq!(fixture_snapshot(&observer).destroys, 0);
        } else {
            assert_result!(
                result.unwrap().destroy(),
                Err(Error::native(crate::error::Operation::Destroy, u32::MAX))
            );
            assert_eq!(fake.references.get(), 2);
            assert_eq!(fixture_snapshot(&observer).destroys, 1);
        }
        drop(device);
        drop(runtime);
        assert_eq!(weak.upgrade().is_some(), create_code == 0);
        assert_eq!(fake.references.get(), u32::from(create_code == 0));
    }
}

#[test]
fn public_siblings_survive_failed_create_and_destroy() {
    let (mut runtime, observer) = fixture_runtime();
    let (device, fake) = fake_device();
    let weak = std::rc::Rc::downgrade(&runtime.state);
    let a = unsafe { runtime.upscaler_dx12(&device) }
        .create(&sizes())
        .unwrap();
    fixture_set(&observer, u32::MAX, 0, 0);
    assert!(matches!(
        unsafe { runtime.upscaler_dx12(&device) }.create(&sizes()),
        Err(Error::Native(crate::error::NativeError {
            operation: crate::error::Operation::Create,
            code: u32::MAX
        }))
    ));
    assert_eq!(fake.references.get(), 2); // caller + A; failed sibling released
    fixture_set(&observer, 0, u32::MAX, 0);
    let b = unsafe { runtime.upscaler_dx12(&device) }
        .create(&sizes())
        .unwrap();
    drop(runtime);
    drop(device);
    assert_eq!(fake.references.get(), 2); // A + B
    assert_result!(
        a.destroy(),
        Err(Error::native(crate::error::Operation::Destroy, u32::MAX))
    );
    fixture_set(&observer, 0, 0, 0);
    b.destroy().unwrap();
    assert_eq!(fixture_snapshot(&observer).destroys, 2);
    assert_eq!(fake.references.get(), 1); // A retained after destroy failure
    assert!(weak.upgrade().is_some());
}

#[test]
fn descriptor_links_survive_moves_of_outer_storage() {
    // Address sentinel only; no native call and no device dereference.
    let device = std::ptr::dangling_mut::<fsr_sdk_sys::dx12::ID3D12Device>();
    let (state, root) = UpscaleCreateState::new(device, &sizes(), Default::default(), None);
    let addresses = (
        &raw const state.root,
        &raw const state.version,
        &raw const state.backend,
    );
    let outer = Box::new(Some(state));
    let moved = (*outer).unwrap();
    assert_eq!(
        addresses,
        (
            &raw const moved.root,
            &raw const moved.version,
            &raw const moved.backend
        )
    );
    assert_eq!(root, (&raw const moved.root.header).cast_mut());
    assert_eq!(
        moved.root.header.pNext,
        (&raw const moved.version.header).cast_mut()
    );
    assert_eq!(
        moved.version.header.pNext,
        (&raw const moved.backend.header).cast_mut()
    );
    assert!(moved.backend.header.pNext.is_null());
    assert_eq!(moved.backend.device, device);
    assert_eq!(
        moved.root.header.r#type,
        FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE
    );
    assert_eq!(
        moved.version.header.r#type,
        FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION
    );
    assert_eq!(
        moved.backend.header.r#type,
        FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12
    );
    assert_eq!(moved.version.version, FFX_UPSCALER_VERSION);
    assert_eq!(moved.root.flags, 0);
    assert!(moved.root.fpMessage.is_none());
    assert_eq!(moved.root.maxRenderSize.width, 1280);
    assert_eq!(moved.root.maxRenderSize.height, 720);
    assert_eq!(moved.root.maxUpscaleSize.width, 1920);
    assert_eq!(moved.root.maxUpscaleSize.height, 1080);
}

#[test]
fn dimensions_reject_every_zero_component() {
    for (width, height) in [(0, 0), (0, 720), (1280, 0)] {
        assert_result!(
            Dimensions::new(width, height),
            Err(Error::Input(crate::error::InputError::InvalidDimensions {
                width,
                height
            }))
        );
    }
    // Both option fields use the same checked type.
    assert!(Dimensions::new(u32::MAX, 1).is_ok());
}
