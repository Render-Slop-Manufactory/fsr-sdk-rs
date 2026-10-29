// SPDX-License-Identifier: MPL-2.0

#[test]
#[ignore = "trusted native GPU timing pilot; use experiment-gpu-timing"]
fn native_gpu_timing() {
    super::performance_comparison::run(true);
}
