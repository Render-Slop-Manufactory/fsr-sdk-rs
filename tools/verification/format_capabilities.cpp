// SPDX-License-Identifier: MPL-2.0
#include <d3d12.h>
#include <dxgi1_6.h>
#include <wrl/client.h>
#include <cstdio>
#include <cstdlib>
#include <string>
#include <dx12/ffx_api_dx12.h>
using Microsoft::WRL::ComPtr;
int main(int argc, char** argv) {
    if (argc != 2) return 2;
    ComPtr<IDXGIFactory6> factory;
    ComPtr<IDXGIAdapter1> adapter;
    ComPtr<ID3D12Device> device;
    if (FAILED(CreateDXGIFactory2(0, IID_PPV_ARGS(&factory))) ||
        FAILED(factory->EnumAdapters1(static_cast<UINT>(std::stoul(argv[1])), &adapter)) ||
        FAILED(D3D12CreateDevice(adapter.Get(), D3D_FEATURE_LEVEL_12_0, IID_PPV_ARGS(&device)))) return 3;
    DXGI_ADAPTER_DESC1 desc{}; adapter->GetDesc1(&desc);
    std::printf("{\"adapter_vendor\":%u,\"adapter_device\":%u}\n", desc.VendorId, desc.DeviceId);
    for (UINT value = 1; value <= 115; ++value) {
        auto format = static_cast<DXGI_FORMAT>(value);
        D3D12_FEATURE_DATA_FORMAT_SUPPORT support{format,{}, {}};
        HRESULT hr = device->CheckFeatureSupport(D3D12_FEATURE_FORMAT_SUPPORT, &support, sizeof(support));
        std::printf("{\"format\":%u,\"native\":%u,\"hresult\":%u,\"support1\":%u,\"support2\":%u}\n",
            value, ffxApiGetSurfaceFormatDX12(format), static_cast<UINT>(hr), support.Support1, support.Support2);
    }
    return FAILED(device->GetDeviceRemovedReason()) ? 4 : 0;
}
