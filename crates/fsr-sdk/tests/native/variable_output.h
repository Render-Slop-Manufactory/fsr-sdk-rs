// SPDX-License-Identifier: MPL-2.0

// Dedicated experimental equipment. No FidelityFX calls or public API changes.
struct OutputConfig {
    uint32_t active_width, active_height, output_width, output_height;
    uint32_t motion_width, motion_height, motion_mode, frame;
    float jitter_x, jitter_y;
};
static_assert(sizeof(OutputConfig) == 40 && alignof(OutputConfig) == 4);
struct OutputMetrics {
    uint32_t finite_active, written_active, written_outside;
    uint32_t rightmost_written, bottommost_written, input_mismatches;
    uint64_t debug_errors;
    int32_t device_removed;
    uint32_t completed;
};
static_assert(sizeof(OutputMetrics) == 40 && alignof(OutputMetrics) == 8);

struct OutputHarness {
    Harness base;
    OutputConfig config{};
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprints[Count]{};
    UINT64 sizes[Count]{};
    ComPtr<ID3D12Resource> snapshots[Count];
};

static HRESULT output_texture(OutputHarness& o, Role role, DXGI_FORMAT motion_format = DXGI_FORMAT_R16G16_FLOAT, bool motion_uav = false) {
    auto& h = o.base;
    auto desc = texture_desc(role);
    if (role == Motion) desc.Format = motion_format;
    if (role == Motion && motion_uav) desc.Flags |= D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
    desc.Width = role == Output ? o.config.output_width : role == Motion ? o.config.motion_width
        : role == Exposure ? 1 : 320;
    desc.Height = role == Output ? o.config.output_height : role == Motion ? o.config.motion_height
        : role == Exposure ? 1 : 180;
    const auto device_heap = heap_properties(D3D12_HEAP_TYPE_DEFAULT);
    HRESULT hr = h.device->CreateCommittedResource(&device_heap, D3D12_HEAP_FLAG_NONE, &desc,
        D3D12_RESOURCE_STATE_COPY_DEST, nullptr, IID_PPV_ARGS(&h.textures[role]));
    if (FAILED(hr)) return hr;
    h.device->GetCopyableFootprints(&desc, 0, 1, 0, &o.footprints[role], nullptr, nullptr, &o.sizes[role]);
    const auto buffer = buffer_desc(o.sizes[role]);
    const auto upload_heap = heap_properties(D3D12_HEAP_TYPE_UPLOAD);
    const auto readback_heap = heap_properties(D3D12_HEAP_TYPE_READBACK);
    hr = h.device->CreateCommittedResource(&upload_heap, D3D12_HEAP_FLAG_NONE, &buffer,
        D3D12_RESOURCE_STATE_GENERIC_READ, nullptr, IID_PPV_ARGS(&h.uploads[role]));
    if (FAILED(hr)) return hr;
    hr = h.device->CreateCommittedResource(&readback_heap, D3D12_HEAP_FLAG_NONE, &buffer,
        D3D12_RESOURCE_STATE_COPY_DEST, nullptr, IID_PPV_ARGS(&o.snapshots[role]));
    if (FAILED(hr)) return hr;
    uint8_t* mapped = nullptr;
    hr = h.uploads[role]->Map(0, nullptr, reinterpret_cast<void**>(&mapped));
    if (FAILED(hr)) return hr;
    std::memset(mapped, 0, static_cast<size_t>(o.sizes[role]));
    for (uint32_t y = 0; y < desc.Height; ++y) {
        auto* row = mapped + o.footprints[role].Offset + y * o.footprints[role].Footprint.RowPitch;
        for (uint32_t x = 0; x < desc.Width; ++x) {
            if (role == Output) {
                const uint16_t pixel[4] = {sentinel_half, sentinel_half, sentinel_half, sentinel_half};
                std::memcpy(row + x * sizeof(pixel), pixel, sizeof(pixel));
            } else if (role == Color) {
                // Static plane translated by 1/128 normalized units per frame.
                const float u = (x + 0.5f + o.config.jitter_x) / 320.0f - o.config.frame / 128.0f;
                const float v = (y + 0.5f + o.config.jitter_y) / 180.0f - o.config.frame / 128.0f;
                const uint16_t pixel[4] = {half(0.2f + 0.4f * u), half(0.2f + 0.4f * v), half(0.3f), half(1.0f)};
                std::memcpy(row + x * sizeof(pixel), pixel, sizeof(pixel));
            } else if (role == Motion) {
                const float shift = o.config.frame == 0 ? 0.0f : -1.0f / 128.0f;
                const float sx = o.config.motion_mode == 2 ? static_cast<float>(o.config.active_width) : o.config.motion_mode == 3 ? 320.0f : 1.0f;
                const float sy = o.config.motion_mode == 2 ? static_cast<float>(o.config.active_height) : o.config.motion_mode == 3 ? 180.0f : 1.0f;
                const uint16_t motion[2] = {half(shift * sx), half(shift * sy)};
                std::memcpy(row + x * sizeof(motion), motion, sizeof(motion));
            } else {
                const float value = role == Depth ? 0.25f : 1.0f;
                std::memcpy(row + x * sizeof(value), &value, sizeof(value));
            }
        }
    }
    h.uploads[role]->Unmap(0, nullptr);
    D3D12_TEXTURE_COPY_LOCATION from{}, to{};
    from.pResource = h.uploads[role].Get(); from.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
    from.PlacedFootprint = o.footprints[role];
    to.pResource = h.textures[role].Get(); to.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
    h.list->CopyTextureRegion(&to, 0, 0, 0, &from, nullptr);
    transition(h.list.Get(), h.textures[role].Get(), D3D12_RESOURCE_STATE_COPY_DEST,
        role == Output ? D3D12_RESOURCE_STATE_UNORDERED_ACCESS : D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE);
    return S_OK;
}

