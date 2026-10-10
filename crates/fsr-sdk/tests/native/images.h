// SPDX-License-Identifier: MPL-2.0

// Test-only analytic image generation. Included after Harness/Role definitions
// inside the GPU helper's anonymous namespace.

uint16_t half(float value) {
    uint32_t bits;
    std::memcpy(&bits, &value, sizeof(bits));
    const uint32_t sign = (bits >> 16) & 0x8000;
    const int exponent = static_cast<int>((bits >> 23) & 0xff) - 127 + 15;
    const uint32_t fraction = bits & 0x7fffff;
    if (exponent <= 0) return static_cast<uint16_t>(sign);
    if (exponent >= 31) return static_cast<uint16_t>(sign | 0x7c00);
    return static_cast<uint16_t>(sign | (exponent << 10) | (fraction >> 13));
}

float unhalf(uint16_t value) {
    const uint32_t sign = static_cast<uint32_t>(value & 0x8000) << 16;
    const uint32_t exponent = (value >> 10) & 31;
    const uint32_t fraction = value & 1023;
    uint32_t bits;
    if (exponent == 0) {
        if (fraction == 0) bits = sign;
        else {
            uint32_t mantissa = fraction;
            int shift = 0;
            while ((mantissa & 1024) == 0) { mantissa <<= 1; ++shift; }
            bits = sign | (static_cast<uint32_t>(127 - 14 - shift) << 23)
                | ((mantissa & 1023) << 13);
        }
    } else if (exponent == 31) {
        bits = sign | 0x7f800000 | (fraction << 13);
    } else {
        bits = sign | ((exponent + 112) << 23) | (fraction << 13);
    }
    float result;
    std::memcpy(&result, &bits, sizeof(result));
    return result;
}

void fill_row(Role role, uint32_t y, uint8_t* row, float red_shift = 0.0f) {
    const auto desc = texture_desc(role);
    for (uint32_t x = 0; x < desc.Width; ++x) {
        if (role == Color) {
            // CPU-generated image sampled at the same known subpixel jitter
            // passed in the Rust descriptor. It needs no camera or renderer.
            const float red = 0.1f + red_shift + 0.75f * (x + 0.25f) / render_width;
            const float green = 0.1f + 0.75f * (y + 0.75f) / render_height;
            const float blue = ((x / 16 + y / 16) & 1) ? 0.4f : 0.2f;
            uint16_t pixel[4] = { half(red), half(green), half(blue), half(1.0f) };
            std::memcpy(row + x * sizeof(pixel), pixel, sizeof(pixel));
        } else if (role == Output) {
            uint16_t pixel[4] = { sentinel_half, sentinel_half, sentinel_half, sentinel_half };
            std::memcpy(row + x * sizeof(pixel), pixel, sizeof(pixel));
        } else if (role == Motion) {
            uint16_t pixel[2] = { 0, 0 };
            std::memcpy(row + x * sizeof(pixel), pixel, sizeof(pixel));
        } else {
            const float value = role == Exposure ? 1.0f : 0.5f;
            std::memcpy(row + x * sizeof(value), &value, sizeof(value));
        }
    }
}

// Rewrite only uploads whose copy commands have not yet been submitted.
// Projection jitter affects the analytic image; UV motion excludes jitter.
HRESULT set_temporal_frame(Harness& h, uint32_t frame, float jitter_x, float jitter_y) {
    for (Role role : {Color, Motion}) {
        const auto desc = texture_desc(role);
        D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint{};
        h.device->GetCopyableFootprints(&desc, 0, 1, 0, &footprint, nullptr, nullptr, nullptr);
        uint8_t* mapped = nullptr;
        HRESULT hr = h.uploads[role]->Map(0, nullptr, reinterpret_cast<void**>(&mapped));
        if (FAILED(hr)) return hr;
        for (uint32_t y = 0; y < render_height; ++y) {
            uint8_t* row = mapped + footprint.Offset + y * footprint.Footprint.RowPitch;
            for (uint32_t x = 0; x < render_width; ++x) {
                if (role == Color) {
                    // A positive projection jitter shifts the projected image
                    // positively; a fixed pixel therefore samples scene(x-j).
                    const float sx = x + 0.5f - jitter_x;
                    const float sy = y + 0.5f - jitter_y;
                    const float u = sx - static_cast<float>(frame);
                    uint16_t pixel[4] = {
                        half(0.1f + 0.75f * u / render_width),
                        half(0.1f + 0.75f * sy / render_height), half(0.2f), half(1.0f)
                    };
                    std::memcpy(row + x * sizeof(pixel), pixel, sizeof(pixel));
                } else {
                    uint16_t pixel[2] = {half(frame == 0 ? 0.0f : -1.0f / render_width), 0};
                    std::memcpy(row + x * sizeof(pixel), pixel, sizeof(pixel));
                }
            }
        }
        h.uploads[role]->Unmap(0, nullptr);
    }
    return S_OK;
}

// Controlled semantic comparisons use the same submission/fence machinery.
// No SDK calls occur here. The scene translates two render pixels per frame;
// vectors are UV current-to-previous, with projection jitter excluded.
HRESULT set_semantic_frame(
    Harness& h, uint32_t frame, float jitter_x, float jitter_y, uint32_t reverse_motion) {
    if (reverse_motion > 1 || !std::isfinite(jitter_x) || !std::isfinite(jitter_y))
        return E_INVALIDARG;
    for (Role role : {Color, Motion}) {
        const auto desc = texture_desc(role);
        D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint{};
        h.device->GetCopyableFootprints(&desc, 0, 1, 0, &footprint, nullptr, nullptr, nullptr);
        uint8_t* mapped = nullptr;
        HRESULT hr = h.uploads[role]->Map(0, nullptr, reinterpret_cast<void**>(&mapped));
        if (FAILED(hr)) return hr;
        for (uint32_t y = 0; y < render_height; ++y) {
            uint8_t* row = mapped + footprint.Offset + y * footprint.Footprint.RowPitch;
            for (uint32_t x = 0; x < render_width; ++x) {
                if (role == Color) {
                    const float u = x + 0.5f - jitter_x - 2.0f * frame;
                    const float v = y + 0.5f - jitter_y;
                    const float red = 0.5f + 0.4f * std::sin((u + 0.37f * v) * 0.8f);
                    uint16_t pixel[4] = {half(red), half(red), half(red), half(1.0f)};
                    std::memcpy(row + x * sizeof(pixel), pixel, sizeof(pixel));
                } else {
                    const float motion = (reverse_motion ? 2.0f : -2.0f) / render_width;
                    uint16_t pixel[2] = {half(frame == 0 ? 0.0f : motion), 0};
                    std::memcpy(row + x * sizeof(pixel), pixel, sizeof(pixel));
                }
            }
        }
        h.uploads[role]->Unmap(0, nullptr);
    }
    return S_OK;
}
