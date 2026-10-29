// SPDX-License-Identifier: MPL-2.0

// Opt-in GPU laboratory for the public Rust dispatch path. No AMD DLL
// calls occur here; the included inline helper is used only for parity checks.
#define NOMINMAX
#include <d3d12.h>
#include <dxgi1_4.h>
#include <wrl/client.h>
#include <algorithm>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <memory>
#include <vector>
#include "ffx_api_dx12.h"
#ifdef FSR_FORMAT_PROBE
#include <d3dcompiler.h>
#endif

using Microsoft::WRL::ComPtr;

namespace {
#ifdef FSR_GPU_TIMING
// Dedicated timing-helper build: configured once, before creating its harness.
uint32_t render_width = 320;
uint32_t render_height = 180;
constexpr uint32_t output_width = 1920;
constexpr uint32_t output_height = 1080;
#else
constexpr uint32_t render_width = 320;
constexpr uint32_t render_height = 180;
constexpr uint32_t output_width = 640;
constexpr uint32_t output_height = 360;
#endif
constexpr uint16_t sentinel_half = 0x7bff; // 65504, unlike expected output
enum Role { Color, Depth, Motion, Exposure, Output, Count };

struct Harness {
#ifdef FSR_REACTIVE_PROBE
    ComPtr<ID3D12Resource> probe_opaque, probe_opaque_upload;
    ComPtr<ID3D12Resource> probe_reactive, probe_seed, probe_readback;
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT probe_footprint{};
    UINT64 probe_bytes = 0;
    uint32_t probe_width = 0, probe_height = 0;
#endif
#ifdef FSR_GPU_TIMING
    ComPtr<ID3D12QueryHeap> timestamps;
    ComPtr<ID3D12Resource> timestamp_readback;
    UINT64 timestamp_frequency = 0;
    uint32_t timing_frame = 0, timing_width = 0, timing_height = 0;
    uint32_t timing_feature = 0;
    float timing_max_red = 0;
#endif
    UINT64 performance_fence = 0;
    bool display_motion = false;
    bool dynamic_render = false;
    bool hdr = false;
    ComPtr<ID3D12Resource> masks[2];
    ComPtr<ID3D12Resource> mask_uploads[2];
    ComPtr<ID3D12Resource> mask_readbacks[2];
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT mask_footprints[2]{};
    UINT64 mask_bytes[2]{};
    ComPtr<ID3D12Resource> input_readbacks[4];
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT input_footprints[4]{};
    UINT64 input_bytes[4]{};
    ComPtr<ID3D12Device> device;
    ComPtr<ID3D12CommandQueue> queue;
    ComPtr<ID3D12CommandAllocator> allocator;
    ComPtr<ID3D12GraphicsCommandList> list;
    ComPtr<ID3D12Resource> textures[Count];
    ComPtr<ID3D12Resource> uploads[Count];
    ComPtr<ID3D12Resource> readback;
    ComPtr<ID3D12Resource> calibration_inputs[4];
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT calibration_footprints[4]{};
    UINT64 calibration_bytes[4]{};
    ComPtr<ID3D12Fence> fence;
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT output_footprint{};
    UINT64 output_bytes = 0;
};

struct Metrics {
    uint32_t width;
    uint32_t height;
    uint32_t finite_pixels;
    uint32_t changed_pixels;
    float min_red;
    float max_red;
    float left_red;
    float right_red;
    uint64_t debug_errors;
    int32_t device_removed;
};

D3D12_HEAP_PROPERTIES heap_properties(D3D12_HEAP_TYPE type) {
    D3D12_HEAP_PROPERTIES properties{};
    properties.Type = type;
    properties.CPUPageProperty = D3D12_CPU_PAGE_PROPERTY_UNKNOWN;
    properties.MemoryPoolPreference = D3D12_MEMORY_POOL_UNKNOWN;
    properties.CreationNodeMask = 1;
    properties.VisibleNodeMask = 1;
    return properties;
}

D3D12_RESOURCE_DESC buffer_desc(UINT64 size) {
    D3D12_RESOURCE_DESC desc{};
    desc.Dimension = D3D12_RESOURCE_DIMENSION_BUFFER;
    desc.Width = size;
    desc.Height = 1;
    desc.DepthOrArraySize = 1;
    desc.MipLevels = 1;
    desc.Format = DXGI_FORMAT_UNKNOWN;
    desc.SampleDesc.Count = 1;
    desc.Layout = D3D12_TEXTURE_LAYOUT_ROW_MAJOR;
    return desc;
}

D3D12_RESOURCE_DESC texture_desc(Role role, bool display_motion = false) {
    D3D12_RESOURCE_DESC desc{};
    desc.Dimension = D3D12_RESOURCE_DIMENSION_TEXTURE2D;
    const bool display = role == Output || (role == Motion && display_motion);
    desc.Width = display ? output_width : role == Exposure ? 1 : render_width;
    desc.Height = display ? output_height : role == Exposure ? 1 : render_height;
    desc.DepthOrArraySize = 1;
    desc.MipLevels = 1;
    desc.Format = role == Color || role == Output
        ? DXGI_FORMAT_R16G16B16A16_FLOAT
        : role == Motion ? DXGI_FORMAT_R16G16_FLOAT : DXGI_FORMAT_R32_FLOAT;
    desc.SampleDesc.Count = 1;
    desc.Layout = D3D12_TEXTURE_LAYOUT_UNKNOWN;
    desc.Flags = role == Output ? D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS
                                : D3D12_RESOURCE_FLAG_NONE;
    return desc;
}

#include "images.h"

void transition(ID3D12GraphicsCommandList* list, ID3D12Resource* resource,
                D3D12_RESOURCE_STATES before, D3D12_RESOURCE_STATES after) {
    D3D12_RESOURCE_BARRIER barrier{};
    barrier.Type = D3D12_RESOURCE_BARRIER_TYPE_TRANSITION;
    barrier.Transition.pResource = resource;
    barrier.Transition.Subresource = D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES;
    barrier.Transition.StateBefore = before;
    barrier.Transition.StateAfter = after;
    list->ResourceBarrier(1, &barrier);
}

HRESULT prepare_texture(Harness& h, Role role) {
    const auto desc = texture_desc(role, h.display_motion);
    const auto default_heap = heap_properties(D3D12_HEAP_TYPE_DEFAULT);
    HRESULT hr = h.device->CreateCommittedResource(
        &default_heap, D3D12_HEAP_FLAG_NONE, &desc, D3D12_RESOURCE_STATE_COPY_DEST,
        nullptr, IID_PPV_ARGS(&h.textures[role]));
    if (FAILED(hr)) return hr;
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint{};
    UINT64 bytes = 0;
    h.device->GetCopyableFootprints(&desc, 0, 1, 0, &footprint, nullptr, nullptr, &bytes);
    const auto upload_desc = buffer_desc(bytes);
    const auto upload_heap = heap_properties(D3D12_HEAP_TYPE_UPLOAD);
    hr = h.device->CreateCommittedResource(
        &upload_heap, D3D12_HEAP_FLAG_NONE, &upload_desc,
        D3D12_RESOURCE_STATE_GENERIC_READ, nullptr, IID_PPV_ARGS(&h.uploads[role]));
    if (FAILED(hr)) return hr;
    uint8_t* mapped = nullptr;
    hr = h.uploads[role]->Map(0, nullptr, reinterpret_cast<void**>(&mapped));
    if (FAILED(hr)) return hr;
    std::memset(mapped, 0, static_cast<size_t>(bytes));
    for (uint32_t y = 0; y < desc.Height; ++y) {
        fill_row(role, y, mapped + footprint.Offset + y * footprint.Footprint.RowPitch,
                 0.0f, h.display_motion);
    }
    h.uploads[role]->Unmap(0, nullptr);
    D3D12_TEXTURE_COPY_LOCATION target{};
    target.pResource = h.textures[role].Get();
    target.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
    target.SubresourceIndex = 0;
    D3D12_TEXTURE_COPY_LOCATION source{};
    source.pResource = h.uploads[role].Get();
    source.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
    source.PlacedFootprint = footprint;
    h.list->CopyTextureRegion(&target, 0, 0, 0, &source, nullptr);
    transition(h.list.Get(), h.textures[role].Get(), D3D12_RESOURCE_STATE_COPY_DEST,
        role == Output ? D3D12_RESOURCE_STATE_UNORDERED_ACCESS
                       : D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE);
    return S_OK;
}

uint64_t debug_error_count(ID3D12Device* device) {
    ComPtr<ID3D12InfoQueue> info;
    if (FAILED(device->QueryInterface(IID_PPV_ARGS(&info)))) return UINT64_MAX;
    uint64_t errors = 0;
    const uint64_t count = info->GetNumStoredMessages();
    for (uint64_t index = 0; index < count; ++index) {
        SIZE_T bytes = 0;
        if (FAILED(info->GetMessage(index, nullptr, &bytes))) return UINT64_MAX;
        std::vector<uint8_t> storage(bytes);
        auto* message = reinterpret_cast<D3D12_MESSAGE*>(storage.data());
        if (FAILED(info->GetMessage(index, message, &bytes))) return UINT64_MAX;
        if (message->Severity == D3D12_MESSAGE_SEVERITY_ERROR
            || message->Severity == D3D12_MESSAGE_SEVERITY_CORRUPTION) {
            ++errors;
            std::fprintf(stderr, "D3D12 error: id=%u severity=%u text=%s\n",
                         static_cast<unsigned>(message->ID),
                         static_cast<unsigned>(message->Severity), message->pDescription);
        }
    }
    return errors;
}

HRESULT record_readback(Harness& h) {
    const auto desc = texture_desc(Output);
    h.device->GetCopyableFootprints(&desc, 0, 1, 0, &h.output_footprint,
                                    nullptr, nullptr, &h.output_bytes);
    const auto readback_desc = buffer_desc(h.output_bytes);
    const auto readback_heap = heap_properties(D3D12_HEAP_TYPE_READBACK);
    HRESULT hr = h.device->CreateCommittedResource(
        &readback_heap, D3D12_HEAP_FLAG_NONE, &readback_desc,
        D3D12_RESOURCE_STATE_COPY_DEST, nullptr, IID_PPV_ARGS(&h.readback));
    if (FAILED(hr)) return hr;
    transition(h.list.Get(), h.textures[Output].Get(),
               D3D12_RESOURCE_STATE_UNORDERED_ACCESS, D3D12_RESOURCE_STATE_COPY_SOURCE);
    D3D12_TEXTURE_COPY_LOCATION source{};
    source.pResource = h.textures[Output].Get();
    source.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
    source.SubresourceIndex = 0;
    D3D12_TEXTURE_COPY_LOCATION target{};
    target.pResource = h.readback.Get();
    target.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
    target.PlacedFootprint = h.output_footprint;
    h.list->CopyTextureRegion(&target, 0, 0, 0, &source, nullptr);
    return S_OK;
}

HRESULT wait_for_fence(Harness& h) {
    HANDLE event = CreateEventW(nullptr, FALSE, FALSE, nullptr);
    if (!event) return HRESULT_FROM_WIN32(GetLastError());
    HRESULT hr = h.fence->SetEventOnCompletion(1, event);
    if (SUCCEEDED(hr) && WaitForSingleObject(event, 60000) != WAIT_OBJECT_0) {
        hr = HRESULT_FROM_WIN32(ERROR_TIMEOUT);
    }
    CloseHandle(event);
    return hr;
}

HRESULT read_metrics(Harness& h, Metrics* metrics) {
    *metrics = {};
    metrics->width = output_width;
    metrics->height = output_height;
    metrics->min_red = INFINITY;
    metrics->max_red = -INFINITY;
    metrics->device_removed = h.device->GetDeviceRemovedReason();
    metrics->debug_errors = debug_error_count(h.device.Get());
    uint8_t* mapped = nullptr;
    D3D12_RANGE read_range{0, static_cast<SIZE_T>(h.output_bytes)};
    HRESULT hr = h.readback->Map(0, &read_range, reinterpret_cast<void**>(&mapped));
    if (FAILED(hr)) return hr;
    double left_sum = 0.0, right_sum = 0.0;
    for (uint32_t y = 0; y < output_height; ++y) {
        const uint8_t* row = mapped + h.output_footprint.Offset
            + y * h.output_footprint.Footprint.RowPitch;
        for (uint32_t x = 0; x < output_width; ++x) {
            uint16_t pixel[4];
            std::memcpy(pixel, row + x * sizeof(pixel), sizeof(pixel));
            const float red = unhalf(pixel[0]);
            bool finite = true;
            for (uint16_t channel : pixel) finite &= std::isfinite(unhalf(channel));
            metrics->finite_pixels += finite ? 1u : 0u;
            metrics->changed_pixels += (pixel[0] != sentinel_half || pixel[1] != sentinel_half
                || pixel[2] != sentinel_half || pixel[3] != sentinel_half) ? 1u : 0u;
            if (std::isfinite(red)) {
                metrics->min_red = std::min(metrics->min_red, red);
                metrics->max_red = std::max(metrics->max_red, red);
                if (x < output_width / 4) left_sum += red;
                if (x >= output_width * 3 / 4) right_sum += red;
            }
        }
    }
    h.readback->Unmap(0, nullptr);
    const double quarter_pixels = static_cast<double>(output_width / 4) * output_height;
    metrics->left_red = static_cast<float>(left_sum / quarter_pixels);
    metrics->right_red = static_cast<float>(right_sum / quarter_pixels);
    return S_OK;
}
} // namespace

