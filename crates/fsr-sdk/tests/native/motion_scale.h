// SPDX-License-Identifier: MPL-2.0

// Probe-only inputs layered on the existing fenced/readback equipment.
struct MotionConfig {
    uint32_t format, axis, frame;
    float stored_x, stored_y;
};
static_assert(sizeof(MotionConfig) == 20 && alignof(MotionConfig) == 4);

extern "C" __declspec(dllexport) int32_t __cdecl gpu_motion_create(
    ID3D12Device* device, const MotionConfig* config, void** handle,
    ID3D12GraphicsCommandList** list, ID3D12Resource** resources) {
    if (!config || config->format > 1 || config->axis > 2 || config->frame > 3
        || !std::isfinite(config->stored_x) || !std::isfinite(config->stored_y)) return E_INVALIDARG;
    const OutputConfig output{640, 360, 640, 360, 320, 180, 0, config->frame, 0.0f, 0.0f};
    const auto format = config->format ? DXGI_FORMAT_R32G32_FLOAT : DXGI_FORMAT_R16G16_FLOAT;
    HRESULT hr = output_create(device, &output, handle, list, resources, format);
    if (FAILED(hr)) return hr;
    auto& o = *static_cast<OutputHarness*>(*handle);
    // These uploads have recorded copies but have not been submitted. The helper
    // retains them through the queue fence, so every copy reads the final bytes.
    for (const auto role : {Color, Motion}) {
        uint8_t* mapped = nullptr;
        hr = o.base.uploads[role]->Map(0, nullptr, reinterpret_cast<void**>(&mapped));
        if (FAILED(hr)) return hr;
        for (uint32_t y = 0; y < 180; ++y) {
            auto* row = mapped + o.footprints[role].Offset + y * o.footprints[role].Footprint.RowPitch;
            for (uint32_t x = 0; x < 320; ++x) {
                if (role == Color) {
                    const uint32_t u = (x + 320 - (config->axis != 1 ? 4 * config->frame : 0)) % 320;
                    const uint32_t v = (y + 180 - (config->axis != 0 ? 2 * config->frame : 0)) % 180;
                    // Dyadic levels permit an exact independent Python input oracle.
                    const uint16_t pixel[4] = {half((1 + (u / 4) % 7) / 8.0f),
                        half((1 + (v / 3) % 7) / 8.0f), half(((u / 8 + v / 8) % 2) ? 0.75f : 0.25f), half(1.0f)};
                    std::memcpy(row + x * 8, pixel, 8);
                } else {
                    const float values[2] = {config->frame ? config->stored_x : 0.0f,
                        config->frame ? config->stored_y : 0.0f};
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