static int32_t output_create(
    ID3D12Device* device, const OutputConfig* config, void** handle,
    ID3D12GraphicsCommandList** list, ID3D12Resource** resources,
    DXGI_FORMAT motion_format = DXGI_FORMAT_R16G16_FLOAT, bool motion_uav = false) {
    if (!device || !config || !handle || !list || !resources || !config->active_width || !config->active_height
        || config->active_width > config->output_width || config->active_height > config->output_height
        || config->output_width > 672 || config->output_height > 384 || config->motion_mode > 3
        || !config->motion_width || !config->motion_height || config->motion_width > 672 || config->motion_height > 384)
        return E_INVALIDARG;
    *handle = nullptr; *list = nullptr;
    auto o = std::make_unique<OutputHarness>(); o->config = *config;
    auto& h = o->base; h.device = device;
    D3D12_COMMAND_QUEUE_DESC queue{}; queue.Type = D3D12_COMMAND_LIST_TYPE_DIRECT;
    HRESULT hr = device->CreateCommandQueue(&queue, IID_PPV_ARGS(&h.queue));
    if (FAILED(hr)) return hr;
    hr = device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT, IID_PPV_ARGS(&h.allocator));
    if (FAILED(hr)) return hr;
    hr = device->CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, h.allocator.Get(), nullptr, IID_PPV_ARGS(&h.list));
    if (FAILED(hr)) return hr;
    hr = device->CreateFence(0, D3D12_FENCE_FLAG_NONE, IID_PPV_ARGS(&h.fence));
    if (FAILED(hr)) return hr;
    for (int i = 0; i < Count; ++i) {
        hr = output_texture(*o, static_cast<Role>(i), motion_format, motion_uav);
        if (FAILED(hr)) return hr;
    }
    *list = h.list.Get(); (*list)->AddRef();
    for (int i = 0; i < Count; ++i) { resources[i] = h.textures[i].Get(); resources[i]->AddRef(); }
    *handle = o.release();
    return S_OK;
}

extern "C" __declspec(dllexport) int32_t __cdecl gpu_output_create(
    ID3D12Device* device, const OutputConfig* config, void** handle,
    ID3D12GraphicsCommandList** list, ID3D12Resource** resources) {
    return output_create(device, config, handle, list, resources);
}

