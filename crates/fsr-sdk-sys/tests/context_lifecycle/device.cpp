// SPDX-License-Identifier: MPL-2.0

// Experiment-only device plumbing. No AMD calls or custom COM ABI declarations.
#include <d3d12.h>
#include <dxgi1_4.h>
#include <cstdio>
#include <cstdint>
#include <type_traits>

extern "C" __declspec(dllexport) int32_t __cdecl probe_create_device(ID3D12Device** output)
{
    *output = nullptr;
    IDXGIFactory1* factory = nullptr;
    HRESULT result = CreateDXGIFactory1(IID_PPV_ARGS(&factory));
    if (FAILED(result)) return result;
    // First enumerated hardware adapter supporting feature level 12_0; no WARP.
    for (UINT index = 0; ; ++index) {
        IDXGIAdapter1* adapter = nullptr;
        result = factory->EnumAdapters1(index, &adapter);
        if (FAILED(result)) break;
        DXGI_ADAPTER_DESC1 desc{};
        result = adapter->GetDesc1(&desc);
        if (SUCCEEDED(result) && !(desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE)) {
            result = D3D12CreateDevice(adapter, D3D_FEATURE_LEVEL_12_0, IID_PPV_ARGS(output));
            if (SUCCEEDED(result)) {
                char name[512]{};
                WideCharToMultiByte(CP_UTF8, 0, desc.Description, -1, name, sizeof(name), nullptr, nullptr);
                std::fprintf(stderr, "Adapter: %s; vendor=0x%04x device=0x%04x; feature_level=12_0\n",
                             name, desc.VendorId, desc.DeviceId);
                IDXGIAdapter3* adapter3 = nullptr;
                if (SUCCEEDED(adapter->QueryInterface(IID_PPV_ARGS(&adapter3)))) {
                    DXGI_QUERY_VIDEO_MEMORY_INFO memory{};
                    if (SUCCEEDED(adapter3->QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, &memory))) {
                        std::fprintf(stderr, "Local GPU memory: dedicated_bytes=%llu budget_bytes=%llu current_usage_bytes=%llu\n",
                                     static_cast<unsigned long long>(desc.DedicatedVideoMemory),
                                     static_cast<unsigned long long>(memory.Budget),
                                     static_cast<unsigned long long>(memory.CurrentUsage));
                    }
                    adapter3->Release();
                }
                LARGE_INTEGER driver{};
                HRESULT driverResult = adapter->CheckInterfaceSupport(__uuidof(IDXGIDevice), &driver);
                std::fprintf(stderr, "Driver query: HRESULT=0x%08lx; version=%u.%u.%u.%u\n",
                             static_cast<unsigned long>(driverResult),
                             HIWORD(driver.HighPart), LOWORD(driver.HighPart),
                             HIWORD(driver.LowPart), LOWORD(driver.LowPart));
                adapter->Release();
                factory->Release();
                return result;
            }
        }
        adapter->Release();
    }
    factory->Release();
    return result;
}

extern "C" __declspec(dllexport) void __cdecl probe_release_device(ID3D12Device* device)
{
    device->Release();
}

static_assert(std::is_same_v<decltype(&probe_create_device), int32_t (__cdecl *)(ID3D12Device**)>);
static_assert(std::is_same_v<decltype(&probe_release_device), void (__cdecl *)(ID3D12Device*)>);

// Dedicated virtual allocation: protection never covers the Rust heap or stack.
extern "C" __declspec(dllexport) void* __cdecl probe_allocate_pages(size_t bytes)
{
    return VirtualAlloc(nullptr, bytes, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
}

extern "C" __declspec(dllexport) int32_t __cdecl probe_protect_pages(void* pages, size_t bytes)
{
    DWORD oldProtection = 0;
    if (!VirtualProtect(pages, bytes, PAGE_NOACCESS, &oldProtection)) return 0;
    MEMORY_BASIC_INFORMATION info{};
    return VirtualQuery(pages, &info, sizeof(info)) == sizeof(info)
        && info.Protect == PAGE_NOACCESS && info.RegionSize >= bytes;
}

extern "C" __declspec(dllexport) int32_t __cdecl probe_free_pages(void* pages)
{
    return VirtualFree(pages, 0, MEM_RELEASE) != 0;
}

static_assert(std::is_same_v<decltype(&probe_allocate_pages), void* (__cdecl *)(size_t)>);
static_assert(std::is_same_v<decltype(&probe_protect_pages), int32_t (__cdecl *)(void*, size_t)>);
static_assert(std::is_same_v<decltype(&probe_free_pages), int32_t (__cdecl *)(void*)>);
