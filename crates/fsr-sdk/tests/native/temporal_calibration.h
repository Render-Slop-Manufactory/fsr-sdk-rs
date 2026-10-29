// SPDX-License-Identifier: MPL-2.0

// Experiment-only scene and input readbacks. No AMD calls. The paired Python
// oracle independently projects geometry, quantizes inputs and decodes outputs.
#include <psapi.h>
#pragma comment(lib, "psapi.lib")

extern "C" __declspec(dllexport) int32_t __cdecl temporal_calibration_set_frame(
    void* handle, uint32_t scene, uint32_t frame, float jx, float jy,
    uint32_t reversed, uint32_t cut) {
    if (!handle || scene > 1 || frame > 63 || reversed > 1 || cut > 1
        || !std::isfinite(jx) || !std::isfinite(jy)) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    const bool after_cut = cut && frame >= 37;
    const double left = 100.0 + 0.375 * frame - (after_cut ? 96.0 : 0.0);
    const double top = 50.0 + 0.25 * frame + (after_cut ? 48.0 : 0.0);
    for (Role role : {Color, Depth, Motion}) {
        const auto desc = texture_desc(role);
        D3D12_PLACED_SUBRESOURCE_FOOTPRINT fp{};
        h.device->GetCopyableFootprints(&desc, 0, 1, 0, &fp, nullptr, nullptr, nullptr);
        uint8_t* mapped = nullptr;
        HRESULT hr = h.uploads[role]->Map(0, nullptr, reinterpret_cast<void**>(&mapped));
        if (FAILED(hr)) return hr;
        for (uint32_t y = 0; y < render_height; ++y) {
            auto* row = mapped + fp.Offset + y * fp.Footprint.RowPitch;
            for (uint32_t x = 0; x < render_width; ++x) {
                const double sx = x + 0.5 - static_cast<double>(jx);
                const double sy = y + 0.5 - static_cast<double>(jy);
                const bool fg = scene == 1 && sx >= left && sx < left + 64.0
                    && sy >= top && sy < top + 44.0;
                if (role == Color) {
                    const double red = scene == 0 ? 0.125 + sx / 1024.0 : fg ? 0.875 : 0.125;
                    const double green = scene == 0 ? 0.25 + sy / 1024.0 : fg ? 0.125 : 0.25;
                    const double blue = scene == 0 ? 0.375 + (sx + sy) / 2048.0 : fg ? 0.0625 : 0.375;
                    const uint16_t pixel[4] = {half(static_cast<float>(red)),
                        half(static_cast<float>(green)), half(static_cast<float>(blue)), half(1.0f)};
                    std::memcpy(row + x * 8, pixel, sizeof(pixel));
                } else if (role == Depth) {
                    const double z = fg ? 2.0 : 8.0;
                    const float d = static_cast<float>(20.0 / 19.5 - 10.0 / (19.5 * z));
                    std::memcpy(row + x * 4, &d, sizeof(d));
                } else {
                    double mx = fg && frame ? -0.375 : 0.0;
                    double my = fg && frame ? -0.25 : 0.0;
                    if (cut && frame == 37) { mx += fg ? 96.0 : 24.0; my -= fg ? 48.0 : 12.0; }
                    if (reversed && fg) { mx = -mx; my = -my; }
                    const uint16_t pixel[2] = {half(static_cast<float>(mx / 320.0)),
                        half(static_cast<float>(my / 180.0))};
                    std::memcpy(row + x * 4, pixel, sizeof(pixel));
                }
            }
        }
        h.uploads[role]->Unmap(0, nullptr);
    }
    return S_OK;
}

