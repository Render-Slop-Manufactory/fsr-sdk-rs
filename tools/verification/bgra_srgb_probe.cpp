// SPDX-License-Identifier: MPL-2.0
// Independent DX12 view probe; no AMD runtime is loaded or dispatched.
#define WIN32_LEAN_AND_MEAN
#define NOMINMAX
#include <windows.h>
#include <d3d12.h>
#include <d3d12sdklayers.h>
#include <dxgi1_6.h>
#include <d3dcompiler.h>
#include <wrl/client.h>
#include <dx12/ffx_api_dx12.h>
#include <array>
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <fstream>
#include <stdexcept>
#include <string>
#include <vector>

using Microsoft::WRL::ComPtr;
constexpr UINT side = 8;
constexpr UINT pixels = side * side;
struct Route { const char* name; DXGI_FORMAT allocation, srv, uav; bool bgra, srgb, casting; };
const Route routes[] = {
    {"rgba", DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM, false, false, false},
    {"bgra", DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_B8G8R8A8_UNORM, true, false, false},
    {"rgba_srgb", DXGI_FORMAT_R8G8B8A8_UNORM_SRGB, DXGI_FORMAT_R8G8B8A8_UNORM_SRGB, DXGI_FORMAT_R8G8B8A8_UNORM, false, true, true},
    {"bgra_srgb", DXGI_FORMAT_B8G8R8A8_UNORM_SRGB, DXGI_FORMAT_B8G8R8A8_UNORM_SRGB, DXGI_FORMAT_B8G8R8A8_UNORM, true, true, true},
    {"rgba_typeless", DXGI_FORMAT_R8G8B8A8_TYPELESS, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM, false, false, false},
    {"bgra_typeless", DXGI_FORMAT_B8G8R8A8_TYPELESS, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_B8G8R8A8_UNORM, true, false, false},
};

void check(HRESULT hr, const char* operation) {
    if (FAILED(hr)) {
        char message[160];
        std::snprintf(message, sizeof(message), "%s failed: 0x%08x", operation, static_cast<unsigned>(hr));
        throw std::runtime_error(message);
    }
}
void write_file(const std::string& name, const void* data, size_t bytes) {
    std::ofstream file(name, std::ios::binary);
    file.write(static_cast<const char*>(data), static_cast<std::streamsize>(bytes));
    if (!file) throw std::runtime_error("Readback file write failed");
}
D3D12_RESOURCE_DESC buffer_desc(UINT64 bytes, D3D12_RESOURCE_FLAGS flags = D3D12_RESOURCE_FLAG_NONE) {
    D3D12_RESOURCE_DESC d{};
    d.Dimension = D3D12_RESOURCE_DIMENSION_BUFFER; d.Width = bytes; d.Height = 1;
    d.DepthOrArraySize = 1; d.MipLevels = 1; d.SampleDesc.Count = 1;
    d.Layout = D3D12_TEXTURE_LAYOUT_ROW_MAJOR; d.Flags = flags;
    return d;
}
D3D12_RESOURCE_DESC texture_desc(DXGI_FORMAT format, D3D12_RESOURCE_FLAGS flags) {
    D3D12_RESOURCE_DESC d{};
    d.Dimension = D3D12_RESOURCE_DIMENSION_TEXTURE2D; d.Width = side; d.Height = side;
    d.DepthOrArraySize = 1; d.MipLevels = 1; d.SampleDesc.Count = 1;
    d.Format = format; d.Flags = flags;
    return d;
}

class Equipment {
public:
    ComPtr<ID3D12Device> device;
    ComPtr<ID3D12InfoQueue> messages;
    ComPtr<ID3D12CommandQueue> queue;
    ComPtr<ID3D12CommandAllocator> allocator;
    ComPtr<ID3D12GraphicsCommandList> list;
    ComPtr<ID3D12DescriptorHeap> heap;
    ComPtr<ID3D12RootSignature> root;
    ComPtr<ID3D12PipelineState> read_pipeline, write_pipeline;
    ComPtr<ID3D12Fence> fence;
    HANDLE event = nullptr;
    UINT64 sequence = 0;
    UINT descriptor_size = 0;
    bool typed_casting = false;