extern "C" __declspec(dllexport) int32_t __cdecl gpu_enable_debug() {
    ComPtr<ID3D12Debug> debug;
    HRESULT hr = D3D12GetDebugInterface(IID_PPV_ARGS(&debug));
    if (FAILED(hr)) return hr;
    debug->EnableDebugLayer();
    // GPU-based validation is a separate opt-in: on the tested driver it can
    // stall inside the closed provider's CPU dispatch path. The debug layer
    // remains enabled for the ordinary native proof.
    char gpu_validation[2]{};
    if (GetEnvironmentVariableA("FSR_SDK_TEST_GBV", gpu_validation, sizeof(gpu_validation)) == 1
        && gpu_validation[0] == '1') {
        ComPtr<ID3D12Debug1> validation;
        hr = debug.As(&validation);
        if (FAILED(hr)) return hr;
        validation->SetEnableGPUBasedValidation(TRUE);
    }
    return S_OK;
}

extern "C" __declspec(dllexport) int32_t __cdecl gpu_require_amd(ID3D12Device* device) {
    if (!device) return E_INVALIDARG;
    ComPtr<IDXGIFactory4> factory;
    HRESULT hr = CreateDXGIFactory1(IID_PPV_ARGS(&factory));
    if (FAILED(hr)) return hr;
    ComPtr<IDXGIAdapter1> adapter;
    hr = factory->EnumAdapterByLuid(device->GetAdapterLuid(), IID_PPV_ARGS(&adapter));
    if (FAILED(hr)) return hr;
    DXGI_ADAPTER_DESC1 desc{};
    hr = adapter->GetDesc1(&desc);
    if (FAILED(hr)) return hr;
    const bool amd = desc.VendorId == 0x1002 && !(desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE);
    std::fprintf(stderr, "Selected-device AMD check: vendor=0x%04x device=0x%04x hardware_amd=%s\n",
                 desc.VendorId, desc.DeviceId, amd ? "true" : "false");
    return amd ? S_OK : DXGI_ERROR_UNSUPPORTED;
}