// Copies occur before provider dispatch and restore each declared input state.
extern "C" __declspec(dllexport) int32_t __cdecl temporal_calibration_record_inputs(void* handle) {
    if (!handle) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    for (int index = 0; index < 4; ++index) {
        if (h.calibration_inputs[index]) return E_INVALIDARG;
        const auto desc = texture_desc(static_cast<Role>(index));
        h.device->GetCopyableFootprints(&desc, 0, 1, 0, &h.calibration_footprints[index],
            nullptr, nullptr, &h.calibration_bytes[index]);
        const auto buffer = buffer_desc(h.calibration_bytes[index]);
        const auto heap = heap_properties(D3D12_HEAP_TYPE_READBACK);
        HRESULT hr = h.device->CreateCommittedResource(&heap, D3D12_HEAP_FLAG_NONE,
            &buffer, D3D12_RESOURCE_STATE_COPY_DEST, nullptr, IID_PPV_ARGS(&h.calibration_inputs[index]));
        if (FAILED(hr)) return hr;
        transition(h.list.Get(), h.textures[index].Get(), D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
            D3D12_RESOURCE_STATE_COPY_SOURCE);
        D3D12_TEXTURE_COPY_LOCATION from{}, to{};
        from.pResource = h.textures[index].Get();
        from.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
        to.pResource = h.calibration_inputs[index].Get();
        to.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
        to.PlacedFootprint = h.calibration_footprints[index];
        h.list->CopyTextureRegion(&to, 0, 0, 0, &from, nullptr);
        transition(h.list.Get(), h.textures[index].Get(), D3D12_RESOURCE_STATE_COPY_SOURCE,
            D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE);
    }
    return S_OK;
}

extern "C" __declspec(dllexport) int32_t __cdecl temporal_calibration_copy_inputs(
    void* handle, uint8_t* output, uint32_t bytes) {
    if (!handle || !output || bytes != 320 * 180 * 16 + 4) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    if (h.fence->GetCompletedValue() != 1) return E_FAIL;
    size_t offset = 0;
    for (int index = 0; index < 4; ++index) {
        const auto& fp = h.calibration_footprints[index];
        uint8_t* mapped = nullptr;
        D3D12_RANGE range{0, static_cast<SIZE_T>(h.calibration_bytes[index])};
        HRESULT hr = h.calibration_inputs[index]->Map(0, &range, reinterpret_cast<void**>(&mapped));
        if (FAILED(hr)) return hr;
        const uint32_t row_bytes = index == Color ? 320 * 8 : index == Exposure ? 4 : 320 * 4;
        for (uint32_t y = 0; y < fp.Footprint.Height; ++y) {
            std::memcpy(output + offset, mapped + fp.Offset + y * fp.Footprint.RowPitch, row_bytes);
            offset += row_bytes;
        }
        const D3D12_RANGE written{0, 0};
        h.calibration_inputs[index]->Unmap(0, &written);
    }
    return S_OK;
}

extern "C" __declspec(dllexport) void __cdecl temporal_calibration_memory(ID3D12Device* device) {
    for (const wchar_t* name : {L"amd_fidelityfx_loader_dx12.dll", L"amd_fidelityfx_upscaler_dx12.dll"}) {
        HMODULE module = GetModuleHandleW(name);
        wchar_t path[32768]{};
        char utf8[32768]{};
        char basename[128]{};
        if (module && GetModuleFileNameW(module, path, 32768)
            && WideCharToMultiByte(CP_UTF8, 0, path, -1, utf8, sizeof(utf8), nullptr, nullptr)
            && WideCharToMultiByte(CP_UTF8, 0, name, -1, basename, sizeof(basename), nullptr, nullptr))
            std::fprintf(stderr, "Temporal calibration module: %s path=%s\n", basename, utf8);
    }
    PROCESS_MEMORY_COUNTERS_EX memory{};
    memory.cb = sizeof(memory);
    if (GetProcessMemoryInfo(GetCurrentProcess(), reinterpret_cast<PROCESS_MEMORY_COUNTERS*>(&memory), sizeof(memory)))
        std::fprintf(stderr, "Temporal calibration memory: working_set=%llu peak_working_set=%llu private_bytes=%llu\n",
            static_cast<unsigned long long>(memory.WorkingSetSize),
            static_cast<unsigned long long>(memory.PeakWorkingSetSize),
            static_cast<unsigned long long>(memory.PrivateUsage));
    ComPtr<IDXGIFactory4> factory;
    ComPtr<IDXGIAdapter3> adapter;
    if (SUCCEEDED(CreateDXGIFactory1(IID_PPV_ARGS(&factory)))
        && SUCCEEDED(factory->EnumAdapterByLuid(device->GetAdapterLuid(), IID_PPV_ARGS(&adapter)))) {
        DXGI_QUERY_VIDEO_MEMORY_INFO local{};
        if (SUCCEEDED(adapter->QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, &local)))
            std::fprintf(stderr, "Temporal calibration GPU memory: usage=%llu budget=%llu\n",
                static_cast<unsigned long long>(local.CurrentUsage), static_cast<unsigned long long>(local.Budget));
    }
}