    explicit Equipment(UINT adapter_index) {
        ComPtr<ID3D12Debug> debug;
        check(D3D12GetDebugInterface(IID_PPV_ARGS(&debug)), "D3D12 debug interface");
        debug->EnableDebugLayer();
        ComPtr<IDXGIFactory4> factory;
        check(CreateDXGIFactory2(0, IID_PPV_ARGS(&factory)), "DXGI factory");
        ComPtr<IDXGIAdapter1> adapter;
        check(factory->EnumAdapters1(adapter_index, &adapter), "DXGI adapter selection");
        DXGI_ADAPTER_DESC1 desc{}; check(adapter->GetDesc1(&desc), "Adapter description");
        std::fprintf(stderr, "Adapter: %ls\n", desc.Description);
        if (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE) throw std::runtime_error("Software adapter is outside this probe");
        check(D3D12CreateDevice(adapter.Get(), D3D_FEATURE_LEVEL_12_0, IID_PPV_ARGS(&device)), "D3D12 device");
        check(device.As(&messages), "D3D12 info queue");
        LARGE_INTEGER driver{};
        HRESULT driver_hr = adapter->CheckInterfaceSupport(__uuidof(IDXGIDevice), &driver);
        std::printf("{\"kind\":\"equipment\",\"adapter_index\":%u,\"vendor\":%u,\"device\":%u,\"driver_hr\":%u,\"driver_high\":%u,\"driver_low\":%u}\n",
            adapter_index, desc.VendorId, desc.DeviceId, static_cast<UINT>(driver_hr), static_cast<UINT>(driver.HighPart), driver.LowPart);
        D3D12_FEATURE_DATA_D3D12_OPTIONS3 options{};
        check(device->CheckFeatureSupport(D3D12_FEATURE_D3D12_OPTIONS3, &options, sizeof(options)), "OPTIONS3");
        typed_casting = options.CastingFullyTypedFormatSupported != FALSE;
        std::printf("{\"kind\":\"casting\",\"supported\":%s}\n", typed_casting ? "true" : "false");
        D3D12_COMMAND_QUEUE_DESC q{};
        check(device->CreateCommandQueue(&q, IID_PPV_ARGS(&queue)), "Command queue");
        check(device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT, IID_PPV_ARGS(&allocator)), "Command allocator");
        check(device->CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, allocator.Get(), nullptr, IID_PPV_ARGS(&list)), "Command list");
        check(list->Close(), "Initial command-list close");
        D3D12_DESCRIPTOR_HEAP_DESC hd{};
        hd.Type = D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV; hd.NumDescriptors = 2;
        hd.Flags = D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE;
        check(device->CreateDescriptorHeap(&hd, IID_PPV_ARGS(&heap)), "Descriptor heap");
        descriptor_size = device->GetDescriptorHandleIncrementSize(hd.Type);
        D3D12_DESCRIPTOR_RANGE ranges[2]{};
        D3D12_ROOT_PARAMETER parameters[2]{};
        for (UINT i = 0; i < 2; ++i) {
            ranges[i].RangeType = i == 0 ? D3D12_DESCRIPTOR_RANGE_TYPE_SRV : D3D12_DESCRIPTOR_RANGE_TYPE_UAV;
            ranges[i].NumDescriptors = 1;
            parameters[i].ParameterType = D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE;
            parameters[i].DescriptorTable.NumDescriptorRanges = 1;
            parameters[i].DescriptorTable.pDescriptorRanges = &ranges[i];
        }
        D3D12_STATIC_SAMPLER_DESC sampler{};
        sampler.Filter = D3D12_FILTER_MIN_MAG_MIP_POINT;
        sampler.AddressU = sampler.AddressV = sampler.AddressW = D3D12_TEXTURE_ADDRESS_MODE_CLAMP;
        sampler.MaxLOD = D3D12_FLOAT32_MAX; sampler.ComparisonFunc = D3D12_COMPARISON_FUNC_ALWAYS;
        D3D12_ROOT_SIGNATURE_DESC rd{};
        rd.NumParameters = 2; rd.pParameters = parameters;
        rd.NumStaticSamplers = 1; rd.pStaticSamplers = &sampler;
        ComPtr<ID3DBlob> blob, errors;
        check(D3D12SerializeRootSignature(&rd, D3D_ROOT_SIGNATURE_VERSION_1, &blob, &errors), "Root signature serialisation");
        check(device->CreateRootSignature(0, blob->GetBufferPointer(), blob->GetBufferSize(), IID_PPV_ARGS(&root)), "Root signature");
        read_pipeline = pipeline(
            "Texture2D<float4> source:register(t0); SamplerState pointSampler:register(s0);"
            "RWStructuredBuffer<float4> output:register(u0);"
            "[numthreads(8,8,1)] void main(uint3 p:SV_DispatchThreadID) {"
            "uint i=p.y*8+p.x; output[i]=source.Load(int3(p.xy,0));"
            "output[64+i]=source.SampleLevel(pointSampler,(float2(p.xy)+0.5)/8.0,0); }");
        write_pipeline = pipeline(
            "RWTexture2D<float4> target:register(u0);"
            "[numthreads(8,8,1)] void main(uint3 p:SV_DispatchThreadID) {"
            "target[p.xy]=float4(48.0/255.0,128.0/255.0,224.0/255.0,160.0/255.0); }");
        check(device->CreateFence(0, D3D12_FENCE_FLAG_NONE, IID_PPV_ARGS(&fence)), "Fence");
        event = CreateEventW(nullptr, FALSE, FALSE, nullptr);
        if (!event) throw std::runtime_error("Fence event creation failed");
        clean();
    }
    ~Equipment() { if (event) CloseHandle(event); }
    ComPtr<ID3D12PipelineState> pipeline(const char* source) {
        ComPtr<ID3DBlob> shader, errors;
        auto hr = D3DCompile(source, std::strlen(source), "independent-view-probe", nullptr, nullptr, "main", "cs_5_0",
            D3DCOMPILE_ENABLE_STRICTNESS | D3DCOMPILE_WARNINGS_ARE_ERRORS, 0, &shader, &errors);
        if (FAILED(hr) && errors) std::fprintf(stderr, "%s", static_cast<const char*>(errors->GetBufferPointer()));
        check(hr, "Independent shader compilation");
        D3D12_COMPUTE_PIPELINE_STATE_DESC d{}; d.pRootSignature = root.Get();
        d.CS = {shader->GetBufferPointer(), shader->GetBufferSize()};
        ComPtr<ID3D12PipelineState> result;
        check(device->CreateComputePipelineState(&d, IID_PPV_ARGS(&result)), "Compute pipeline");
        return result;
    }
    D3D12_CPU_DESCRIPTOR_HANDLE cpu(UINT i) {
        auto h = heap->GetCPUDescriptorHandleForHeapStart(); h.ptr += i * descriptor_size; return h;
    }
    void clean() {
        bool bad = false;
        for (UINT64 i = 0; i < messages->GetNumStoredMessages(); ++i) {
            SIZE_T size = 0; check(messages->GetMessage(i, nullptr, &size), "Debug-message size");
            std::vector<unsigned char> bytes(size);
            auto* m = reinterpret_cast<D3D12_MESSAGE*>(bytes.data());
            check(messages->GetMessage(i, m, &size), "Debug message");
            if (m->Severity <= D3D12_MESSAGE_SEVERITY_WARNING) {
                std::fprintf(stderr, "DX12 severity=%u id=%u: %s\n", m->Severity, m->ID, m->pDescription);
                bad = bad || m->Severity <= D3D12_MESSAGE_SEVERITY_ERROR;
            }
        }
        messages->ClearStoredMessages();
        if (bad) throw std::runtime_error("Unexpected DX12 error");
        check(device->GetDeviceRemovedReason(), "Device removal status");
    }
    D3D12_FEATURE_DATA_FORMAT_SUPPORT support(DXGI_FORMAT format) {
        D3D12_FEATURE_DATA_FORMAT_SUPPORT s{format};
        check(device->CheckFeatureSupport(D3D12_FEATURE_FORMAT_SUPPORT, &s, sizeof(s)), "Format support");
        return s;
    }
    ComPtr<ID3D12Resource> resource(D3D12_RESOURCE_DESC d, D3D12_HEAP_TYPE type, D3D12_RESOURCE_STATES state) {
        D3D12_HEAP_PROPERTIES hp{}; hp.Type = type;
        ComPtr<ID3D12Resource> r;
        check(device->CreateCommittedResource(&hp, D3D12_HEAP_FLAG_NONE, &d, state, nullptr, IID_PPV_ARGS(&r)), "Resource creation");
        return r;
    }
    void begin() {
        check(allocator->Reset(), "Allocator reset"); check(list->Reset(allocator.Get(), nullptr), "Command-list reset");
        list->SetComputeRootSignature(root.Get());
        ID3D12DescriptorHeap* heaps[] = {heap.Get()}; list->SetDescriptorHeaps(1, heaps);
        auto h = heap->GetGPUDescriptorHandleForHeapStart(); list->SetComputeRootDescriptorTable(0, h);
        h.ptr += descriptor_size; list->SetComputeRootDescriptorTable(1, h);
    }
    void finish() {
        check(list->Close(), "Command-list close"); clean();
        ID3D12CommandList* lists[] = {list.Get()}; queue->ExecuteCommandLists(1, lists);
        check(queue->Signal(fence.Get(), ++sequence), "Queue signal");
        check(fence->SetEventOnCompletion(sequence, event), "Fence event");
        if (WaitForSingleObject(event, 15000) != WAIT_OBJECT_0) throw std::runtime_error("GPU fence timeout");
        clean();
    }
    void transition(ID3D12Resource* r, D3D12_RESOURCE_STATES before, D3D12_RESOURCE_STATES after) {
        D3D12_RESOURCE_BARRIER b{}; b.Type = D3D12_RESOURCE_BARRIER_TYPE_TRANSITION;
        b.Transition = {r, D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES, before, after}; list->ResourceBarrier(1, &b);
    }
    void srv(ID3D12Resource* r, DXGI_FORMAT format) {
        D3D12_SHADER_RESOURCE_VIEW_DESC d{}; d.Format = format;
        d.ViewDimension = D3D12_SRV_DIMENSION_TEXTURE2D; d.Shader4ComponentMapping = D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING;
        d.Texture2D.MipLevels = 1; device->CreateShaderResourceView(r, &d, cpu(0)); clean();
    }
    void read_texture(ID3D12Resource* texture, const Route& route, const std::string& name) {
        srv(texture, route.srv);
        constexpr UINT bytes = pixels * 2 * 4 * sizeof(float);
        auto output = resource(buffer_desc(bytes, D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS), D3D12_HEAP_TYPE_DEFAULT, D3D12_RESOURCE_STATE_COPY_DEST);
        auto initial = resource(buffer_desc(bytes), D3D12_HEAP_TYPE_UPLOAD, D3D12_RESOURCE_STATE_GENERIC_READ);
        auto readback = resource(buffer_desc(bytes), D3D12_HEAP_TYPE_READBACK, D3D12_RESOURCE_STATE_COPY_DEST);
        void* p = nullptr; check(initial->Map(0, nullptr, &p), "Sentinel upload map");
        std::memset(p, 0xa5, bytes); initial->Unmap(0, nullptr);
        D3D12_UNORDERED_ACCESS_VIEW_DESC u{};
        u.ViewDimension = D3D12_UAV_DIMENSION_BUFFER; u.Buffer.NumElements = pixels * 2; u.Buffer.StructureByteStride = 4 * sizeof(float);
        device->CreateUnorderedAccessView(output.Get(), nullptr, &u, cpu(1)); clean();
        begin(); list->CopyBufferRegion(output.Get(), 0, initial.Get(), 0, bytes);
        transition(output.Get(), D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_UNORDERED_ACCESS);
        list->SetPipelineState(read_pipeline.Get()); list->Dispatch(1, 1, 1);
        transition(output.Get(), D3D12_RESOURCE_STATE_UNORDERED_ACCESS, D3D12_RESOURCE_STATE_COPY_SOURCE);
        list->CopyBufferRegion(readback.Get(), 0, output.Get(), 0, bytes); finish();
        check(readback->Map(0, nullptr, &p), "Float readback map"); write_file(name, p, bytes); readback->Unmap(0, nullptr);
    }
    void input(const Route& route, bool swapped = false) {
        auto s = support(route.srv);
        constexpr UINT required = D3D12_FORMAT_SUPPORT1_TEXTURE2D | D3D12_FORMAT_SUPPORT1_SHADER_LOAD | D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE;
        if ((s.Support1 & required) != required) throw std::runtime_error("Required input SRV support absent");
        auto d = texture_desc(route.allocation, D3D12_RESOURCE_FLAG_NONE);
        auto texture = resource(d, D3D12_HEAP_TYPE_DEFAULT, D3D12_RESOURCE_STATE_COPY_DEST);
        D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint{}; UINT64 bytes = 0;
        device->GetCopyableFootprints(&d, 0, 1, 0, &footprint, nullptr, nullptr, &bytes);
        auto upload = resource(buffer_desc(bytes), D3D12_HEAP_TYPE_UPLOAD, D3D12_RESOURCE_STATE_GENERIC_READ);
        unsigned char* p = nullptr; check(upload->Map(0, nullptr, reinterpret_cast<void**>(&p)), "Colour upload map");
        std::memset(p, 0xcc, static_cast<size_t>(bytes));
        const unsigned char colors[2][4] = {{48,128,224,160}, {192,64,16,127}};
        for (UINT y = 0; y < side; ++y) for (UINT x = 0; x < side; ++x) {
            const auto* c = colors[(x + y) & 1];
            auto* dst = p + footprint.Offset + y * footprint.Footprint.RowPitch + x * 4;
            bool reverse = route.bgra != swapped;
            dst[0] = c[reverse ? 2 : 0]; dst[1] = c[1]; dst[2] = c[reverse ? 0 : 2]; dst[3] = c[3];
        }
        write_file(std::string(route.name) + (swapped ? "_swapped.upload" : ".upload"), p, static_cast<size_t>(bytes));
        upload->Unmap(0, nullptr);
        D3D12_TEXTURE_COPY_LOCATION dst{}; dst.pResource = texture.Get(); dst.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
        D3D12_TEXTURE_COPY_LOCATION src{}; src.pResource = upload.Get(); src.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT; src.PlacedFootprint = footprint;
        begin(); list->CopyTextureRegion(&dst, 0, 0, 0, &src, nullptr);
        transition(texture.Get(), D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE); finish();
        const auto name = std::string(route.name) + (swapped ? "_swapped.floats" : ".floats");
        read_texture(texture.Get(), route, name);
        const auto native = ffxApiGetResourceDX12(texture.Get());
        std::printf("{\"kind\":\"input\",\"name\":\"%s\",\"swapped\":%s,\"allocation\":%u,\"srv\":%u,\"native_format\":%u,\"native_usage\":%u,\"row_pitch\":%u,\"support1\":%u,\"support2\":%u}\n",
            route.name, swapped ? "true" : "false", route.allocation, route.srv, native.description.format, native.description.usage,
            footprint.Footprint.RowPitch, s.Support1, s.Support2);
    }
    void output(const Route& route) {
        auto s = support(route.uav);
        constexpr UINT required = D3D12_FORMAT_SUPPORT1_TEXTURE2D | D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW;
        bool capable = (s.Support1 & required) == required && (s.Support2 & D3D12_FORMAT_SUPPORT2_UAV_TYPED_STORE) != 0;
        if (!capable || (route.casting && !typed_casting)) {
            std::printf("{\"kind\":\"output\",\"name\":\"%s\",\"status\":\"unsupported\",\"support1\":%u,\"support2\":%u}\n", route.name, s.Support1, s.Support2);
            return;
        }
        auto d = texture_desc(route.allocation, D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS);
        D3D12_HEAP_PROPERTIES hp{}; hp.Type = D3D12_HEAP_TYPE_DEFAULT;
        ComPtr<ID3D12Resource> texture;
        HRESULT hr = device->CreateCommittedResource(&hp, D3D12_HEAP_FLAG_NONE, &d, D3D12_RESOURCE_STATE_UNORDERED_ACCESS, nullptr, IID_PPV_ARGS(&texture));
        if (FAILED(hr)) {
            // Preserve allocation failure and diagnostics separately from view support.
            std::printf("{\"kind\":\"output\",\"name\":\"%s\",\"status\":\"allocation_rejected\",\"hresult\":%u,\"support1\":%u,\"support2\":%u}\n", route.name, static_cast<UINT>(hr), s.Support1, s.Support2);
            for (UINT64 i = 0; i < messages->GetNumStoredMessages(); ++i) {
                SIZE_T n = 0; check(messages->GetMessage(i, nullptr, &n), "Allocation-message size");
                std::vector<unsigned char> b(n); auto* m = reinterpret_cast<D3D12_MESSAGE*>(b.data());
                check(messages->GetMessage(i, m, &n), "Allocation message");
                std::fprintf(stderr, "Allocation rejected %s id=%u: %s\n", route.name, m->ID, m->pDescription);
            }
            messages->ClearStoredMessages(); check(device->GetDeviceRemovedReason(), "Rejected allocation device status"); return;
        }
        srv(texture.Get(), route.srv);
        D3D12_UNORDERED_ACCESS_VIEW_DESC u{}; u.Format = route.uav; u.ViewDimension = D3D12_UAV_DIMENSION_TEXTURE2D;
        device->CreateUnorderedAccessView(texture.Get(), nullptr, &u, cpu(1)); clean();
        D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprint{}; UINT64 bytes = 0;
        device->GetCopyableFootprints(&d, 0, 1, 0, &footprint, nullptr, nullptr, &bytes);
        auto readback = resource(buffer_desc(bytes), D3D12_HEAP_TYPE_READBACK, D3D12_RESOURCE_STATE_COPY_DEST);
        D3D12_TEXTURE_COPY_LOCATION src{}; src.pResource = texture.Get(); src.Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
        D3D12_TEXTURE_COPY_LOCATION dst{}; dst.pResource = readback.Get(); dst.Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT; dst.PlacedFootprint = footprint;
        begin(); list->SetPipelineState(write_pipeline.Get()); list->Dispatch(1, 1, 1);
        transition(texture.Get(), D3D12_RESOURCE_STATE_UNORDERED_ACCESS, D3D12_RESOURCE_STATE_COPY_SOURCE);
        list->CopyTextureRegion(&dst, 0, 0, 0, &src, nullptr);
        transition(texture.Get(), D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE); finish();
        void* p = nullptr; check(readback->Map(0, nullptr, &p), "Output byte readback map");
        // Retain only real texels; padding is not a shader-storage observation.
        std::array<unsigned char, pixels * 4> packed{};
        for (UINT y = 0; y < side; ++y) std::memcpy(packed.data() + y * side * 4,
            static_cast<unsigned char*>(p) + footprint.Offset + y * footprint.Footprint.RowPitch, side * 4);
        write_file(std::string(route.name) + ".stores", packed.data(), packed.size()); readback->Unmap(0, nullptr);
        read_texture(texture.Get(), route, std::string(route.name) + ".stored_floats");
        std::printf("{\"kind\":\"output\",\"name\":\"%s\",\"status\":\"written\",\"allocation\":%u,\"srv\":%u,\"uav\":%u,\"row_pitch\":%u,\"support1\":%u,\"support2\":%u}\n",
            route.name, route.allocation, route.srv, route.uav, footprint.Footprint.RowPitch, s.Support1, s.Support2);
    }
};

int main(int argc, char** argv) {
    try {
        if (argc != 2) throw std::runtime_error("Supply the explicit DXGI adapter index");
        const auto index = std::stoul(argv[1]);
        if (index > 32) throw std::runtime_error("Adapter index outside probe range");
        Equipment equipment(static_cast<UINT>(index));
        for (const auto& route : routes) equipment.input(route);
        equipment.input(routes[1], true);
        for (const auto& route : routes) equipment.output(route);
        equipment.clean();
        std::puts("{\"kind\":\"finished\",\"fenced\":true,\"unexpected_debug_errors\":0}");
        return 0;
    } catch (const std::exception& error) {
        std::fprintf(stderr, "%s\n", error.what()); return 1;
    }
}
