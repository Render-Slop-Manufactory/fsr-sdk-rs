// SPDX-License-Identifier: MPL-2.0

#![cfg(all(windows, feature = "dx12"))]

use fsr_sdk::{
    error::{DispatchScalar, DispatchValidation, Error},
    upscaler::{NativeSharpness, Sharpening, UpscaleDispatch, UpscaleResources, dx12},
};

macro_rules! not_impl {
    ($ty:ty, $bound:path) => {
        const _: fn() = || {
            trait Ambiguous<A> {
                fn check() {}
            }
            impl<T: ?Sized> Ambiguous<()> for T {}
            struct Implemented;
            impl<T: ?Sized + $bound> Ambiguous<Implemented> for T {}
            let _ = <$ty as Ambiguous<_>>::check;
        };
    };
}
not_impl!(UpscaleDispatch<'static>, Copy);
not_impl!(UpscaleDispatch<'static>, Clone);
not_impl!(UpscaleResources<'static>, Copy);
not_impl!(Sharpening, Copy);

#[test]
fn native_parameter_preserves_bits_and_rejects_invalid_values() {
    for input in [0.0_f32, -0.0, f32::from_bits(1), 0.35, 1.0] {
        let value = NativeSharpness::new(input).unwrap();
        assert_eq!(value.value().to_bits(), input.to_bits());
    }
    for input in [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        -1.0,
        2.0,
        -f32::from_bits(1),
        f32::from_bits(1.0_f32.to_bits() + 1),
    ] {
        assert_eq!(
            NativeSharpness::new(input),
            Err(Error::InvalidDispatch {
                reason: DispatchValidation::Scalar(DispatchScalar::NativeSharpness),
            })
        );
    }
}

#[test]
fn public_traits_reexports_and_non_exhaustive_selection() {
    fn send_sync<T: Send + Sync>() {}
    fn clone_debug<T: Clone + std::fmt::Debug>() {}
    fn scalar<T: Copy + PartialEq>() {}
    send_sync::<UpscaleDispatch<'static>>();
    send_sync::<UpscaleResources<'static>>();
    send_sync::<NativeSharpness>();
    send_sync::<Sharpening>();
    clone_debug::<UpscaleResources<'static>>();
    clone_debug::<Sharpening>();
    clone_debug::<NativeSharpness>();
    scalar::<NativeSharpness>();

    fn inspect(value: &Sharpening) -> Option<f32> {
        match value {
            Sharpening::Disabled => None,
            Sharpening::Native(value) => Some(value.value()),
            _ => None,
        }
    }
    let selected: dx12::Sharpening = Sharpening::Native(dx12::NativeSharpness::new(0.0).unwrap());
    assert_eq!(inspect(&selected), Some(0.0));
    assert_eq!(inspect(&Sharpening::Disabled), None);
    let _: Option<dx12::UpscaleDispatch<'static>> = None::<UpscaleDispatch<'static>>;
    let _: Option<dx12::UpscaleResources<'static>> = None::<UpscaleResources<'static>>;
}
