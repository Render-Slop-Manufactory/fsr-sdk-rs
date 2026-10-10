// SPDX-License-Identifier: MPL-2.0

#![cfg(all(windows, feature = "dx12"))]

use fsr_sdk::{
    error::Error,
    upscaler::{Dimensions, UpscalerOptions},
};

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
