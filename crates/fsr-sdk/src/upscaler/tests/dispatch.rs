// SPDX-License-Identifier: MPL-2.0

use super::{super::*, support::*};

#[test]
fn private_dispatch_forwards_once_and_preserves_unknown_native_code() {
    let (runtime, observer) = fixture_runtime();
    let (device, _) = fake_device();
    let mut upscaler = Upscaler::new(&runtime, &device, sizes()).unwrap();
    // SAFETY: These exports are provided by the local fixture, which only
    // snapshots the descriptor header and records no GPU commands.
    unsafe {
        let set = observer
            .get::<extern "C" fn(u32)>(c"fixture_set_dispatch_code")
            .unwrap();
        let snapshot = observer
            .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
            .unwrap();
        set(u32::MAX);
        let header = ffxApiHeader {
            r#type: FFX_API_DISPATCH_DESC_TYPE_UPSCALE,
            pNext: null_mut(),
        };
        assert_eq!(upscaler.owner.dispatch(&header), u32::MAX);
        let result = snapshot();
        assert_eq!(result.calls, 1);
        assert_eq!(result.tag, FFX_API_DISPATCH_DESC_TYPE_UPSCALE);
    }
    upscaler.destroy().unwrap();
}

#[test]
fn public_dispatch_core_preserves_preflight_errors_and_poison_policy() {
    use crate::error::{DispatchValidation, InspectionObject, InspectionOperation, Operation};
    let (runtime, observer) = fixture_runtime();
    let (device, fake) = fake_device();
    let mut upscaler = Upscaler::new(&runtime, &device, sizes()).unwrap();
    // The local fixture records no GPU commands. A descriptor header is enough
    // to exercise the terminal policy after the separated inspection phase.
    let descriptor = || {
        // SAFETY: The fixture reads only the header; zero is a valid bit pattern
        // for this selected ABI descriptor and its pointers.
        let mut desc: ffxDispatchDescUpscale = unsafe { std::mem::zeroed() };
        desc.header.r#type = FFX_API_DISPATCH_DESC_TYPE_UPSCALE;
        desc
    };
    let observation = || -> DispatchObservation {
        // SAFETY: The locally compiled fixture exports this exact signature.
        unsafe {
            observer
                .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
                .unwrap()()
        }
    };
    let set_code = |code| {
        // SAFETY: The locally compiled fixture exports this exact signature.
        unsafe {
            observer
                .get::<extern "C" fn(u32)>(c"fixture_set_dispatch_code")
                .unwrap()(code)
        }
    };
    let invalid = Error::InvalidDispatch {
        reason: DispatchValidation::CommandListType,
    };
    // SAFETY: No native call occurs for the injected preflight error.
    assert_eq!(
        unsafe { upscaler.dispatch_core(|_| Err(invalid)) },
        Err(invalid)
    );
    assert_eq!(observation().calls, 0);
    let com = Error::Dx12Inspection {
        object: InspectionObject::CommandList,
        operation: InspectionOperation::GetDevice,
        hresult: 0x8000_4005u32 as i32,
    };
    // SAFETY: No native call occurs for the injected COM inspection error.
    assert_eq!(unsafe { upscaler.dispatch_core(|_| Err(com)) }, Err(com));
    assert_eq!(observation().calls, 0);
    set_code(0);
    // SAFETY: This fixture records no GPU commands and reads only the header.
    assert_eq!(
        unsafe { upscaler.dispatch_core(|_| Ok(descriptor())) },
        Ok(())
    );
    assert_eq!(observation().calls, 1);
    assert_eq!(observation().tag, FFX_API_DISPATCH_DESC_TYPE_UPSCALE);
    set_code(u32::MAX);
    // SAFETY: This fixture records no GPU commands and reads only the header.
    assert_eq!(
        unsafe { upscaler.dispatch_core(|_| Ok(descriptor())) },
        Err(Error::Native {
            operation: Operation::Dispatch,
            code: u32::MAX
        })
    );
    assert_eq!(observation().calls, 2);
    // SAFETY: Poison stops inspection and native dispatch before any GPU work.
    assert_eq!(
        unsafe { upscaler.dispatch_core(|_| panic!("inspected poisoned context")) },
        Err(Error::DispatchPoisoned)
    );
    assert_eq!(observation().calls, 2);
    upscaler.destroy().unwrap();
    assert_eq!(fixture_snapshot(&observer).destroys, 1);
    assert_eq!(fake.references.get(), 1);
}

#[test]
fn poisoned_dispatch_still_uses_terminal_destroy_retention() {
    use crate::error::Operation;
    let (runtime, observer) = fixture_runtime();
    let (device, fake) = fake_device();
    let weak = std::rc::Rc::downgrade(&runtime.state);
    let mut upscaler = Upscaler::new(&runtime, &device, sizes()).unwrap();
    fixture_set(&observer, 0, u32::MAX, 0);
    // SAFETY: Exact signature from the local fixture.
    unsafe {
        observer
            .get::<extern "C" fn(u32)>(c"fixture_set_dispatch_code")
            .unwrap()(u32::MAX)
    };
    // SAFETY: Fixture records no GPU work; descriptor header is sufficient.
    assert!(matches!(
        unsafe {
            upscaler.dispatch_core(|_| {
                let mut desc: ffxDispatchDescUpscale = std::mem::zeroed();
                desc.header.r#type = FFX_API_DISPATCH_DESC_TYPE_UPSCALE;
                Ok(desc)
            })
        },
        Err(Error::Native {
            operation: Operation::Dispatch,
            code: u32::MAX
        })
    ));
    assert_eq!(
        upscaler.destroy(),
        Err(Error::Native {
            operation: Operation::Destroy,
            code: u32::MAX
        })
    );
    assert_eq!(fixture_snapshot(&observer).destroys, 1);
    drop(device);
    drop(runtime);
    assert_eq!(fake.references.get(), 1);
    assert!(weak.upgrade().is_some());
}
