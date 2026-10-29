// SPDX-License-Identifier: MPL-2.0

// Submission/readback equipment ABI; contains no AMD API declarations or policy.
#pragma once
#include <d3d12.h>
#include <cstdint>

struct CpuMetrics {
    uint32_t width, height, finite, changed;
    float min, max, left, right;
    uint64_t errors;
    int32_t removed;
};

extern "C" {
__declspec(dllimport) int32_t gpu_enable_debug();
__declspec(dllimport) int32_t probe_create_device(ID3D12Device**);
__declspec(dllimport) int32_t gpu_require_amd(ID3D12Device*);
__declspec(dllimport) int32_t gpu_create(ID3D12Device*, void**, ID3D12GraphicsCommandList**, ID3D12Resource**);
__declspec(dllimport) int32_t performance_prepare_readback(void*);
__declspec(dllimport) int32_t performance_finish(void*, uint32_t, CpuMetrics*);
__declspec(dllimport) int64_t performance_clock();
__declspec(dllimport) int64_t performance_frequency();
__declspec(dllimport) int32_t gpu_copy_payload(void*, uint8_t*, uint32_t);
__declspec(dllimport) void gpu_destroy(void*);
__declspec(dllimport) int32_t gpu_check_device(ID3D12Device*);
__declspec(dllimport) int32_t gpu_check_device_removed(ID3D12Device*);
#ifdef FSR_GPU_TIMING
__declspec(dllimport) int32_t gpu_timing_configure(uint32_t, uint32_t);
__declspec(dllimport) int32_t gpu_timing_begin(void*, uint32_t, float, float, uint32_t);
__declspec(dllimport) int32_t gpu_timing_finish(void*, uint32_t, CpuMetrics*, uint64_t*);
__declspec(dllimport) int32_t active_timing_begin(void*, uint32_t, uint32_t, uint32_t, float, float, uint32_t);
__declspec(dllimport) int32_t active_timing_hashes(void*, uint64_t*);
__declspec(dllimport) int32_t feature_timing_configure(void*, uint32_t);
__declspec(dllimport) int32_t gpu_create_masks(void*, uint32_t, uint32_t, uint32_t, float, float, ID3D12Resource**);
#endif
}
