// SPDX-License-Identifier: MPL-2.0

use super::super::*;

macro_rules! assert_not_impl {
    ($bound:path) => {
        const _: fn() = || {
            trait Ambiguous<A> {
                fn check() {}
            }
            impl<T: ?Sized> Ambiguous<()> for T {}
            struct IfImplemented;
            impl<T: ?Sized + $bound> Ambiguous<IfImplemented> for T {}
            let _ = <Upscaler as Ambiguous<_>>::check;
        };
    };
}
assert_not_impl!(Send);
assert_not_impl!(Sync);
assert_not_impl!(Copy);
assert_not_impl!(Clone);

macro_rules! assert_runtime_not_impl {
    ($bound:path) => {
        const _: fn() = || {
            trait Ambiguous<A> {
                fn check() {}
            }
            impl<T: ?Sized> Ambiguous<()> for T {}
            struct IfImplemented;
            impl<T: ?Sized + $bound> Ambiguous<IfImplemented> for T {}
            let _ = <Runtime as Ambiguous<_>>::check;
        };
    };
}
assert_runtime_not_impl!(Send);
assert_runtime_not_impl!(Sync);
assert_runtime_not_impl!(Copy);
assert_runtime_not_impl!(Clone);