extern "C" __declspec(dllexport) int32_t __cdecl gpu_output_finish(void* handle, OutputMetrics* metrics) {
    if (!handle || !metrics) return E_INVALIDARG;
    auto& o = *static_cast<OutputHarness*>(handle); auto& h = o.base; *metrics = {};
    for (int i = 0; i < Count; ++i) {
        const auto state = i == Output ? D3D12_RESOURCE_STATE_UNORDERED_ACCESS : D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE;
        transition(h.list.Get(), h.textures[i].Get(), state, D3D12_RESOURCE_STATE_COPY_SOURCE);
        D3D12_TEXTURE_COPY_LOCATION from{}, to{};
        from.pResource = h.textures[i].Get(); from.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
        to.pResource = o.snapshots[i].Get(); to.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
        to.PlacedFootprint = o.footprints[i];
        h.list->CopyTextureRegion(&to, 0, 0, 0, &from, nullptr);
        transition(h.list.Get(), h.textures[i].Get(), D3D12_RESOURCE_STATE_COPY_SOURCE, state);
    }
    HRESULT hr = h.list->Close();
    if (FAILED(hr)) return hr; // Caller discards this never-submitted recording.
    ID3D12CommandList* lists[] = {h.list.Get()}; h.queue->ExecuteCommandLists(1, lists);
    hr = h.queue->Signal(h.fence.Get(), 1);
    if (FAILED(hr)) return hr;
    hr = wait_for_fence(h);
    if (FAILED(hr)) return hr;
    const auto completed = h.fence->GetCompletedValue();
    metrics->device_removed = h.device->GetDeviceRemovedReason();
    if (completed == UINT64_MAX || completed < 1 || metrics->device_removed != S_OK) return E_FAIL;
    metrics->completed = 1; metrics->debug_errors = debug_error_count(h.device.Get());
    for (int i = 0; i < Count; ++i) {
        uint8_t* snapshot = nullptr; uint8_t* upload = nullptr;
        hr = o.snapshots[i]->Map(0, nullptr, reinterpret_cast<void**>(&snapshot));
        if (FAILED(hr)) return hr;
        hr = h.uploads[i]->Map(0, nullptr, reinterpret_cast<void**>(&upload));
        if (FAILED(hr)) { o.snapshots[i]->Unmap(0, nullptr); return hr; }
        const auto desc = h.textures[i]->GetDesc();
        const uint32_t stride = i == Color || i == Output || desc.Format == DXGI_FORMAT_R32G32_FLOAT ? 8 : 4;
        for (uint32_t y = 0; y < desc.Height; ++y) {
            const auto offset = o.footprints[i].Offset + y * o.footprints[i].Footprint.RowPitch;
            if (i != Output) {
                for (uint32_t x = 0; x < desc.Width * stride; ++x)
                    metrics->input_mismatches += snapshot[offset + x] != upload[offset + x] ? 1u : 0u;
                continue;
            }
            for (uint32_t x = 0; x < desc.Width; ++x) {
                uint16_t pixel[4]; std::memcpy(pixel, snapshot + offset + x * 8, 8);
                const bool written = std::any_of(pixel, pixel + 4, [](uint16_t v) { return v != sentinel_half; });
                if (written) { metrics->rightmost_written = std::max(metrics->rightmost_written, x + 1);
                    metrics->bottommost_written = std::max(metrics->bottommost_written, y + 1); }
                if (x < o.config.active_width && y < o.config.active_height) {
                    metrics->finite_active += std::all_of(pixel, pixel + 4, [](uint16_t v) { return std::isfinite(unhalf(v)); }) ? 1u : 0u;
                    metrics->written_active += written ? 1u : 0u;
                } else { metrics->written_outside += written ? 1u : 0u; }
            }
        }
        h.uploads[i]->Unmap(0, nullptr); o.snapshots[i]->Unmap(0, nullptr);
    }
    return S_OK;
}

extern "C" __declspec(dllexport) int32_t __cdecl gpu_output_copy(void* handle, uint32_t role, uint8_t* output, uint32_t count) {
    if (!handle || !output || role >= Count) return E_INVALIDARG;
    auto& o = *static_cast<OutputHarness*>(handle);
    const auto desc = o.base.textures[role]->GetDesc();
    const uint32_t stride = role == Color || role == Output || desc.Format == DXGI_FORMAT_R32G32_FLOAT ? 8 : 4;
    if (count != desc.Width * desc.Height * stride || o.base.fence->GetCompletedValue() != 1) return E_INVALIDARG;
    uint8_t* mapped = nullptr;
    HRESULT hr = o.snapshots[role]->Map(0, nullptr, reinterpret_cast<void**>(&mapped));
    if (FAILED(hr)) return hr;
    for (uint32_t y = 0; y < desc.Height; ++y)
        std::memcpy(output + y * desc.Width * stride,
            mapped + o.footprints[role].Offset + y * o.footprints[role].Footprint.RowPitch, static_cast<size_t>(desc.Width * stride));
    o.snapshots[role]->Unmap(0, nullptr);
    return S_OK;
}

extern "C" __declspec(dllexport) void __cdecl gpu_output_discard(void* handle) {
    static_cast<OutputHarness*>(handle)->base.list.Reset();
}
extern "C" __declspec(dllexport) void __cdecl gpu_output_destroy(void* handle) {
    delete static_cast<OutputHarness*>(handle);
}
