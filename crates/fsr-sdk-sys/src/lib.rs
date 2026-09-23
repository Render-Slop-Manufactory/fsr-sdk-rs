// SPDX-License-Identifier: MPL-2.0

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub mod api;
pub mod upscale;

#[cfg(all(windows, feature = "dx12"))]
pub mod loader;

#[cfg(all(windows, feature = "dx12"))]
pub mod dx12;

#[cfg(feature = "vulkan")]
compile_error!("The 'vulkan' backend is not implemented yet.");
