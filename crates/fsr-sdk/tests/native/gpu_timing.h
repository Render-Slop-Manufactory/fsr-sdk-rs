// SPDX-License-Identifier: MPL-2.0

// Included only by the dedicated timing-helper build. No AMD SDK calls here.
extern "C" __declspec(dllexport) int32_t __cdecl gpu_timing_configure(uint32_t width, uint32_t height) {
    if (!width || !height || width > output_width || height > output_height) return E_INVALIDARG;
    render_width = width; render_height = height;
    return S_OK;
}

HRESULT timing_queries(Harness& h) {
    HRESULT hr = h.queue->GetTimestampFrequency(&h.timestamp_frequency);
    if (FAILED(hr)) return hr;
    if (!h.timestamp_frequency) return E_FAIL;
    D3D12_QUERY_HEAP_DESC queries{};
    queries.Type = D3D12_QUERY_HEAP_TYPE_TIMESTAMP;
    queries.Count = 4;
    hr = h.device->CreateQueryHeap(&queries, IID_PPV_ARGS(&h.timestamps));
    if (FAILED(hr)) return hr;
    const auto heap = heap_properties(D3D12_HEAP_TYPE_READBACK);
    const auto buffer = buffer_desc(4 * sizeof(UINT64));
    return h.device->CreateCommittedResource(&heap, D3D12_HEAP_FLAG_NONE, &buffer,
        D3D12_RESOURCE_STATE_COPY_DEST, nullptr, IID_PPV_ARGS(&h.timestamp_readback));
}

extern "C" __declspec(dllexport) int32_t __cdecl gpu_timing_begin(
    void* handle, uint32_t frame, float jx, float jy, uint32_t final_frame) {
    if (!handle || frame >= 160 || final_frame > 1 || !std::isfinite(jx) || !std::isfinite(jy)) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    if (!h.timestamps) {
        const HRESULT hr = timing_queries(h);
        if (FAILED(hr)) return hr;
    }
    // Previous submission was fenced before rewriting/reusing either upload.
    HRESULT hr = set_temporal_frame(h, frame, jx, jy);
    if (FAILED(hr)) return hr;
    for (Role role : {Color, Motion}) {
        const auto desc = texture_desc(role);
        D3D12_PLACED_SUBRESOURCE_FOOTPRINT fp{};
        h.device->GetCopyableFootprints(&desc, 0, 1, 0, &fp, nullptr, nullptr, nullptr);
        transition(h.list.Get(), h.textures[role].Get(), D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                   D3D12_RESOURCE_STATE_COPY_DEST);
        D3D12_TEXTURE_COPY_LOCATION source{};
        source.pResource = h.uploads[role].Get();
        source.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
        source.PlacedFootprint = fp;
        D3D12_TEXTURE_COPY_LOCATION target{};
        target.pResource = h.textures[role].Get();
        target.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
        h.list->CopyTextureRegion(&target, 0, 0, 0, &source, nullptr);
        transition(h.list.Get(), h.textures[role].Get(), D3D12_RESOURCE_STATE_COPY_DEST,
                   D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE);
    }
    if (final_frame) {
        hr = performance_prepare_readback(handle);
        if (FAILED(hr)) return hr;
    }
    // Empty bracket and FSR start, on exactly the queue used for submission.
    h.list->EndQuery(h.timestamps.Get(), D3D12_QUERY_TYPE_TIMESTAMP, 0);
    h.list->EndQuery(h.timestamps.Get(), D3D12_QUERY_TYPE_TIMESTAMP, 1);
    h.list->EndQuery(h.timestamps.Get(), D3D12_QUERY_TYPE_TIMESTAMP, 2);
    return S_OK;
}

extern "C" __declspec(dllexport) int32_t __cdecl gpu_timing_finish(
    void* handle, uint32_t readback, Metrics* metrics, uint64_t* ticks) {
    if (!handle || !ticks) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    if (!h.timestamps) return E_INVALIDARG;
    h.list->EndQuery(h.timestamps.Get(), D3D12_QUERY_TYPE_TIMESTAMP, 3);
    h.list->ResolveQueryData(h.timestamps.Get(), D3D12_QUERY_TYPE_TIMESTAMP,
                            0, 4, h.timestamp_readback.Get(), 0);
    if (h.dynamic_render && readback) {
        const HRESULT hr = record_input_readbacks(h);
        if (FAILED(hr)) return hr;
        const HRESULT mask_hr = record_mask_inputs(h);
        if (FAILED(mask_hr)) return mask_hr;
    }
    // Resolving, final texture readback, submission, wait and map are outside
    // both dispatch brackets. A completed fence protects reuse of all queries.
    HRESULT hr = performance_finish(handle, readback, metrics);
    if (FAILED(hr)) return hr;
    if (h.dynamic_render && readback) {
        hr = check_input_readbacks(h);
        if (FAILED(hr)) return hr;
        hr = check_mask_inputs(h);
        if (FAILED(hr)) return hr;
        h.timing_max_red = metrics->max_red;
    }
    UINT64 frequency = 0;
    hr = h.queue->GetTimestampFrequency(&frequency);
    if (FAILED(hr)) return hr;
    if (frequency != h.timestamp_frequency) return E_FAIL;
    const D3D12_RANGE range{0, 4 * sizeof(UINT64)};
    void* mapped = nullptr;
    hr = h.timestamp_readback->Map(0, &range, &mapped);
    if (FAILED(hr)) return hr;
    std::memcpy(ticks, mapped, 4 * sizeof(UINT64));
    const D3D12_RANGE written{0, 0};
    h.timestamp_readback->Unmap(0, &written);
    ticks[4] = frequency;
    return S_OK;
}

