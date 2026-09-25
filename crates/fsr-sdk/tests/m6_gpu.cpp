// SPDX-License-Identifier: MPL-2.0

// Opt-in GPU laboratory for the private M6a Rust dispatch path. No AMD DLL
// calls occur here; the included inline helper is used only for parity checks.
#define NOMINMAX
#include <d3d12.h>
#include <wrl/client.h>
#include <algorithm>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <memory>
#include <vector>
#include "ffx_api_dx12.h"

using Microsoft::WRL::ComPtr;

namespace {
constexpr uint32_t render_width = 320;
constexpr uint32_t render_height = 180;
constexpr uint32_t output_width = 640;
constexpr uint32_t output_height = 360;
constexpr uint16_t sentinel_half = 0x7bff; // 65504, unlike expected output
enum Role { Color, Depth, Motion, Exposure, Output, Count };

struct Harness {
    ComPtr<ID3D12Device> device;
    ComPtr<ID3D12CommandQueue> queue;
    ComPtr<ID3D12CommandAllocator> allocator;
    ComPtr<ID3D12GraphicsCommandList> list;
    ComPtr<ID3D12Resource> textures[Count];
    ComPtr<ID3D12Resource> uploads[Count];
    ComPtr<ID3D12Resource> readback;
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

D3D12_RESOURCE_DESC texture_desc(Role role) {
    D3D12_RESOURCE_DESC desc{};
    desc.Dimension = D3D12_RESOURCE_DIMENSION_TEXTURE2D;
    desc.Width = role == Output ? output_width : role == Exposure ? 1 : render_width;
    desc.Height = role == Output ? output_height : role == Exposure ? 1 : render_height;
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

uint16_t half(float value) {
    uint32_t bits;
    std::memcpy(&bits, &value, sizeof(bits));
    const uint32_t sign = (bits >> 16) & 0x8000;
    const int exponent = static_cast<int>((bits >> 23) & 0xff) - 127 + 15;
    const uint32_t fraction = bits & 0x7fffff;
    if (exponent <= 0) return static_cast<uint16_t>(sign);
    if (exponent >= 31) return static_cast<uint16_t>(sign | 0x7c00);
    return static_cast<uint16_t>(sign | (exponent << 10) | (fraction >> 13));
}

float unhalf(uint16_t value) {
    const uint32_t sign = static_cast<uint32_t>(value & 0x8000) << 16;
    const uint32_t exponent = (value >> 10) & 31;
    const uint32_t fraction = value & 1023;
    uint32_t bits;
    if (exponent == 0) {
        if (fraction == 0) bits = sign;
        else {
            uint32_t mantissa = fraction;
            int shift = 0;
            while ((mantissa & 1024) == 0) { mantissa <<= 1; ++shift; }
            bits = sign | (static_cast<uint32_t>(127 - 14 - shift) << 23)
                | ((mantissa & 1023) << 13);
        }
    } else if (exponent == 31) {
        bits = sign | 0x7f800000 | (fraction << 13);
    } else {
        bits = sign | ((exponent + 112) << 23) | (fraction << 13);
    }
    float result;
    std::memcpy(&result, &bits, sizeof(result));
    return result;
}

void fill_row(Role role, uint32_t y, uint8_t* row, float red_shift = 0.0f) {
    const auto desc = texture_desc(role);
    for (uint32_t x = 0; x < desc.Width; ++x) {
        if (role == Color) {
            // CPU-generated image sampled at the same known subpixel jitter
            // passed in the Rust descriptor. It needs no camera or renderer.
            const float red = 0.1f + red_shift + 0.75f * (x + 0.75f) / render_width;
            const float green = 0.1f + 0.75f * (y + 0.25f) / render_height;
            const float blue = ((x / 16 + y / 16) & 1) ? 0.4f : 0.2f;
            uint16_t pixel[4] = { half(red), half(green), half(blue), half(1.0f) };
            std::memcpy(row + x * sizeof(pixel), pixel, sizeof(pixel));
        } else if (role == Output) {
            uint16_t pixel[4] = { sentinel_half, sentinel_half, sentinel_half, sentinel_half };
            std::memcpy(row + x * sizeof(pixel), pixel, sizeof(pixel));
        } else if (role == Motion) {
            uint16_t pixel[2] = { 0, 0 };
            std::memcpy(row + x * sizeof(pixel), pixel, sizeof(pixel));
        } else {
            const float value = role == Exposure ? 1.0f : 0.5f;
            std::memcpy(row + x * sizeof(value), &value, sizeof(value));
        }
    }
}

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
    const auto desc = texture_desc(role);
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
        fill_row(role, y, mapped + footprint.Offset + y * footprint.Footprint.RowPitch);
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
        if (FAILED(info->GetMessage(index, nullptr, &bytes))) continue;
        std::vector<uint8_t> storage(bytes);
        auto* message = reinterpret_cast<D3D12_MESSAGE*>(storage.data());
        if (SUCCEEDED(info->GetMessage(index, message, &bytes))
            && (message->Severity == D3D12_MESSAGE_SEVERITY_ERROR
                || message->Severity == D3D12_MESSAGE_SEVERITY_CORRUPTION)) ++errors;
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

extern "C" __declspec(dllexport) int32_t __cdecl m6_enable_debug() {
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

extern "C" __declspec(dllexport) int32_t __cdecl m6_create(
    ID3D12Device* device, void** out_harness,
    ID3D12GraphicsCommandList** out_list, ID3D12Resource** out_resources) {
    if (!device || !out_harness || !out_list || !out_resources) return E_INVALIDARG;
    *out_harness = nullptr;
    *out_list = nullptr;
    for (int i = 0; i < Count; ++i) out_resources[i] = nullptr;
    auto h = std::make_unique<Harness>();
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

extern "C" __declspec(dllexport) int32_t __cdecl m6_compare_resource(
    ID3D12Resource* resource, const FfxApiResource* actual, uint32_t state) {
    if (!resource || !actual) return 0;
    const FfxApiResource expected = ffxApiGetResourceDX12(resource, state);
    const auto& a = actual->description;
    const auto& e = expected.description;
    return actual->resource == expected.resource && actual->state == expected.state
        && a.type == e.type && a.format == e.format && a.width == e.width
        && a.height == e.height && a.depth == e.depth && a.mipCount == e.mipCount
        && a.flags == e.flags && a.usage == e.usage;
}

// Change only this harness's not-yet-submitted upload. Each batch output then
// has a distinct expected red level, exposing stale or aliased descriptors.
extern "C" __declspec(dllexport) int32_t __cdecl m6_set_color_shift(
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

extern "C" __declspec(dllexport) int32_t __cdecl m6_finish(void* handle, Metrics* metrics) {
    if (!handle || !metrics) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    HRESULT hr = record_readback(h);
    if (FAILED(hr)) return hr;
    hr = h.list->Close();
    if (FAILED(hr)) return hr;
    ID3D12CommandList* lists[] = { h.list.Get() };
    h.queue->ExecuteCommandLists(1, lists);
    hr = h.queue->Signal(h.fence.Get(), 1);
    if (FAILED(hr)) return hr;
    hr = wait_for_fence(h);
    if (FAILED(hr)) return hr;
    return read_metrics(h, metrics);
}

// Record every readback before submission, then submit all lists in order on
// one queue. Each list has its own allocator and resource set from m6_create.
extern "C" __declspec(dllexport) int32_t __cdecl m6_finish_recorded_batch(
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

extern "C" __declspec(dllexport) void __cdecl m6_destroy(void* handle) {
    delete static_cast<Harness*>(handle);
}