// Call after context teardown while the caller's device remains alive.
// The explicit Proton case checks device removal independently of unavailable
// Windows diagnostics. gpu_check_device retains the Windows debug requirement.
extern "C" __declspec(dllexport) int32_t __cdecl gpu_check_device_removed(ID3D12Device* device) {
    if (!device) return E_INVALIDARG;
    const HRESULT removed = device->GetDeviceRemovedReason();
    std::fprintf(stderr, "Proton device checkpoint: device_removed=0x%08lx\n",
                 static_cast<unsigned long>(removed));
    return removed;
}

// Call after context teardown while the caller's device remains alive.
extern "C" __declspec(dllexport) int32_t __cdecl gpu_check_device(ID3D12Device* device) {
    if (!device) return E_INVALIDARG;
    const uint64_t errors = debug_error_count(device);
    const HRESULT removed = device->GetDeviceRemovedReason();
    std::fprintf(stderr, "Device checkpoint: debug_errors=%llu device_removed=0x%08lx\n",
                 static_cast<unsigned long long>(errors), static_cast<unsigned long>(removed));
    return removed != S_OK ? removed : errors == 0 ? S_OK : E_FAIL;
}

extern "C" __declspec(dllexport) int32_t __cdecl gpu_set_temporal_frame(
    void* handle, uint32_t frame, float jitter_x, float jitter_y) {
    if (!handle || frame > 2 || !std::isfinite(jitter_x) || !std::isfinite(jitter_y))
        return E_INVALIDARG;
    return set_temporal_frame(*static_cast<Harness*>(handle), frame, jitter_x, jitter_y);
}

