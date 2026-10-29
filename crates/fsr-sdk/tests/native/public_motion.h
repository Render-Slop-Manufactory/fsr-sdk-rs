// SPDX-License-Identifier: MPL-2.0

// Public-route input equipment. Scales are applied by the Rust/native consumer,
// never by this helper; stored input bytes are independently audited in Python.
struct PublicMotionConfig {
    uint32_t active_width, active_height, display_motion, format, motion_uav, frame;
    float stored_x, stored_y, jitter_x, jitter_y;
};
static_assert(sizeof(PublicMotionConfig) == 40 && alignof(PublicMotionConfig) == 4);

extern "C" __declspec(dllexport) int32_t __cdecl gpu_public_motion_create(
    ID3D12Device* device, const PublicMotionConfig* config, void** handle,
    ID3D12GraphicsCommandList** list, ID3D12Resource** resources) {
    if (!device || !config || config->display_motion > 1 || config->format > 1 || config->motion_uav > 1
        || config->frame > 2 || !std::isfinite(config->stored_x) || !std::isfinite(config->stored_y)
        || !std::isfinite(config->jitter_x) || !std::isfinite(config->jitter_y)) return E_INVALIDARG;
    const OutputConfig output{config->active_width, config->active_height, 640, 360,
        config->display_motion ? 640u : 320u, config->display_motion ? 360u : 180u,
        config->display_motion ? 1u : 0u, config->frame, config->jitter_x, config->jitter_y};
    const auto format = config->format ? DXGI_FORMAT_R32G32_FLOAT : DXGI_FORMAT_R16G16_FLOAT;
    D3D12_FEATURE_DATA_FORMAT_SUPPORT support{format, {}, {}};
    HRESULT hr = device->CheckFeatureSupport(D3D12_FEATURE_FORMAT_SUPPORT, &support, sizeof(support));
    if (FAILED(hr)) return hr;
    std::fprintf(stderr, "PUBLIC_MOTION_CAPS {\"support1\":%u,\"support2\":%u}\n",
        static_cast<uint32_t>(support.Support1), static_cast<uint32_t>(support.Support2));
    hr = output_create(device, &output, handle, list, resources, format, config->motion_uav != 0);
    if (FAILED(hr)) return hr;
    auto& o = *static_cast<OutputHarness*>(*handle);
    for (const auto role : {Color, Motion}) {
        uint8_t* mapped = nullptr;
        hr = o.base.uploads[role]->Map(0, nullptr, reinterpret_cast<void**>(&mapped));
        if (FAILED(hr)) return hr;
        const auto desc = o.base.textures[role]->GetDesc();
        for (uint32_t y = 0; y < desc.Height; ++y) {
            auto* row = mapped + o.footprints[role].Offset + y * o.footprints[role].Footprint.RowPitch;
            for (uint32_t x = 0; x < desc.Width; ++x) {
                if (role == Color) {
                    const auto wave = [](float value) {
                        const float tile = value - std::floor(value / 16.0f) * 16.0f;
                        return std::fabs(tile - 8.0f) / 8.0f;
                    };
                    const float u = x + 0.5f + config->jitter_x - config->frame * 4.0f;
                    const float v = y + 0.5f + config->jitter_y - config->frame * 2.0f;
                    const uint16_t pixel[4] = {half(0.125f + 0.75f * wave(u)),
                        half(0.125f + 0.75f * wave(v)), half(0.25f), half(1.0f)};
                    std::memcpy(row + x * 8, pixel, 8);
                } else {
                    const float values[2] = {config->stored_x, config->stored_y};
                    if (config->format) std::memcpy(row + x * 8, values, 8);
                    else {
                        const uint16_t pixel[2] = {half(values[0]), half(values[1])};
                        std::memcpy(row + x * 4, pixel, 4);
                    }
                }
            }
        }
        o.base.uploads[role]->Unmap(0, nullptr);
    }
    return S_OK;
}
