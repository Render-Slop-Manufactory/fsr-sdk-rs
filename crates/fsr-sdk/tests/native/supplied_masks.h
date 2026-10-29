// SPDX-License-Identifier: MPL-2.0

// Renderer-owned R8_UNORM inputs for bounded mask dispatch integration.
// Patterns distinguish the two inputs; readback checks preserve their exact
// uploaded bytes. No image-quality or provider-consumption inference is made.
extern "C" __declspec(dllexport) int32_t __cdecl gpu_create_masks(
    void* handle, uint32_t width, uint32_t height, uint32_t frame,
    float jitter_x, float jitter_y, ID3D12Resource** output) {
    if (!handle || !output || width == 0 || height == 0 || width > render_width
        || height > render_height || frame > 2 || !std::isfinite(jitter_x)
        || !std::isfinite(jitter_y)) return E_INVALIDARG;
    output[0] = output[1] = nullptr;
    auto& h = *static_cast<Harness*>(handle);
    if (h.masks[0] || h.masks[1]) return E_INVALIDARG;
    for (uint32_t index = 0; index < 2; ++index) {
        auto desc = texture_desc(Depth);
        desc.Format = DXGI_FORMAT_R8_UNORM;
        const auto default_heap = heap_properties(D3D12_HEAP_TYPE_DEFAULT);
        HRESULT hr = h.device->CreateCommittedResource(&default_heap, D3D12_HEAP_FLAG_NONE,
            &desc, D3D12_RESOURCE_STATE_COPY_DEST, nullptr, IID_PPV_ARGS(&h.masks[index]));
        if (FAILED(hr)) return hr;
        h.device->GetCopyableFootprints(&desc, 0, 1, 0, &h.mask_footprints[index],
            nullptr, nullptr, &h.mask_bytes[index]);
        const auto buffer = buffer_desc(h.mask_bytes[index]);
        const auto upload_heap = heap_properties(D3D12_HEAP_TYPE_UPLOAD);
        hr = h.device->CreateCommittedResource(&upload_heap, D3D12_HEAP_FLAG_NONE,
            &buffer, D3D12_RESOURCE_STATE_GENERIC_READ, nullptr, IID_PPV_ARGS(&h.mask_uploads[index]));
        if (FAILED(hr)) return hr;
        uint8_t* mapped = nullptr;
        hr = h.mask_uploads[index]->Map(0, nullptr, reinterpret_cast<void**>(&mapped));
        if (FAILED(hr)) return hr;
        const auto& footprint = h.mask_footprints[index];
        std::memset(mapped, 0, static_cast<size_t>(h.mask_bytes[index]));
        for (uint32_t y = 0; y < render_height; ++y) {
            auto* row = mapped + footprint.Offset + y * footprint.Footprint.RowPitch;
            for (uint32_t x = 0; x < render_width; ++x) {
                const float u = (static_cast<float>(x) + 0.5f - jitter_x) / static_cast<float>(width)
                    - static_cast<float>(frame) * 0.003f;
                const float v = (static_cast<float>(y) + 0.5f - jitter_y) / static_cast<float>(height);
                const bool marked = index == 0 ? u > 0.25f && u < 0.5f : v > 0.5f && v < 0.75f;
                row[x] = static_cast<uint8_t>(x >= width || y >= height ? 227 : marked ? (index == 0 ? 128 : 255) : 0);
            }
        }
        h.mask_uploads[index]->Unmap(0, nullptr);
        D3D12_TEXTURE_COPY_LOCATION source{}, target{};
        source.pResource = h.mask_uploads[index].Get();
        source.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
        source.PlacedFootprint = footprint;
        target.pResource = h.masks[index].Get();
        target.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
        h.list->CopyTextureRegion(&target, 0, 0, 0, &source, nullptr);
        transition(h.list.Get(), h.masks[index].Get(), D3D12_RESOURCE_STATE_COPY_DEST,
            D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE);
    }
    for (uint32_t index = 0; index < 2; ++index) {
        output[index] = h.masks[index].Get();
        output[index]->AddRef();
    }
    return S_OK;
}

HRESULT record_mask_inputs(Harness& h) {
    for (uint32_t index = 0; index < 2; ++index) {
        if (!h.masks[index]) continue;
        const auto heap = heap_properties(D3D12_HEAP_TYPE_READBACK);
        const auto buffer = buffer_desc(h.mask_bytes[index]);
        HRESULT hr = h.device->CreateCommittedResource(&heap, D3D12_HEAP_FLAG_NONE,
            &buffer, D3D12_RESOURCE_STATE_COPY_DEST, nullptr, IID_PPV_ARGS(&h.mask_readbacks[index]));
        if (FAILED(hr)) return hr;
        transition(h.list.Get(), h.masks[index].Get(),
            D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE, D3D12_RESOURCE_STATE_COPY_SOURCE);
        D3D12_TEXTURE_COPY_LOCATION source{}, target{};
        source.pResource = h.masks[index].Get();
        source.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
        target.pResource = h.mask_readbacks[index].Get();
        target.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
        target.PlacedFootprint = h.mask_footprints[index];
        h.list->CopyTextureRegion(&target, 0, 0, 0, &source, nullptr);
        transition(h.list.Get(), h.masks[index].Get(),
            D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE);
    }
    return S_OK;
}

HRESULT check_mask_inputs(Harness& h) {
    for (uint32_t index = 0; index < 2; ++index) {
        if (!h.masks[index]) continue;
        uint8_t* actual = nullptr;
        uint8_t* expected = nullptr;
        const D3D12_RANGE range{0, static_cast<SIZE_T>(h.mask_bytes[index])};
        HRESULT hr = h.mask_readbacks[index]->Map(0, &range, reinterpret_cast<void**>(&actual));
        if (FAILED(hr)) return hr;
        hr = h.mask_uploads[index]->Map(0, &range, reinterpret_cast<void**>(&expected));
        if (FAILED(hr)) {
            h.mask_readbacks[index]->Unmap(0, nullptr);
            return hr;
        }
        bool equal = true;
        const auto& footprint = h.mask_footprints[index];
        for (uint32_t y = 0; y < render_height; ++y) {
            const auto offset = footprint.Offset + y * footprint.Footprint.RowPitch;
            equal &= std::memcmp(actual + offset, expected + offset, render_width) == 0;
        }
        const D3D12_RANGE written{0, 0};
        h.mask_uploads[index]->Unmap(0, &written);
        h.mask_readbacks[index]->Unmap(0, &written);
        if (!equal) return E_FAIL;
        std::fprintf(stderr, "Supplied mask input readback: PASSED index=%u\n", index);
    }
    return S_OK;
}