static int32_t create_harness(
    ID3D12Device* device, bool display_motion, void** out_harness,
    ID3D12GraphicsCommandList** out_list, ID3D12Resource** out_resources) {
    if (!device || !out_harness || !out_list || !out_resources) return E_INVALIDARG;
    *out_harness = nullptr;
    *out_list = nullptr;
    for (int i = 0; i < Count; ++i) out_resources[i] = nullptr;
    auto h = std::make_unique<Harness>();
    h->display_motion = display_motion;
    h->device = device; // independent COM reference
    D3D12_COMMAND_QUEUE_DESC queue_desc{};
    queue_desc.Type = D3D12_COMMAND_LIST_TYPE_DIRECT;
    HRESULT hr = device->CreateCommandQueue(&queue_desc, IID_PPV_ARGS(&h->queue));
    if (FAILED(hr)) return hr;
    hr = device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT, IID_PPV_ARGS(&h->allocator));
    if (FAILED(hr)) return hr;
    hr = device->CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT,
                                  h->allocator.Get(), nullptr, IID_PPV_ARGS(&h->list));
    if (FAILED(hr)) return hr;
    hr = device->CreateFence(0, D3D12_FENCE_FLAG_NONE, IID_PPV_ARGS(&h->fence));
    if (FAILED(hr)) return hr;
    for (int i = 0; i < Count; ++i) {
        hr = prepare_texture(*h, static_cast<Role>(i));
        if (FAILED(hr)) return hr;
    }
    *out_list = h->list.Get();
    (*out_list)->AddRef();
    for (int i = 0; i < Count; ++i) {
        out_resources[i] = h->textures[i].Get();
        out_resources[i]->AddRef();
    }
    *out_harness = h.release();
    return S_OK;
}

