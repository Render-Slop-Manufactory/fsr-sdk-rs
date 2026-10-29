// SPDX-License-Identifier: MPL-2.0

// Independent pinned-header reference. No Rust descriptor construction is used.
#define NOMINMAX
#include <windows.h>
#include <d3d12.h>
#include "ffx_api_dx12.h"
#include "ffx_upscale.h"
#include <cstdint>
#include <memory>
#include <cstring>
#include <wrl/client.h>
template<class T> static T format_symbol(HMODULE module,const char* name) {
    const auto pointer=GetProcAddress(module,name);T result{};
    static_assert(sizeof(result)==sizeof(pointer));std::memcpy(&result,&pointer,sizeof(result));return result;
}
struct FormatReference {
    HMODULE library=nullptr;
    Microsoft::WRL::ComPtr<ID3D12Device> device;
    ffxContext context=nullptr;
    PfnFfxDispatch dispatch=nullptr;
    PfnFfxDestroyContext destroy=nullptr;
    ~FormatReference() {if(library)FreeLibrary(library);}
};
static FfxApiResource format_resource(ID3D12Resource* resource,uint32_t state=FFX_API_RESOURCE_STATE_COMPUTE_READ) {
    auto result=ffxApiGetResourceDX12(resource,state);
    const auto format=resource->GetDesc().Format;
    if(format==DXGI_FORMAT_R32G32B32_FLOAT || format==DXGI_FORMAT_R32G32B32_TYPELESS)
        result.description.format=FFX_API_SURFACE_FORMAT_R32G32B32_FLOAT;
    return result;
}
extern "C" __declspec(dllexport) uint32_t __cdecl formats_reference_create(
    const wchar_t* path,ID3D12Device* device,uint64_t provider,uint32_t flags,void** handle) {
    *handle=nullptr;auto r=std::make_unique<FormatReference>();r->device=device;
    r->library=LoadLibraryExW(path,nullptr,LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR|LOAD_LIBRARY_SEARCH_DEFAULT_DIRS);
    if(!r->library)return 0xffffffffu;
    const auto create=format_symbol<PfnFfxCreateContext>(r->library,"ffxCreateContext");
    const auto query=format_symbol<PfnFfxQuery>(r->library,"ffxQuery");
    r->dispatch=format_symbol<PfnFfxDispatch>(r->library,"ffxDispatch");
    r->destroy=format_symbol<PfnFfxDestroyContext>(r->library,"ffxDestroyContext");
    if(!create||!query||!r->dispatch||!r->destroy)return 0xffffffffu;
    ffxCreateBackendDX12Desc backend{};backend.header.type=FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12;backend.device=device;
    ffxOverrideVersion version{};version.header.type=FFX_API_DESC_TYPE_OVERRIDE_VERSION;version.header.pNext=&backend.header;version.versionId=provider;
    ffxCreateContextDescUpscale desc{};desc.header.type=FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE;desc.header.pNext=&version.header;
    desc.flags=flags;desc.maxRenderSize={320,180};desc.maxUpscaleSize={640,360};
    const auto code=create(&r->context,&desc.header,nullptr);
    // Native errors establish no context ownership. Success without a context
    // retains dependencies rather than inventing a valid destruction target.
    if(code!=FFX_API_RETURN_OK)return code;
    if(!r->context){r.release();return 0xfffffffdu;}
    ffxQueryGetProviderVersion identity{};identity.header.type=FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION;
    const auto queried=query(&r->context,&identity.header);
    if(queried!=FFX_API_RETURN_OK||identity.versionId!=provider){
        if(r->destroy(&r->context,nullptr)!=FFX_API_RETURN_OK)r.release();
        return 0xfffffffeu;
    }
    *handle=r.release();return FFX_API_RETURN_OK;
}
extern "C" __declspec(dllexport) uint32_t __cdecl formats_reference_record(
    void* handle,ID3D12GraphicsCommandList* list,ID3D12Resource* const* resources,
    uint32_t render_width,uint32_t render_height,uint32_t width,uint32_t height,uint32_t phase,uint32_t sharpen,uint32_t generated) {
    auto& r=*static_cast<FormatReference*>(handle);ffxDispatchDescUpscale d{};
    d.header.type=FFX_API_DISPATCH_DESC_TYPE_UPSCALE;d.commandList=list;
    d.color=format_resource(resources[0]);d.depth=ffxApiGetResourceDX12(resources[1]);
    d.motionVectors=ffxApiGetResourceDX12(resources[2]);d.exposure=ffxApiGetResourceDX12(resources[3]);
    d.output=ffxApiGetResourceDX12(resources[4],FFX_API_RESOURCE_STATE_UNORDERED_ACCESS);
    if(generated&1)d.reactive=ffxApiGetResourceDX12(resources[6]);
    if(generated&2)d.transparencyAndComposition=ffxApiGetResourceDX12(resources[6]);
    d.motionVectorScale={1,1};d.renderSize={render_width,render_height};d.upscaleSize={width,height};
    d.enableSharpening=sharpen!=0;d.sharpness=sharpen?0.5f:0.0f;d.frameTimeDelta=16.667f;d.preExposure=1.0f;d.reset=phase==0;
    d.cameraNear=0.1f;d.cameraFar=100.0f;
    d.cameraFovAngleVertical=1.047197551196597746f;d.viewSpaceToMetersFactor=1.0f;
    return r.dispatch(&r.context,&d.header);
}
extern "C" __declspec(dllexport) uint32_t __cdecl formats_reference_generate(
    void* handle,ID3D12GraphicsCommandList* list,ID3D12Resource* const* resources,uint32_t width,uint32_t height) {
    auto& r=*static_cast<FormatReference*>(handle);ffxDispatchDescUpscaleGenerateReactiveMask d{};
    d.header.type=FFX_API_DISPATCH_DESC_TYPE_UPSCALE_GENERATEREACTIVEMASK;d.commandList=list;
    d.colorOpaqueOnly=format_resource(resources[5]);d.colorPreUpscale=format_resource(resources[0]);
    d.outReactive=ffxApiGetResourceDX12(resources[6],FFX_API_RESOURCE_STATE_UNORDERED_ACCESS);
    d.renderSize={width,height};d.scale=1.0f;d.cutoffThreshold=0.2f;d.binaryValue=0.9f;
    return r.dispatch(&r.context,&d.header);
}
extern "C" __declspec(dllexport) uint32_t __cdecl formats_reference_destroy(void* handle) {
    auto* r=static_cast<FormatReference*>(handle);const auto code=r->destroy(&r->context,nullptr);
    if(code==FFX_API_RETURN_OK)delete r;return code;
}