// Active-size experiment only. Allocation dimensions never change. The scene
// moves 1/max-render-width UV per frame, independent of the active resolution.
extern "C" __declspec(dllexport) int32_t __cdecl active_timing_begin(
    void* handle, uint32_t frame, uint32_t width, uint32_t height,
    float jx, float jy, uint32_t readback) {
    if (!handle || frame >= 160 || !width || !height || width > render_width
        || height > render_height || readback > 1 || !std::isfinite(jx) || !std::isfinite(jy))
        return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    h.dynamic_render = true;
    h.timing_frame = frame; h.timing_width = width; h.timing_height = height;
    if (!h.timestamps) {
        const HRESULT hr = timing_queries(h);
        if (FAILED(hr)) return hr;
    }
    for (Role role : {Color, Depth, Motion}) {
        const auto desc = h.textures[role]->GetDesc();
        D3D12_PLACED_SUBRESOURCE_FOOTPRINT fp{};
        h.device->GetCopyableFootprints(&desc, 0, 1, 0, &fp, nullptr, nullptr, nullptr);
        uint8_t* mapped = nullptr;
        HRESULT hr = h.uploads[role]->Map(0, nullptr, reinterpret_cast<void**>(&mapped));
        if (FAILED(hr)) return hr;
        for (uint32_t y = 0; y < render_height; ++y) {
            auto* row = mapped + fp.Offset + y * fp.Footprint.RowPitch;
            for (uint32_t x = 0; x < render_width; ++x) {
                const bool active = x < width && y < height;
                if (role == Color) {
                    const float u = (x + 0.5f - jx) / width - static_cast<float>(frame) / render_width;
                    const float v = (y + 0.5f - jy) / height;
                    const float scale = h.timing_feature == 2 ? 8.f : 1.f;
                    const uint16_t pixel[] = {active ? half(scale * (0.1f + 0.75f * u)) : sentinel_half,
                        active ? half(scale * (0.1f + 0.75f * v)) : sentinel_half,
                        active ? half(scale * 0.2f) : sentinel_half, half(1.f)};
                    std::memcpy(row + x * sizeof(pixel), pixel, sizeof(pixel));
                } else if (role == Depth) {
                    const float depth = active ? 0.5f : 1.f;
                    std::memcpy(row + x * sizeof(depth), &depth, sizeof(depth));
                } else {
                    const uint16_t motion[] = {active ? half(frame ? -1.f / render_width : 0.f) : sentinel_half,
                        active ? half(0.f) : sentinel_half};
                    std::memcpy(row + x * sizeof(motion), motion, sizeof(motion));
                }
            }
        }
        h.uploads[role]->Unmap(0, nullptr);
        transition(h.list.Get(), h.textures[role].Get(), D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                   D3D12_RESOURCE_STATE_COPY_DEST);
        D3D12_TEXTURE_COPY_LOCATION source{}, target{};
        source.pResource = h.uploads[role].Get(); source.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
        source.PlacedFootprint = fp;
        target.pResource = h.textures[role].Get(); target.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
        h.list->CopyTextureRegion(&target, 0, 0, 0, &source, nullptr);
        transition(h.list.Get(), h.textures[role].Get(), D3D12_RESOURCE_STATE_COPY_DEST,
                   D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE);
    }
    if (readback) {
        const HRESULT hr = performance_prepare_readback(handle);
        if (FAILED(hr)) return hr;
    }
    h.list->EndQuery(h.timestamps.Get(), D3D12_QUERY_TYPE_TIMESTAMP, 0);
    h.list->EndQuery(h.timestamps.Get(), D3D12_QUERY_TYPE_TIMESTAMP, 1);
    h.list->EndQuery(h.timestamps.Get(), D3D12_QUERY_TYPE_TIMESTAMP, 2);
    return S_OK;
}

