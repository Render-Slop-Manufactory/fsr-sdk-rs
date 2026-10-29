// SPDX-License-Identifier: MPL-2.0

// Bounded active-image generator and shared read-only input guards for gpu.cpp.
// Allocation dimensions stay fixed. This checks integration, not image quality
// or whether the provider sampled any particular inactive input texel.
extern "C" __declspec(dllexport) int32_t __cdecl gpu_set_active_render(
    void* handle, uint32_t width, uint32_t height, uint32_t pixels,
    uint32_t frame, float jitter_x, float jitter_y) {
    if (!handle || width == 0 || height == 0 || width > render_width
        || height > render_height || pixels > 1 || frame > 2
        || !std::isfinite(jitter_x) || !std::isfinite(jitter_y)) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    h.dynamic_render = true;
    for (Role role : {Color, Depth, Motion}) {
        const auto desc = h.textures[role]->GetDesc();
        D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint{};
        h.device->GetCopyableFootprints(&desc, 0, 1, 0, &footprint, nullptr, nullptr, nullptr);
        uint8_t* mapped = nullptr;
        HRESULT hr = h.uploads[role]->Map(0, nullptr, reinterpret_cast<void**>(&mapped));
        if (FAILED(hr)) return hr;
        const uint32_t active_width = role == Motion && h.display_motion ? output_width : width;
        const uint32_t active_height = role == Motion && h.display_motion ? output_height : height;
        for (uint32_t y = 0; y < desc.Height; ++y) {
            auto* row = mapped + footprint.Offset + y * footprint.Footprint.RowPitch;
            for (uint32_t x = 0; x < desc.Width; ++x) {
                const bool active = x < active_width && y < active_height;
                if (role == Color) {
                    const float u = (static_cast<float>(x) + 0.5f - jitter_x) / static_cast<float>(width)
                        - static_cast<float>(frame) * 0.003f;
                    const float v = (static_cast<float>(y) + 0.5f - jitter_y) / static_cast<float>(height);
                    const float red = 0.1f + 0.4f * u + 0.05f * std::sin(20.0f * u);
                    const uint16_t color[4] = {
                        active ? half(red) : sentinel_half,
                        active ? half(0.2f + 0.3f * v) : sentinel_half,
                        active ? half(0.25f) : sentinel_half, half(1.0f)};
                    std::memcpy(row + x * sizeof(color), color, sizeof(color));
                } else if (role == Depth) {
                    const float depth = active ? 0.25f : 1.0f;
                    std::memcpy(row + x * sizeof(depth), &depth, sizeof(depth));
                } else {
                    const float dx = frame == 0 ? 0.0f : -0.003f;
                    const uint16_t motion[2] = {
                        active ? half(pixels ? dx * static_cast<float>(active_width) : dx) : sentinel_half,
                        active ? half(0.0f) : sentinel_half};
                    std::memcpy(row + x * sizeof(motion), motion, sizeof(motion));
                }
            }
        }
        h.uploads[role]->Unmap(0, nullptr);
    }
    return S_OK;
}

HRESULT record_input_readbacks(Harness& h) {
    for (Role role : {Color, Depth, Motion, Exposure}) {
        if (role == Exposure && !h.hdr) continue;
        const auto desc = h.textures[role]->GetDesc();
        auto& footprint = h.input_footprints[role];
        auto& bytes = h.input_bytes[role];
        h.device->GetCopyableFootprints(&desc, 0, 1, 0, &footprint, nullptr, nullptr, &bytes);
        const auto heap = heap_properties(D3D12_HEAP_TYPE_READBACK);
        const auto buffer = buffer_desc(bytes);
        HRESULT hr = h.device->CreateCommittedResource(&heap, D3D12_HEAP_FLAG_NONE, &buffer,
            D3D12_RESOURCE_STATE_COPY_DEST, nullptr, IID_PPV_ARGS(&h.input_readbacks[role]));
        if (FAILED(hr)) return hr;
        transition(h.list.Get(), h.textures[role].Get(),
            D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE, D3D12_RESOURCE_STATE_COPY_SOURCE);
        D3D12_TEXTURE_COPY_LOCATION source{}, target{};
        source.pResource = h.textures[role].Get();
        source.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
        target.pResource = h.input_readbacks[role].Get();
        target.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
        target.PlacedFootprint = footprint;
        h.list->CopyTextureRegion(&target, 0, 0, 0, &source, nullptr);
        transition(h.list.Get(), h.textures[role].Get(),
            D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE);
    }
    return S_OK;
}

HRESULT check_input_readbacks(Harness& h) {
    for (Role role : {Color, Depth, Motion, Exposure}) {
        if (role == Exposure && !h.hdr) continue;
        uint8_t* actual = nullptr;
        uint8_t* expected = nullptr;
        const D3D12_RANGE range{0, static_cast<SIZE_T>(h.input_bytes[role])};
        HRESULT hr = h.input_readbacks[role]->Map(0, &range, reinterpret_cast<void**>(&actual));
        if (FAILED(hr)) return hr;
        hr = h.uploads[role]->Map(0, &range, reinterpret_cast<void**>(&expected));
        if (FAILED(hr)) {
            h.input_readbacks[role]->Unmap(0, nullptr);
            return hr;
        }
        const auto desc = h.textures[role]->GetDesc();
        const auto& footprint = h.input_footprints[role];
        const size_t row_bytes = static_cast<size_t>(desc.Width) * (role == Color ? 8 : 4);
        bool equal = true;
        for (uint32_t y = 0; y < desc.Height; ++y) {
            const auto offset = footprint.Offset + y * footprint.Footprint.RowPitch;
            equal &= std::memcmp(actual + offset, expected + offset, row_bytes) == 0;
        }
        const D3D12_RANGE written{0, 0};
        h.uploads[role]->Unmap(0, &written);
        h.input_readbacks[role]->Unmap(0, &written);
        if (!equal) return E_FAIL;
    }
    std::fprintf(stderr, "%s input readback/guards: PASSED\n", h.hdr ? "HDR" : "Active render");
    return S_OK;
}