extern "C" __declspec(dllexport) int32_t __cdecl gpu_create(
    ID3D12Device* device, void** out_harness,
    ID3D12GraphicsCommandList** out_list, ID3D12Resource** out_resources) {
    return create_harness(device, false, out_harness, out_list, out_resources);
}

extern "C" __declspec(dllexport) int32_t __cdecl gpu_create_motion_resolution(
    ID3D12Device* device, uint32_t display_motion, void** out_harness,
    ID3D12GraphicsCommandList** out_list, ID3D12Resource** out_resources) {
    if (display_motion > 1) return E_INVALIDARG;
    return create_harness(device, display_motion != 0, out_harness, out_list, out_resources);
}

extern "C" __declspec(dllexport) int32_t __cdecl gpu_compare_resource(
    ID3D12Resource* resource, const FfxApiResource* actual, uint32_t state) {
    if (!resource || !actual) return 0;
    FfxApiResource expected = ffxApiGetResourceDX12(resource, state);
    // The public helper omits the RGB32F cases although the native enum and
    // backend support the fixed FLOAT view. This is an explicit, tested gap.
    if(resource->GetDesc().Format==DXGI_FORMAT_R32G32B32_FLOAT ||
       resource->GetDesc().Format==DXGI_FORMAT_R32G32B32_TYPELESS)
        expected.description.format=FFX_API_SURFACE_FORMAT_R32G32B32_FLOAT;
    const auto& a = actual->description;
    const auto& e = expected.description;
    return actual->resource == expected.resource && actual->state == expected.state
        && a.type == e.type && a.format == e.format && a.width == e.width
        && a.height == e.height && a.depth == e.depth && a.mipCount == e.mipCount
        && a.flags == e.flags && a.usage == e.usage;
}

// Change only this harness's not-yet-submitted upload. Each batch output then
// has a distinct expected red level, exposing stale or aliased descriptors.
extern "C" __declspec(dllexport) int32_t __cdecl gpu_set_color_shift(
    void* handle, float red_shift) {
    if (!handle || !std::isfinite(red_shift) || red_shift < 0.0f || red_shift > 0.12f)
        return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    const auto desc = texture_desc(Color);
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint{};
    h.device->GetCopyableFootprints(&desc, 0, 1, 0, &footprint, nullptr, nullptr, nullptr);
    uint8_t* mapped = nullptr;
    HRESULT hr = h.uploads[Color]->Map(0, nullptr, reinterpret_cast<void**>(&mapped));
    if (FAILED(hr)) return hr;
    for (uint32_t y = 0; y < render_height; ++y) {
        fill_row(Color, y, mapped + footprint.Offset + y * footprint.Footprint.RowPitch,
                 red_shift);
    }
    h.uploads[Color]->Unmap(0, nullptr);
    return S_OK;
}

