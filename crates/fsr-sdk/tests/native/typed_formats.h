// SPDX-License-Identifier: MPL-2.0

// DX12-only typed texture equipment; the coordinator supplies tight input bytes.
constexpr uint32_t format_roles = 7;
struct FormatHarness {
    Harness base;
    ComPtr<ID3D12Resource> textures[format_roles], uploads[format_roles], snapshots[format_roles];
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT footprints[format_roles]{};
    uint32_t row_bytes[format_roles]{}, heights[format_roles]{};
    ComPtr<ID3D12DescriptorHeap> depth_dsv,depth_srv;
    ComPtr<ID3D12RootSignature> depth_root;
    ComPtr<ID3D12PipelineState> depth_pipeline;
    ComPtr<ID3D12Resource> depth_sample,depth_sample_readback,stencil_readback;
    D3D12_PLACED_SUBRESOURCE_FOOTPRINT stencil_footprint{};
};
#include "depth_formats.h"
static uint32_t format_stride(DXGI_FORMAT format) {
    switch (format) {
    case DXGI_FORMAT_R32G32B32A32_TYPELESS: case DXGI_FORMAT_R32G32B32A32_FLOAT: return 16;
    case DXGI_FORMAT_R32G32B32_TYPELESS: case DXGI_FORMAT_R32G32B32_FLOAT: return 12;
    case DXGI_FORMAT_R32G32_TYPELESS: case DXGI_FORMAT_R32G32_FLOAT:
    case DXGI_FORMAT_R16G16B16A16_TYPELESS: case DXGI_FORMAT_R16G16B16A16_FLOAT: return 8;
    case DXGI_FORMAT_R16_TYPELESS: case DXGI_FORMAT_R16_UNORM:
    case DXGI_FORMAT_R16_FLOAT: return 2;
    case DXGI_FORMAT_R8_TYPELESS: case DXGI_FORMAT_R8_UNORM: return 1;
    // Depth payloads are float32 shader-depth expectations, independently of
    // the physical UNORM depth-plane copy footprint.
    case DXGI_FORMAT_D16_UNORM: case DXGI_FORMAT_R24G8_TYPELESS:
    case DXGI_FORMAT_D24_UNORM_S8_UINT: return 4;
    case DXGI_FORMAT_R32_TYPELESS: case DXGI_FORMAT_D32_FLOAT:
    case DXGI_FORMAT_R32G8X24_TYPELESS: case DXGI_FORMAT_D32_FLOAT_S8X24_UINT:
    case DXGI_FORMAT_R32_FLOAT: case DXGI_FORMAT_R16G16_TYPELESS: case DXGI_FORMAT_R16G16_FLOAT:
    case DXGI_FORMAT_R10G10B10A2_TYPELESS: case DXGI_FORMAT_R8G8B8A8_TYPELESS:
    case DXGI_FORMAT_R8G8B8A8_SNORM: case DXGI_FORMAT_R9G9B9E5_SHAREDEXP:
    case DXGI_FORMAT_B8G8R8A8_TYPELESS:
    case DXGI_FORMAT_R11G11B10_FLOAT: case DXGI_FORMAT_R10G10B10A2_UNORM:
    case DXGI_FORMAT_R8G8B8A8_UNORM: case DXGI_FORMAT_B8G8R8A8_UNORM:
    case DXGI_FORMAT_R8G8B8A8_UNORM_SRGB: case DXGI_FORMAT_B8G8R8A8_UNORM_SRGB: return 4;
    default: return 0;
    }
}
extern "C" __declspec(dllexport) int32_t __cdecl gpu_formats_create(
    ID3D12Device* device, const uint32_t* formats, uint32_t uav_inputs, uint32_t depth_buffer,
    const uint8_t* const* inputs, const uint32_t* counts, void** handle,
    ID3D12GraphicsCommandList** list, ID3D12Resource** resources) {
    if (!device || !formats || !inputs || !counts || !handle || !list || !resources) return E_INVALIDARG;
    *handle = nullptr; *list = nullptr;
    auto o = std::make_unique<FormatHarness>(); auto& h = o->base; h.device = device;
    D3D12_COMMAND_QUEUE_DESC queue{}; queue.Type = D3D12_COMMAND_LIST_TYPE_DIRECT;
    HRESULT hr = device->CreateCommandQueue(&queue, IID_PPV_ARGS(&h.queue));
    if (FAILED(hr)) return hr;
    hr = device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT, IID_PPV_ARGS(&h.allocator));
    if (FAILED(hr)) return hr;
    hr = device->CreateCommandList(0,D3D12_COMMAND_LIST_TYPE_DIRECT,h.allocator.Get(),nullptr,IID_PPV_ARGS(&h.list));
    if (FAILED(hr)) return hr;
    hr = device->CreateFence(0,D3D12_FENCE_FLAG_NONE,IID_PPV_ARGS(&h.fence));
    if (FAILED(hr)) return hr;
    for (uint32_t i=0;i<format_roles;++i) {
        auto d = texture_desc(i < Count ? static_cast<Role>(i) : Color);
        d.Format = static_cast<DXGI_FORMAT>(formats[i]);
        d.Flags = (i==4 || i==6 || (uav_inputs & (1u<<i)))
            ? D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS : D3D12_RESOURCE_FLAG_NONE;
        const bool depth=i==1 && depth_buffer && depth_allocation(d.Format);
        if(depth)d.Flags=D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL;
        const auto stride = format_stride(d.Format);
        if (!stride || counts[i] != d.Width*d.Height*stride || !inputs[i]) return E_INVALIDARG;
        D3D12_FEATURE_DATA_FORMAT_SUPPORT support{d.Format,{},{}};
        hr = device->CheckFeatureSupport(D3D12_FEATURE_FORMAT_SUPPORT,&support,sizeof(support));
        if (FAILED(hr)) return hr;
        std::fprintf(stderr,"FORMAT_CAP {\"role\":%u,\"format\":%u,\"support1\":%u,\"support2\":%u}\n",
            i,formats[i],static_cast<uint32_t>(support.Support1),static_cast<uint32_t>(support.Support2));
        auto heap = heap_properties(D3D12_HEAP_TYPE_DEFAULT);
        hr=device->CreateCommittedResource(&heap,D3D12_HEAP_FLAG_NONE,&d,depth?D3D12_RESOURCE_STATE_DEPTH_WRITE:D3D12_RESOURCE_STATE_COPY_DEST,nullptr,IID_PPV_ARGS(&o->textures[i]));
        if (FAILED(hr)) return hr;
        UINT64 bytes=0;
        device->GetCopyableFootprints(&d,0,1,0,&o->footprints[i],nullptr,nullptr,&bytes);
        const auto b=buffer_desc(bytes);
        heap=heap_properties(D3D12_HEAP_TYPE_UPLOAD);
        hr=device->CreateCommittedResource(&heap,D3D12_HEAP_FLAG_NONE,&b,D3D12_RESOURCE_STATE_GENERIC_READ,nullptr,IID_PPV_ARGS(&o->uploads[i]));
        if (FAILED(hr)) return hr;
        heap=heap_properties(D3D12_HEAP_TYPE_READBACK);
        hr=device->CreateCommittedResource(&heap,D3D12_HEAP_FLAG_NONE,&b,D3D12_RESOURCE_STATE_COPY_DEST,nullptr,IID_PPV_ARGS(&o->snapshots[i]));
        if (FAILED(hr)) return hr;
        o->row_bytes[i]=static_cast<uint32_t>(d.Width)*stride; o->heights[i]=d.Height;
        if(depth) {
            hr=prepare_depth(*o,d,inputs[i]);if(FAILED(hr))return hr;
            continue;
        }
        uint8_t* mapped=nullptr;
        hr=o->uploads[i]->Map(0,nullptr,reinterpret_cast<void**>(&mapped));
        if (FAILED(hr)) return hr;
        std::memset(mapped,0,static_cast<size_t>(bytes));
        for (uint32_t y=0;y<d.Height;++y)
            std::memcpy(mapped+o->footprints[i].Offset+y*o->footprints[i].Footprint.RowPitch,
                inputs[i]+y*o->row_bytes[i],o->row_bytes[i]);
        o->uploads[i]->Unmap(0,nullptr);
        D3D12_TEXTURE_COPY_LOCATION from{},to{};
        from.pResource=o->uploads[i].Get();from.Type=D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;from.PlacedFootprint=o->footprints[i];
        to.pResource=o->textures[i].Get();to.Type=D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
        h.list->CopyTextureRegion(&to,0,0,0,&from,nullptr);
        transition(h.list.Get(),o->textures[i].Get(),D3D12_RESOURCE_STATE_COPY_DEST,
            i==4 || i==6 ? D3D12_RESOURCE_STATE_UNORDERED_ACCESS : D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE);
    }
    *list=h.list.Get();(*list)->AddRef();
    for (uint32_t i=0;i<format_roles;++i) {resources[i]=o->textures[i].Get();resources[i]->AddRef();}
    *handle=o.release();return S_OK;
}
extern "C" __declspec(dllexport) void __cdecl gpu_formats_consume_mask(void* handle) {
    auto& o=*static_cast<FormatHarness*>(handle);
    D3D12_RESOURCE_BARRIER barrier{};barrier.Type=D3D12_RESOURCE_BARRIER_TYPE_UAV;barrier.UAV.pResource=o.textures[6].Get();
    o.base.list->ResourceBarrier(1,&barrier);
    transition(o.base.list.Get(),o.textures[6].Get(),D3D12_RESOURCE_STATE_UNORDERED_ACCESS,D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE);
}
extern "C" __declspec(dllexport) int32_t __cdecl gpu_formats_finish(void* handle,uint32_t consumed,uint64_t* errors,int32_t* removed) {
    if (!handle || !errors || !removed) return E_INVALIDARG;
    auto& o=*static_cast<FormatHarness*>(handle);auto& h=o.base;
    if(o.stencil_readback) {
        transition(h.list.Get(),o.textures[1].Get(),D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,D3D12_RESOURCE_STATE_COPY_SOURCE);
        D3D12_TEXTURE_COPY_LOCATION from{},to{};
        from.pResource=o.textures[1].Get();from.Type=D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;from.SubresourceIndex=1;
        to.pResource=o.stencil_readback.Get();to.Type=D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;to.PlacedFootprint=o.stencil_footprint;
        h.list->CopyTextureRegion(&to,0,0,0,&from,nullptr);
        transition(h.list.Get(),o.textures[1].Get(),D3D12_RESOURCE_STATE_COPY_SOURCE,D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE);
    }
    for (uint32_t i=0;i<format_roles;++i) {
        const auto state=i==4 || (i==6 && !consumed) ? D3D12_RESOURCE_STATE_UNORDERED_ACCESS : D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE;
        transition(h.list.Get(),o.textures[i].Get(),state,D3D12_RESOURCE_STATE_COPY_SOURCE);
        D3D12_TEXTURE_COPY_LOCATION from{},to{};
        from.pResource=o.textures[i].Get();from.Type=D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
        to.pResource=o.snapshots[i].Get();to.Type=D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;to.PlacedFootprint=o.footprints[i];
        h.list->CopyTextureRegion(&to,0,0,0,&from,nullptr);
        transition(h.list.Get(),o.textures[i].Get(),D3D12_RESOURCE_STATE_COPY_SOURCE,state);
    }
    HRESULT hr=h.list->Close();if (FAILED(hr)) return hr;
    ID3D12CommandList* lists[]={h.list.Get()};h.queue->ExecuteCommandLists(1,lists);
    hr=h.queue->Signal(h.fence.Get(),1);if (FAILED(hr)) return hr;
    hr=wait_for_fence(h);if (FAILED(hr)) return hr;
    *removed=h.device->GetDeviceRemovedReason();*errors=debug_error_count(h.device.Get());
    return h.fence->GetCompletedValue()==1 && *removed==S_OK ? S_OK : E_FAIL;
}
extern "C" __declspec(dllexport) int32_t __cdecl gpu_formats_copy(void* handle,uint32_t role,uint8_t* bytes,uint32_t count) {
    if (!handle || role>=format_roles || !bytes) return E_INVALIDARG;
    auto& o=*static_cast<FormatHarness*>(handle);
    if (count!=o.row_bytes[role]*o.heights[role] || o.base.fence->GetCompletedValue()!=1) return E_INVALIDARG;
    uint8_t* mapped=nullptr;const HRESULT hr=o.snapshots[role]->Map(0,nullptr,reinterpret_cast<void**>(&mapped));
    if (FAILED(hr)) return hr;
    const auto format=o.textures[role]->GetDesc().Format;
    const bool d16=role==1 && format==DXGI_FORMAT_D16_UNORM;
    const bool d24=role==1 && (format==DXGI_FORMAT_R24G8_TYPELESS || format==DXGI_FORMAT_D24_UNORM_S8_UINT);
    for (uint32_t y=0;y<o.heights[role];++y) {
        const auto* row=mapped+o.footprints[role].Offset+y*o.footprints[role].Footprint.RowPitch;
        if(d16 || d24) {
            for(uint32_t x=0;x<320;++x) {
                uint32_t code=0;std::memcpy(&code,row+x*(d16?2:4),d16?2:4);
                const float depth=static_cast<float>(code&(d16?0xffffu:0xffffffu))/(d16?65535.0f:16777215.0f);
                std::memcpy(bytes+y*o.row_bytes[role]+x*4,&depth,4);
            }
        } else std::memcpy(bytes+y*o.row_bytes[role],row,o.row_bytes[role]);
    }
    o.snapshots[role]->Unmap(0,nullptr);return S_OK;
}
extern "C" __declspec(dllexport) void __cdecl gpu_formats_discard(void* handle) {static_cast<FormatHarness*>(handle)->base.list.Reset();}
extern "C" __declspec(dllexport) void __cdecl gpu_formats_destroy(void* handle) {delete static_cast<FormatHarness*>(handle);}
