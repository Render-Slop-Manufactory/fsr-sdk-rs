// SPDX-License-Identifier: MPL-2.0

// Diagnostic inputs only. Scenes and velocities are frozen by the coordinator;
// this export does not call AMD or change the original calibration generator.
int32_t diagnosis_frame(
    void* handle, uint32_t scene, uint32_t frame, float jx, float jy,
    float vx, float vy, uint32_t motion, uint32_t cut_frame) {
    if (!handle || scene > 2 || frame > 255 || motion > 2
        || !std::isfinite(jx) || !std::isfinite(jy)
        || !std::isfinite(vx) || !std::isfinite(vy)) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    const bool cut = frame >= cut_frame;
    const double left = 100.0 + static_cast<double>(vx) * frame - (cut ? 96.0 : 0.0);
    const double top = 50.0 + static_cast<double>(vy) * frame + (cut ? 48.0 : 0.0);
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
                const bool fg = scene != 0 && sx >= left && sx < left + 64.0
                    && sy >= top && sy < top + 44.0;
                if (role == Color) {
                    double red = scene == 0 ? 0.125 + sx / 1024.0 : fg ? 0.875 : 0.125;
                    double green = scene == 0 ? 0.25 + sy / 1024.0 : fg ? 0.125 : 0.25;
                    double blue = scene == 0 ? 0.375 + (sx + sy) / 2048.0 : fg ? 0.0625 : 0.375;
                    if (scene == 2) {
                        const double tx = fg ? sx - left : sx + (cut ? 24.0 : 0.0);
                        const double ty = fg ? sy - top : sy - (cut ? 12.0 : 0.0);
                        red = fg ? 0.65 + 0.15 * std::sin(0.4 * tx) : 0.25 + 0.1 * std::sin(0.35 * tx);
                        green = fg ? 0.2 + 0.1 * std::sin(0.3 * ty) : 0.3 + 0.1 * std::sin(0.27 * ty);
                        blue = fg ? 0.15 + 0.08 * std::sin(0.23 * tx + 0.19 * ty)
                            : 0.35 + 0.1 * std::sin(0.21 * tx + 0.17 * ty);
                    }
                    const uint16_t pixel[4] = {half(static_cast<float>(red)),
                        half(static_cast<float>(green)), half(static_cast<float>(blue)), half(1.0f)};
                    std::memcpy(row + x * 8, pixel, sizeof(pixel));
                } else if (role == Depth) {
                    const double z = fg ? 2.0 : 8.0;
                    const float d = static_cast<float>(20.0 / 19.5 - 10.0 / (19.5 * z));
                    std::memcpy(row + x * 4, &d, sizeof(d));
                } else {
                    const double factor = motion == 1 ? -1.0 : motion == 2 ? 0.0 : 1.0;
                    const double mx = frame == cut_frame ? (fg ? 96.0 - vx : 24.0)
                        : fg && frame && motion != 2 ? -static_cast<double>(vx) * factor : 0.0;
                    const double my = frame == cut_frame ? (fg ? -48.0 - vy : -12.0)
                        : fg && frame && motion != 2 ? -static_cast<double>(vy) * factor : 0.0;
                    // Canonical positive zero is independent of /fp signed-zero
                    // optimization and matches the byte-exact input oracle.
                    const uint16_t pixel[2] = {half(mx == 0.0 ? 0.0f : static_cast<float>(mx / 320.0)),
                        half(my == 0.0 ? 0.0f : static_cast<float>(my / 180.0))};
                    std::memcpy(row + x * 4, pixel, sizeof(pixel));
                }
            }
        }
        h.uploads[role]->Unmap(0, nullptr);
    }
    return S_OK;
}

extern "C" __declspec(dllexport) int32_t __cdecl temporal_diagnosis_set_frame(
    void* handle, uint32_t scene, uint32_t frame, float jx, float jy,
    float vx, float vy, uint32_t motion) {
    return diagnosis_frame(handle, scene, frame, jx, jy, vx, vy, motion, UINT32_MAX);
}

extern "C" __declspec(dllexport) int32_t __cdecl temporal_reset_set_frame(
    void* handle, uint32_t scene, uint32_t frame, float jx, float jy,
    float vx, float vy, uint32_t motion, uint32_t cut_frame) {
    if (scene != 2 || motion != 0 || cut_frame == 0 || cut_frame >= 128 || frame >= 128) return E_INVALIDARG;
    return diagnosis_frame(handle, scene, frame, jx, jy, vx, vy, motion, cut_frame);
}