// Bounded pre-exposure integration input: scale RGB in the pending colour
// upload, leaving alpha, exposure, depth and motion unchanged. Called once per
// fresh harness before submission; no quality comparison is performed.
extern "C" __declspec(dllexport) int32_t __cdecl gpu_set_pre_exposure(
    void* handle, float factor) {
    if (!handle || !std::isfinite(factor) || factor < 0.25f || factor > 4.0f)
        return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    const auto desc = texture_desc(Color);
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint{};
    h.device->GetCopyableFootprints(&desc, 0, 1, 0, &footprint, nullptr, nullptr, nullptr);
    uint8_t* mapped = nullptr;
    HRESULT hr = h.uploads[Color]->Map(0, nullptr, reinterpret_cast<void**>(&mapped));
    if (FAILED(hr)) return hr;
    for (uint32_t y = 0; y < render_height; ++y) {
        uint8_t* row = mapped + footprint.Offset + y * footprint.Footprint.RowPitch;
        for (uint32_t x = 0; x < render_width; ++x) {
            uint16_t pixel[4];
            std::memcpy(pixel, row + x * sizeof(pixel), sizeof(pixel));
            for (int channel = 0; channel < 3; ++channel)
                pixel[channel] = half(unhalf(pixel[channel]) * factor);
            std::memcpy(row + x * sizeof(pixel), pixel, sizeof(pixel));
        }
    }
    h.uploads[Color]->Unmap(0, nullptr);
    return S_OK;
}

// Pending device depth and motion for the bounded profile case. Motion uses
// current-to-previous displacement in the selected texture's UV/pixel units.
// Jitter-containing vectors include the actual projection displacement;
// no comparison of native cancellation quality or provider internals is made.
extern "C" __declspec(dllexport) int32_t __cdecl gpu_set_depth_motion_profile(
    void* handle, uint32_t reversed, uint32_t pixels, uint32_t included_jitter,
    uint32_t frame, float jitter_x, float jitter_y, float previous_x, float previous_y) {
    if (!handle || reversed > 1 || pixels > 1 || included_jitter > 1 || frame > 2
        || !std::isfinite(jitter_x) || !std::isfinite(jitter_y)
        || !std::isfinite(previous_x) || !std::isfinite(previous_y)) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    for (Role role : {Depth, Motion}) {
        const auto desc = h.textures[role]->GetDesc();
        D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint{};
        h.device->GetCopyableFootprints(&desc, 0, 1, 0, &footprint, nullptr, nullptr, nullptr);
        uint8_t* mapped = nullptr;
        HRESULT hr = h.uploads[role]->Map(0, nullptr, reinterpret_cast<void**>(&mapped));
        if (FAILED(hr)) return hr;
        for (uint32_t y = 0; y < desc.Height; ++y) {
            uint8_t* row = mapped + footprint.Offset + y * footprint.Footprint.RowPitch;
            for (uint32_t x = 0; x < desc.Width; ++x) {
                if (role == Depth) {
                    const float depth = reversed ? 0.75f : 0.25f;
                    std::memcpy(row + x * sizeof(depth), &depth, sizeof(depth));
                } else {
                    float dx = frame == 0 ? 0.0f : -1.0f / render_width;
                    float dy = 0.0f;
                    if (included_jitter) {
                        dx += (previous_x - jitter_x) / render_width;
                        dy += (previous_y - jitter_y) / render_height;
                    }
                    if (pixels) {
                        dx *= static_cast<float>(desc.Width);
                        dy *= static_cast<float>(desc.Height);
                    }
                    uint16_t motion[2] = {half(dx), half(dy)};
                    std::memcpy(row + x * sizeof(motion), motion, sizeof(motion));
                }
            }
        }
        h.uploads[role]->Unmap(0, nullptr);
    }
    return S_OK;
}

#include "active_render.h"
#include "supplied_masks.h"
#include "color_signal.h"
#ifdef FSR_REACTIVE_PROBE
#include "reactive_mask.h"
#endif

