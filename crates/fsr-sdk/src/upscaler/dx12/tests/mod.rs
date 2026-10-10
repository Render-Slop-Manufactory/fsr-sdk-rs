// SPDX-License-Identifier: MPL-2.0

mod com;
mod construction;
mod dispatch;
mod inspection;
mod lifecycle;
#[cfg(feature = "movable")]
mod movable;
mod sharpening;
mod support;
mod traits;

use super::*;
use support::*;

// Preserve the documented native runner filters while sharing lifecycle equipment.
#[test]
#[ignore = "requires staged trusted AMD v2.3.0 DLLs and the native DX12 helper; use tests/run-m5.ps1"]
fn native_lifecycle() {
    lifecycle::native_lifecycle_case(false);
}

#[test]
#[ignore = "requires staged trusted AMD v2.3.0 DLLs and the native DX12 helper; use tests/run-m5.ps1"]
fn native_lifecycle_drop() {
    lifecycle::native_lifecycle_case(true);
}
