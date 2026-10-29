// SPDX-License-Identifier: MPL-2.0

// Test-only D014 stationary fixtures. Included by gpu.cpp after its equipment.
// Rewrite only the colour upload before submission; the harness already supplies
// depth 0.5, exposure 1, zero motion and an output sentinel. No AMD calls here.
extern "C" __declspec(dllexport) int32_t __cdecl sharpening_set_frame(
    void* handle, uint32_t scene, float jitter_x, float jitter_y) {
    if (!handle || scene > 1 || !std::isfinite(jitter_x) || !std::isfinite(jitter_y))
        return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    const auto desc = texture_desc(Color);
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint{};
    h.device->GetCopyableFootprints(&desc, 0, 1, 0, &footprint, nullptr, nullptr, nullptr);
    uint8_t* mapped = nullptr;
    HRESULT hr = h.uploads[Color]->Map(0, nullptr, reinterpret_cast<void**>(&mapped));
    if (FAILED(hr)) return hr;
    for (uint32_t y = 0; y < render_height; ++y) {
        uint8_t* row = mapped + footprint.Offset + y * footprint.Footprint.RowPitch;
        for (uint32_t x = 0; x < render_width; ++x) {
            const float u = x + 0.5f - jitter_x;
            const float v = y + 0.5f - jitter_y;
            const float red = scene == 0
                ? 0.5f + 0.4f * std::sin(0.8f * (u + 0.37f * v))
                : u < 128.0f + 0.37f * v ? 0.1f : 0.9f;
            uint16_t pixel[4] = {half(red), half(red), half(red), half(1.0f)};
            std::memcpy(row + x * sizeof(pixel), pixel, sizeof(pixel));
        }
    }
    h.uploads[Color]->Unmap(0, nullptr);
    return S_OK;
}

// Preserve actual red half bits, excluding pitched-row padding. Call after fence.
extern "C" __declspec(dllexport) int32_t __cdecl sharpening_copy_red_bits(
    void* handle, uint16_t* output, uint32_t count) {
    if (!handle || !output || count != output_width * output_height) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    uint8_t* mapped = nullptr;
    D3D12_RANGE range{0, static_cast<SIZE_T>(h.output_bytes)};
    HRESULT hr = h.readback->Map(0, &range, reinterpret_cast<void**>(&mapped));
    if (FAILED(hr)) return hr;
    for (uint32_t y = 0; y < output_height; ++y) {
        const uint8_t* row = mapped + h.output_footprint.Offset
            + y * h.output_footprint.Footprint.RowPitch;
        for (uint32_t x = 0; x < output_width; ++x)
            std::memcpy(&output[y * output_width + x], row + x * 8, sizeof(uint16_t));
    }
    D3D12_RANGE written{0, 0};
    h.readback->Unmap(0, &written);
    return S_OK;
}