// Constant linear scene RGB (8,4,2), packed after matching pre-exposure.
// Pending uploads are rewritten before submission; alpha/exposure remain 1.
extern "C" __declspec(dllexport) int32_t __cdecl gpu_set_hdr_frame(
    void* handle, float pre_exposure) {
    if (!handle || !std::isfinite(pre_exposure)
        || pre_exposure < 0.25f || pre_exposure > 4.0f) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    for (Role role : {Color, Depth, Motion, Exposure}) {
        const auto desc = h.textures[role]->GetDesc();
        D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint{};
        h.device->GetCopyableFootprints(&desc, 0, 1, 0, &footprint, nullptr, nullptr, nullptr);
        uint8_t* mapped = nullptr;
        const HRESULT hr = h.uploads[role]->Map(0, nullptr, reinterpret_cast<void**>(&mapped));
        if (FAILED(hr)) return hr;
        for (uint32_t y = 0; y < desc.Height; ++y) {
            auto* row = mapped + footprint.Offset + y * footprint.Footprint.RowPitch;
            for (uint32_t x = 0; x < desc.Width; ++x) {
                if (role == Color) {
                    const uint16_t pixel[4] = {half(8.0f * pre_exposure),
                        half(4.0f * pre_exposure), half(2.0f * pre_exposure), half(1.0f)};
                    std::memcpy(row + x * sizeof(pixel), pixel, sizeof(pixel));
                } else if (role == Motion) {
                    const uint16_t motion[2] = {0, 0};
                    std::memcpy(row + x * sizeof(motion), motion, sizeof(motion));
                } else {
                    const float value = role == Depth ? 0.25f : 1.0f;
                    std::memcpy(row + x * sizeof(value), &value, sizeof(value));
                }
            }
        }
        h.uploads[role]->Unmap(0, nullptr);
    }
    h.hdr = true;
    return S_OK;
}

extern "C" __declspec(dllexport) int32_t __cdecl gpu_finish(void* handle, Metrics* metrics) {
    if (!handle || !metrics) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    HRESULT hr = record_readback(h);
    std::fprintf(stderr, "DX12 readback recording: HRESULT=0x%08lx\n", static_cast<unsigned long>(hr));
    if (FAILED(hr)) return hr;
    if (h.dynamic_render || h.hdr) {
        hr = record_input_readbacks(h);
        if (FAILED(hr)) return hr;
    }
    hr = record_mask_inputs(h);
    if (FAILED(hr)) return hr;
    hr = h.list->Close();
    std::fprintf(stderr, "DX12 list close: HRESULT=0x%08lx\n", static_cast<unsigned long>(hr));
    if (FAILED(hr)) return hr;
    ID3D12CommandList* lists[] = { h.list.Get() };
    h.queue->ExecuteCommandLists(1, lists);
    std::fprintf(stderr, "DX12 submission: ExecuteCommandLists issued (no return value)\n");
    hr = h.queue->Signal(h.fence.Get(), 1);
    std::fprintf(stderr, "DX12 signal: HRESULT=0x%08lx\n", static_cast<unsigned long>(hr));
    if (FAILED(hr)) return hr;
    hr = wait_for_fence(h);
    std::fprintf(stderr, "DX12 fence completion: HRESULT=0x%08lx timeout_ms=60000\n", static_cast<unsigned long>(hr));
    if (FAILED(hr)) return hr;
    hr = read_metrics(h, metrics);
    std::fprintf(stderr, "DX12 mapped readback: HRESULT=0x%08lx\n", static_cast<unsigned long>(hr));
    if (FAILED(hr)) return hr;
    hr = check_mask_inputs(h);
    if (FAILED(hr)) return hr;
    return h.dynamic_render || h.hdr ? check_input_readbacks(h) : S_OK;
}

// Record every readback before submission, then submit all lists in order on
// one queue. Each list has its own allocator and resource set from gpu_create.
extern "C" __declspec(dllexport) int32_t __cdecl gpu_finish_recorded_batch(
    void* const* handles, uint32_t count, Metrics* metrics) {
    if (!handles || !metrics || count == 0 || count > 8 || !handles[0]) return E_INVALIDARG;
    auto& first = *static_cast<Harness*>(handles[0]);
    for (uint32_t index = 0; index < count; ++index) {
        if (!handles[index]) return E_INVALIDARG;
        auto& h = *static_cast<Harness*>(handles[index]);
        if (h.device.Get() != first.device.Get()) return E_INVALIDARG;
        HRESULT hr = record_readback(h);
        if (FAILED(hr)) return hr;
        hr = h.list->Close();
        if (FAILED(hr)) return hr;
    }
    for (uint32_t index = 0; index < count; ++index) {
        auto& h = *static_cast<Harness*>(handles[index]);
        ID3D12CommandList* lists[] = { h.list.Get() };
        first.queue->ExecuteCommandLists(1, lists);
    }
    HRESULT hr = first.queue->Signal(first.fence.Get(), 1);
    if (FAILED(hr)) return hr;
    hr = wait_for_fence(first);
    if (FAILED(hr)) return hr;
    for (uint32_t index = 0; index < count; ++index) {
        hr = read_metrics(*static_cast<Harness*>(handles[index]), &metrics[index]);
        if (FAILED(hr)) return hr;
    }
    return S_OK;
}

