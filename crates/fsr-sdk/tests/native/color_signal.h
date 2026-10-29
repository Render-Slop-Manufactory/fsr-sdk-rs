// SPDX-License-Identifier: MPL-2.0
#pragma once

// Probe-only upload of independently generated RGBA16F bytes. No transfer
// function, exposure multiplication or AMD call occurs in this equipment.
extern "C" __declspec(dllexport) int32_t __cdecl gpu_set_color_signal(
    void* handle, const uint8_t* rgba, uint32_t bytes, float exposure) {
    if (!handle || !rgba || bytes != render_width * render_height * 8
        || !std::isfinite(exposure) || exposure <= 0.0f) return E_INVALIDARG;
    auto& h = *static_cast<Harness*>(handle);
    for (Role role : {Color, Depth, Motion, Exposure}) {
        const auto desc = h.textures[role]->GetDesc();
        D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint{};
        h.device->GetCopyableFootprints(&desc, 0, 1, 0, &footprint, nullptr, nullptr, nullptr);
        uint8_t* mapped = nullptr;
        const HRESULT hr = h.uploads[role]->Map(0, nullptr, reinterpret_cast<void**>(&mapped));
        if (FAILED(hr)) return hr;
        for (uint32_t y = 0; y < desc.Height; ++y) {
            auto* row = mapped + footprint.Offset + y * footprint.Footprint.RowPitch;
            if (role == Color) {
                std::memcpy(row, rgba + y * render_width * 8, render_width * 8);
            } else {
                for (uint32_t x = 0; x < desc.Width; ++x) {
                    // Motion RG16F is two zero half values; depth/exposure are R32F.
                    const float value = role == Motion ? 0.0f : role == Depth ? 0.25f : exposure;
                    std::memcpy(row + x * 4, &value, sizeof(value));
                }
            }
        }
        h.uploads[role]->Unmap(0, nullptr);
    }
    // Reuse the existing full four-input readback/check path, including exposure.
    // This laboratory flag does not set any SDK creation/dispatch bit.
    h.hdr = true;
    return S_OK;
}

// Retain the established helper export for existing probe binaries.
extern "C" __declspec(dllexport) int32_t __cdecl gpu_set_colour_signal(
    void* handle, const uint8_t* rgba, uint32_t bytes, float exposure) {
    return gpu_set_color_signal(handle, rgba, bytes, exposure);
}
