// SPDX-License-Identifier: MPL-2.0

// Independent DSV production and shader readback of the backend's depth SRV.
static bool depth_allocation(DXGI_FORMAT f) {
    return f==DXGI_FORMAT_R32_TYPELESS || f==DXGI_FORMAT_D32_FLOAT ||
        f==DXGI_FORMAT_R32G8X24_TYPELESS || f==DXGI_FORMAT_D32_FLOAT_S8X24_UINT ||
        f==DXGI_FORMAT_D16_UNORM || f==DXGI_FORMAT_R24G8_TYPELESS || f==DXGI_FORMAT_D24_UNORM_S8_UINT;
}
static bool stencil_allocation(DXGI_FORMAT f) {
    return f==DXGI_FORMAT_R32G8X24_TYPELESS || f==DXGI_FORMAT_D32_FLOAT_S8X24_UINT ||
        f==DXGI_FORMAT_R24G8_TYPELESS || f==DXGI_FORMAT_D24_UNORM_S8_UINT;
}
static HRESULT prepare_depth(FormatHarness& o, const D3D12_RESOURCE_DESC& d, const uint8_t* input) {
    auto& h=o.base;
    const bool stencil=stencil_allocation(d.Format);
    const bool d16=d.Format==DXGI_FORMAT_D16_UNORM;
    const bool d24=d.Format==DXGI_FORMAT_R24G8_TYPELESS || d.Format==DXGI_FORMAT_D24_UNORM_S8_UINT;
    const auto dsv_format=d16 ? DXGI_FORMAT_D16_UNORM : d24 ? DXGI_FORMAT_D24_UNORM_S8_UINT :
        stencil ? DXGI_FORMAT_D32_FLOAT_S8X24_UINT : DXGI_FORMAT_D32_FLOAT;
    const auto srv_format=d16 ? DXGI_FORMAT_R16_UNORM : d24 ? DXGI_FORMAT_R24_UNORM_X8_TYPELESS :
        stencil ? DXGI_FORMAT_R32_FLOAT_X8X24_TYPELESS : DXGI_FORMAT_R32_FLOAT;
    D3D12_FEATURE_DATA_D3D12_OPTIONS3 options{};
    HRESULT hr=h.device->CheckFeatureSupport(D3D12_FEATURE_D3D12_OPTIONS3,&options,sizeof(options));
    if(FAILED(hr))return hr;
    if((d.Format==DXGI_FORMAT_D32_FLOAT || d.Format==DXGI_FORMAT_D32_FLOAT_S8X24_UINT ||
        d.Format==DXGI_FORMAT_D16_UNORM || d.Format==DXGI_FORMAT_D24_UNORM_S8_UINT) && !options.CastingFullyTypedFormatSupported)return E_FAIL;
    for(auto f : {dsv_format,srv_format}) {
        D3D12_FEATURE_DATA_FORMAT_SUPPORT support{f,{},{}};
        hr=h.device->CheckFeatureSupport(D3D12_FEATURE_FORMAT_SUPPORT,&support,sizeof(support));
        if(FAILED(hr))return hr;
        const auto required=D3D12_FORMAT_SUPPORT1_TEXTURE2D | (f==dsv_format ? D3D12_FORMAT_SUPPORT1_DEPTH_STENCIL :
            D3D12_FORMAT_SUPPORT1_SHADER_LOAD | D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE);
        if((support.Support1&required)!=required)return E_FAIL;
    }
    D3D12_DESCRIPTOR_HEAP_DESC heap{};heap.Type=D3D12_DESCRIPTOR_HEAP_TYPE_DSV;heap.NumDescriptors=1;
    hr=h.device->CreateDescriptorHeap(&heap,IID_PPV_ARGS(&o.depth_dsv));if(FAILED(hr))return hr;
    D3D12_DEPTH_STENCIL_VIEW_DESC dsv{};dsv.Format=dsv_format;dsv.ViewDimension=D3D12_DSV_DIMENSION_TEXTURE2D;
    const auto handle=o.depth_dsv->GetCPUDescriptorHandleForHeapStart();
    h.device->CreateDepthStencilView(o.textures[1].Get(),&dsv,handle);
    float left=0,right=0;std::memcpy(&left,input,4);std::memcpy(&right,input+160*4,4);
    h.list->ClearDepthStencilView(handle,static_cast<D3D12_CLEAR_FLAGS>(D3D12_CLEAR_FLAG_DEPTH | (stencil?D3D12_CLEAR_FLAG_STENCIL:0)),left,0x5a,0,nullptr);
    D3D12_RECT rect{160,0,320,180};
    h.list->ClearDepthStencilView(handle,D3D12_CLEAR_FLAG_DEPTH,right,0,1,&rect);
    transition(h.list.Get(),o.textures[1].Get(),D3D12_RESOURCE_STATE_DEPTH_WRITE,D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE);

    heap.Type=D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV;heap.Flags=D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE;
    hr=h.device->CreateDescriptorHeap(&heap,IID_PPV_ARGS(&o.depth_srv));if(FAILED(hr))return hr;
    D3D12_SHADER_RESOURCE_VIEW_DESC srv{};srv.Format=srv_format;srv.ViewDimension=D3D12_SRV_DIMENSION_TEXTURE2D;
    srv.Shader4ComponentMapping=D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING;srv.Texture2D.MipLevels=1;srv.Texture2D.PlaneSlice=0;
    h.device->CreateShaderResourceView(o.textures[1].Get(),&srv,o.depth_srv->GetCPUDescriptorHandleForHeapStart());
    D3D12_DESCRIPTOR_RANGE range{};range.RangeType=D3D12_DESCRIPTOR_RANGE_TYPE_SRV;range.NumDescriptors=1;
    D3D12_ROOT_PARAMETER parameters[2]{};
    parameters[0].ParameterType=D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE;
    parameters[0].DescriptorTable={1,&range};
    parameters[1].ParameterType=D3D12_ROOT_PARAMETER_TYPE_UAV;
    D3D12_ROOT_SIGNATURE_DESC root{};root.NumParameters=2;root.pParameters=parameters;
    ComPtr<ID3DBlob> signature,errors,shader;
    hr=D3D12SerializeRootSignature(&root,D3D_ROOT_SIGNATURE_VERSION_1,&signature,&errors);if(FAILED(hr))return hr;
    hr=h.device->CreateRootSignature(0,signature->GetBufferPointer(),signature->GetBufferSize(),IID_PPV_ARGS(&o.depth_root));if(FAILED(hr))return hr;
    const char* source="Texture2D<float> depth:register(t0); RWStructuredBuffer<float> result:register(u0);"
        "[numthreads(8,8,1)] void main(uint3 p:SV_DispatchThreadID){if(p.x<320 && p.y<180)result[p.y*320+p.x]=depth.Load(int3(p.xy,0));}";
    hr=D3DCompile(source,std::strlen(source),nullptr,nullptr,nullptr,"main","cs_5_0",D3DCOMPILE_WARNINGS_ARE_ERRORS,0,&shader,&errors);
    if(FAILED(hr))return hr;
    D3D12_COMPUTE_PIPELINE_STATE_DESC pipeline{};pipeline.pRootSignature=o.depth_root.Get();
    pipeline.CS={shader->GetBufferPointer(),shader->GetBufferSize()};
    hr=h.device->CreateComputePipelineState(&pipeline,IID_PPV_ARGS(&o.depth_pipeline));if(FAILED(hr))return hr;
    auto buffer=buffer_desc(320*180*4);buffer.Flags=D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS;
    auto properties=heap_properties(D3D12_HEAP_TYPE_DEFAULT);
    hr=h.device->CreateCommittedResource(&properties,D3D12_HEAP_FLAG_NONE,&buffer,D3D12_RESOURCE_STATE_UNORDERED_ACCESS,nullptr,IID_PPV_ARGS(&o.depth_sample));if(FAILED(hr))return hr;
    buffer.Flags=D3D12_RESOURCE_FLAG_NONE;properties=heap_properties(D3D12_HEAP_TYPE_READBACK);
    hr=h.device->CreateCommittedResource(&properties,D3D12_HEAP_FLAG_NONE,&buffer,D3D12_RESOURCE_STATE_COPY_DEST,nullptr,IID_PPV_ARGS(&o.depth_sample_readback));if(FAILED(hr))return hr;
    ID3D12DescriptorHeap* heaps[]={o.depth_srv.Get()};h.list->SetDescriptorHeaps(1,heaps);
    h.list->SetComputeRootSignature(o.depth_root.Get());h.list->SetPipelineState(o.depth_pipeline.Get());
    h.list->SetComputeRootDescriptorTable(0,o.depth_srv->GetGPUDescriptorHandleForHeapStart());
    h.list->SetComputeRootUnorderedAccessView(1,o.depth_sample->GetGPUVirtualAddress());h.list->Dispatch(40,23,1);
    transition(h.list.Get(),o.depth_sample.Get(),D3D12_RESOURCE_STATE_UNORDERED_ACCESS,D3D12_RESOURCE_STATE_COPY_SOURCE);
    h.list->CopyBufferRegion(o.depth_sample_readback.Get(),0,o.depth_sample.Get(),0,320*180*4);
    if(stencil) {
        UINT64 bytes=0;h.device->GetCopyableFootprints(&d,1,1,0,&o.stencil_footprint,nullptr,nullptr,&bytes);
        buffer=buffer_desc(bytes);
        hr=h.device->CreateCommittedResource(&properties,D3D12_HEAP_FLAG_NONE,&buffer,D3D12_RESOURCE_STATE_COPY_DEST,nullptr,IID_PPV_ARGS(&o.stencil_readback));if(FAILED(hr))return hr;
    }
    std::fprintf(stderr,"FORMAT_DEPTH {\"allocation\":%u,\"dsv\":%u,\"srv\":%u,\"casting\":%u}\n",
        static_cast<uint32_t>(d.Format),static_cast<uint32_t>(dsv_format),static_cast<uint32_t>(srv_format),options.CastingFullyTypedFormatSupported?1u:0u);
    return S_OK;
}
extern "C" __declspec(dllexport) int32_t __cdecl gpu_formats_depth_copy(void* handle,uint32_t stencil,uint8_t* bytes,uint32_t count) {
    if(!handle || !bytes)return E_INVALIDARG;
    auto& o=*static_cast<FormatHarness*>(handle);
    if(o.base.fence->GetCompletedValue()!=1 || count!=320u*180u*(stencil?1u:4u))return E_INVALIDARG;
    auto* resource=stencil?o.stencil_readback.Get():o.depth_sample_readback.Get();if(!resource)return E_INVALIDARG;
    uint8_t* mapped=nullptr;HRESULT hr=resource->Map(0,nullptr,reinterpret_cast<void**>(&mapped));if(FAILED(hr))return hr;
    if(stencil) {
        for(uint32_t y=0;y<180;++y)std::memcpy(bytes+y*320,mapped+o.stencil_footprint.Offset+y*o.stencil_footprint.Footprint.RowPitch,320);
    } else std::memcpy(bytes,mapped,count);
    resource->Unmap(0,nullptr);return S_OK;
}
