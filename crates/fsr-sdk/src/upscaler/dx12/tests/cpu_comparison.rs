// SPDX-License-Identifier: MPL-2.0

#[test]
#[ignore = "trusted native CPU experiment; use experiment-cpu-comparison"]
fn native_cpu_comparison() {
    super::performance_comparison::run(false);
}