// FNV-1a is a bounded cross-consumer equality diagnostic, not an artefact hash.
extern "C" __declspec(dllexport) int32_t __cdecl active_timing_hashes(void* handle, uint64_t* hashes) {
    if (!handle || !hashes) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    const auto append = [](uint64_t& value, const uint8_t* bytes, size_t count) {
        for (size_t i = 0; i < count; ++i) value = (value ^ bytes[i]) * 1099511628211ull;
    };
    hashes[0] = hashes[1] = 14695981039346656037ull;
    std::vector<uint8_t> output(output_width * output_height * 8);
    HRESULT hr = gpu_copy_payload(handle, output.data(), static_cast<uint32_t>(output.size()));
    if (FAILED(hr)) return hr;
    append(hashes[0], output.data(), output.size());
    const uint32_t xs[] = {0, h.timing_width - 1, h.timing_width};
    const uint32_t ys[] = {0, h.timing_height - 1, h.timing_height};
    const size_t sample_count = h.timing_width < render_width && h.timing_height < render_height ? 3 : 2;
    uint32_t samples[3][7]{};
    for (Role role : {Color, Depth, Motion}) {
        void* mapped = nullptr;
        const D3D12_RANGE range{0, static_cast<SIZE_T>(h.input_bytes[role])};
        hr = h.input_readbacks[role]->Map(0, &range, &mapped);
        if (FAILED(hr)) return hr;
        const auto& fp = h.input_footprints[role];
        for (uint32_t y = 0; y < render_height; ++y)
            append(hashes[1], static_cast<uint8_t*>(mapped) + fp.Offset + y * fp.Footprint.RowPitch,
                   static_cast<size_t>(render_width) * (role == Color ? 8 : 4));
        for (size_t i = 0; i < sample_count; ++i) {
            const auto* pixel = static_cast<uint8_t*>(mapped) + fp.Offset + ys[i] * fp.Footprint.RowPitch
                + xs[i] * (role == Color ? 8 : 4);
            if (role == Depth) std::memcpy(&samples[i][4], pixel, sizeof(uint32_t));
            else for (size_t channel = 0; channel < (role == Color ? 4u : 2u); ++channel) {
                uint16_t value = 0;
                std::memcpy(&value, pixel + channel * sizeof(value), sizeof(value));
                samples[i][(role == Color ? 0 : 5) + channel] = value;
            }
        }
        const D3D12_RANGE written{0, 0};
        h.input_readbacks[role]->Unmap(0, &written);
    }
    for (size_t i = 0; i < sample_count; ++i)
        std::fprintf(stderr, "ACTIVE_INPUT %u %u %u %u %u %u %u %u %u %u\n",h.timing_frame,xs[i],ys[i],
            samples[i][0],samples[i][1],samples[i][2],samples[i][3],samples[i][4],samples[i][5],samples[i][6]);
    if (h.timing_feature) {
        uint64_t mask_hashes[2]{};
        for (uint32_t index = 0; index < 2; ++index) {
            if (!h.masks[index]) continue;
            void* mapped = nullptr;
            const D3D12_RANGE range{0, static_cast<SIZE_T>(h.mask_bytes[index])};
            hr = h.mask_readbacks[index]->Map(0, &range, &mapped);
            if (FAILED(hr)) return hr;
            mask_hashes[index] = 14695981039346656037ull;
            const auto& fp = h.mask_footprints[index];
            for (uint32_t y = 0; y < render_height; ++y)
                append(mask_hashes[index], static_cast<uint8_t*>(mapped) + fp.Offset + y * fp.Footprint.RowPitch, render_width);
            const D3D12_RANGE written{0, 0};
            h.mask_readbacks[index]->Unmap(0, &written);
        }
        uint32_t max_bits = 0;
        std::memcpy(&max_bits, &h.timing_max_red, sizeof(max_bits));
        if (h.timing_feature == 2 && !(h.timing_max_red > 1.f && h.timing_max_red < 64.f)) return E_FAIL;
        std::fprintf(stderr, "FEATURE_VALID %u %u %u %llu %llu\n", h.timing_frame, h.timing_feature, max_bits,
            static_cast<unsigned long long>(mask_hashes[0]), static_cast<unsigned long long>(mask_hashes[1]));
    }
    return S_OK;
}

extern "C" __declspec(dllexport) int32_t __cdecl feature_timing_configure(void* handle, uint32_t feature) {
    if (!handle || feature < 1 || feature > 2) return E_INVALIDARG;
    static_cast<Harness*>(handle)->timing_feature = feature;
    return S_OK;
}