extern "C" __declspec(dllexport) void __cdecl gpu_destroy(void* handle) {
    delete static_cast<Harness*>(handle);
}

// Only for a fresh, never-submitted harness. Release its executable list while
// retaining resources/allocator through terminal FSR-context destruction.
extern "C" __declspec(dllexport) void __cdecl gpu_discard_recording(void* handle) {
    if (handle) static_cast<Harness*>(handle)->list.Reset();
}

// Test-only transfer observations. Call after successful submission/fence, while
// the harness and helper are retained. UINT64_MAX means device removal.
extern "C" __declspec(dllexport) uint64_t __cdecl gpu_fence_value(void* handle) {
    return handle ? static_cast<Harness*>(handle)->fence->GetCompletedValue() : UINT64_MAX;
}

// Exact RGBA16F payload, excluding row padding; no conversion or channel loss.
extern "C" __declspec(dllexport) int32_t __cdecl gpu_copy_payload(
    void* handle, uint8_t* output, uint32_t bytes) {
    const uint32_t row_bytes = output_width * 8;
    if (!handle || !output || bytes != row_bytes * output_height) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    const uint64_t completed = h.fence->GetCompletedValue();
    if (completed == UINT64_MAX || completed < 1) return E_FAIL;
    uint8_t* mapped = nullptr;
    D3D12_RANGE range{0, static_cast<SIZE_T>(h.output_bytes)};
    const HRESULT hr = h.readback->Map(0, &range, reinterpret_cast<void**>(&mapped));
    if (FAILED(hr)) return hr;
    for (uint32_t y = 0; y < output_height; ++y) {
        std::memcpy(output + y * row_bytes,
                    mapped + h.output_footprint.Offset
                        + y * h.output_footprint.Footprint.RowPitch,
                    row_bytes);
    }
    const D3D12_RANGE written{0, 0};
    h.readback->Unmap(0, &written);
    return S_OK;
}

extern "C" __declspec(dllexport) int32_t __cdecl semantic_set_frame(
    void* handle, uint32_t frame, float jitter_x, float jitter_y, uint32_t reverse_motion) {
    if (!handle) return E_INVALIDARG;
    return set_semantic_frame(*static_cast<Harness*>(handle), frame, jitter_x, jitter_y, reverse_motion);
}

// Call only after gpu_finish has fenced completion. Copy into caller-owned storage.
extern "C" __declspec(dllexport) int32_t __cdecl semantic_copy_red(
    void* handle, float* output, uint32_t count) {
    if (!handle || !output || count != output_width * output_height) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    uint8_t* mapped = nullptr;
    D3D12_RANGE range{0, static_cast<SIZE_T>(h.output_bytes)};
    HRESULT hr = h.readback->Map(0, &range, reinterpret_cast<void**>(&mapped));
    if (FAILED(hr)) return hr;
    for (uint32_t y = 0; y < output_height; ++y) {
        const auto* row = reinterpret_cast<const uint16_t*>(mapped + h.output_footprint.Offset
            + y * h.output_footprint.Footprint.RowPitch);
        for (uint32_t x = 0; x < output_width; ++x)
            output[y * output_width + x] = unhalf(row[x * 4]);
    }
    D3D12_RANGE written{0, 0};
    h.readback->Unmap(0, &written);
    return S_OK;
}

#include "sharpening.h"
#include "temporal_calibration.h"
#include "temporal_diagnosis.h"

#include "performance.h"

#ifdef FSR_FORMAT_PROBE
#include "typed_formats.h"
#endif

#ifdef FSR_OUTPUT_PROBE
#include "variable_output.h"
#ifdef FSR_MOTION_PROBE
#include "motion_scale.h"
#include "public_motion.h"
#endif
#endif

#ifdef FSR_GPU_TIMING
#include "gpu_timing.h"
#endif
