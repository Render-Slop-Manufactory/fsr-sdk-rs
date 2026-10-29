// SPDX-License-Identifier: MPL-2.0

// Opt-in D030 equipment only. Generation itself is called from the Rust probe.
struct ReactiveMetrics {
    uint32_t width, height, active_pixels, changed_pixels, zero_pixels;
    uint32_t outside_changed, minimum, maximum;
};
static_assert(sizeof(ReactiveMetrics) == 32 && alignof(ReactiveMetrics) == 4);

extern "C" __declspec(dllexport) int32_t __cdecl gpu_probe_prepare(
    void* handle, uint32_t width, uint32_t height, ID3D12Resource** opaque,
    ID3D12Resource** reactive) {
    if (!handle || !opaque || !reactive || width == 0 || height == 0
        || width > render_width || height > render_height) return E_INVALIDARG;
    *opaque = *reactive = nullptr;
    auto& h = *static_cast<Harness*>(handle);
    if (h.probe_opaque || h.probe_reactive) return E_INVALIDARG;
    h.probe_width = width; h.probe_height = height;
    const auto device_heap = heap_properties(D3D12_HEAP_TYPE_DEFAULT);
    const auto upload_heap = heap_properties(D3D12_HEAP_TYPE_UPLOAD);
    const auto readback_heap = heap_properties(D3D12_HEAP_TYPE_READBACK);
    const auto color = texture_desc(Color);
    HRESULT hr = h.device->CreateCommittedResource(&device_heap, D3D12_HEAP_FLAG_NONE,
        &color, D3D12_RESOURCE_STATE_COPY_DEST, nullptr, IID_PPV_ARGS(&h.probe_opaque));
    if (FAILED(hr)) return hr;
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT color_footprint{};
    UINT64 color_bytes = 0;
    h.device->GetCopyableFootprints(&color, 0, 1, 0, &color_footprint, nullptr, nullptr, &color_bytes);
    const auto color_buffer = buffer_desc(color_bytes);
    hr = h.device->CreateCommittedResource(&upload_heap, D3D12_HEAP_FLAG_NONE,
        &color_buffer, D3D12_RESOURCE_STATE_GENERIC_READ, nullptr, IID_PPV_ARGS(&h.probe_opaque_upload));
    if (FAILED(hr)) return hr;
    uint8_t* source = nullptr; uint8_t* destination = nullptr;
    hr = h.uploads[Color]->Map(0, nullptr, reinterpret_cast<void**>(&source));
    if (FAILED(hr)) return hr;
    hr = h.probe_opaque_upload->Map(0, nullptr, reinterpret_cast<void**>(&destination));
    if (FAILED(hr)) { h.uploads[Color]->Unmap(0, nullptr); return hr; }
    std::memcpy(destination, source, static_cast<size_t>(color_bytes));
    for (uint32_t y = height / 4; y < height * 3 / 4; ++y) {
        for (uint32_t x = width / 4; x < width * 3 / 4; ++x) {
            uint16_t pixel[4];
            auto* address = destination + color_footprint.Offset
                + y * color_footprint.Footprint.RowPitch + x * sizeof(pixel);
            std::memcpy(pixel, address, sizeof(pixel));
            for (int channel = 0; channel < 3; ++channel) pixel[channel] = half(unhalf(pixel[channel]) * 0.5f);
            std::memcpy(address, pixel, sizeof(pixel));
        }
    }
    h.probe_opaque_upload->Unmap(0, nullptr); h.uploads[Color]->Unmap(0, nullptr);
    D3D12_TEXTURE_COPY_LOCATION from{}, to{};
    from.pResource = h.probe_opaque_upload.Get(); from.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
    from.PlacedFootprint = color_footprint;
    to.pResource = h.probe_opaque.Get(); to.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
    h.list->CopyTextureRegion(&to, 0, 0, 0, &from, nullptr);
    transition(h.list.Get(), h.probe_opaque.Get(), D3D12_RESOURCE_STATE_COPY_DEST,
        D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE);

    auto mask = texture_desc(Depth); mask.Format = DXGI_FORMAT_R8_UNORM;
    mask.Flags = D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
    hr = h.device->CreateCommittedResource(&device_heap, D3D12_HEAP_FLAG_NONE,
        &mask, D3D12_RESOURCE_STATE_COPY_DEST, nullptr, IID_PPV_ARGS(&h.probe_reactive));
    if (FAILED(hr)) return hr;
    h.device->GetCopyableFootprints(&mask, 0, 1, 0, &h.probe_footprint, nullptr, nullptr, &h.probe_bytes);
    const auto mask_buffer = buffer_desc(h.probe_bytes);
    hr = h.device->CreateCommittedResource(&upload_heap, D3D12_HEAP_FLAG_NONE,
        &mask_buffer, D3D12_RESOURCE_STATE_GENERIC_READ, nullptr, IID_PPV_ARGS(&h.probe_seed));
    if (FAILED(hr)) return hr;
    hr = h.device->CreateCommittedResource(&readback_heap, D3D12_HEAP_FLAG_NONE,
        &mask_buffer, D3D12_RESOURCE_STATE_COPY_DEST, nullptr, IID_PPV_ARGS(&h.probe_readback));
    if (FAILED(hr)) return hr;
    hr = h.probe_seed->Map(0, nullptr, reinterpret_cast<void**>(&destination));
    if (FAILED(hr)) return hr;
    std::memset(destination, 227, static_cast<size_t>(h.probe_bytes));
    h.probe_seed->Unmap(0, nullptr);
    from.pResource = h.probe_seed.Get(); from.PlacedFootprint = h.probe_footprint;
    to.pResource = h.probe_reactive.Get();
    h.list->CopyTextureRegion(&to, 0, 0, 0, &from, nullptr);
    transition(h.list.Get(), h.probe_reactive.Get(), D3D12_RESOURCE_STATE_COPY_DEST,
        D3D12_RESOURCE_STATE_UNORDERED_ACCESS);
    *opaque = h.probe_opaque.Get(); (*opaque)->AddRef();
    *reactive = h.probe_reactive.Get(); (*reactive)->AddRef();
    return S_OK;
}

extern "C" __declspec(dllexport) void __cdecl gpu_probe_after_generation(void* handle) {
    auto& h = *static_cast<Harness*>(handle);
    D3D12_RESOURCE_BARRIER barrier{};
    barrier.Type = D3D12_RESOURCE_BARRIER_TYPE_UAV;
    barrier.UAV.pResource = h.probe_reactive.Get(); h.list->ResourceBarrier(1, &barrier);
    // This explicitly exercises the proposed restored-UAV boundary; debug-layer
    // failures are evidence against it, never a silently repaired state.
    transition(h.list.Get(), h.probe_reactive.Get(), D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
        D3D12_RESOURCE_STATE_COPY_SOURCE);
    D3D12_TEXTURE_COPY_LOCATION from{}, to{};
    from.pResource = h.probe_reactive.Get(); from.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
    to.pResource = h.probe_readback.Get(); to.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
    to.PlacedFootprint = h.probe_footprint;
    h.list->CopyTextureRegion(&to, 0, 0, 0, &from, nullptr);
    transition(h.list.Get(), h.probe_reactive.Get(), D3D12_RESOURCE_STATE_COPY_SOURCE,
        D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE);
}

extern "C" __declspec(dllexport) int32_t __cdecl gpu_probe_metrics(void* handle, ReactiveMetrics* out) {
    if (!handle || !out) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    const auto completed = h.fence->GetCompletedValue();
    if (completed == UINT64_MAX || completed < 1) return E_FAIL;
    uint8_t* mapped = nullptr;
    const D3D12_RANGE range{0, static_cast<SIZE_T>(h.probe_bytes)};
    HRESULT hr = h.probe_readback->Map(0, &range, reinterpret_cast<void**>(&mapped));
    if (FAILED(hr)) return hr;
    *out = {h.probe_width, h.probe_height, h.probe_width * h.probe_height, 0, 0, 0, 255, 0};
    for (uint32_t y = 0; y < render_height; ++y) {
        const auto* row = mapped + h.probe_footprint.Offset + y * h.probe_footprint.Footprint.RowPitch;
        for (uint32_t x = 0; x < render_width; ++x) {
            if (x < h.probe_width && y < h.probe_height) {
                out->changed_pixels += row[x] != 227 ? 1u : 0u;
                out->zero_pixels += row[x] == 0 ? 1u : 0u;
                out->minimum = std::min(out->minimum, static_cast<uint32_t>(row[x]));
                out->maximum = std::max(out->maximum, static_cast<uint32_t>(row[x]));
            } else { out->outside_changed += row[x] != 227 ? 1u : 0u; }
        }
    }
    const D3D12_RANGE written{0, 0}; h.probe_readback->Unmap(0, &written);
    return S_OK;
}
