// SPDX-License-Identifier: MPL-2.0

// Bounded direct-SDK experiment. No production callback API is implemented.
#define NOMINMAX
#include <windows.h>
#include <d3d12.h>
#include <wrl/client.h>
#include <dx12/ffx_api_dx12.h>
#include <ffx_upscale.h>
#include "../../crates/fsr-sdk/tests/native/cpu_equipment.h"
#include <atomic>
#include <cstddef>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <map>
#include <memory>
#include <mutex>
#include <string>
#include <vector>

extern "C" __declspec(dllimport) int32_t gpu_finish(void*, CpuMetrics*);
using Microsoft::WRL::ComPtr;
static_assert(sizeof(FfxApiConstantBufferAllocation) == 56);
static_assert(alignof(FfxApiConstantBufferAllocation) == 8);
static_assert(offsetof(FfxApiConstantBufferAllocation, handle) == 48);
static_assert(FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_VERBOSE == 0x0fffffffu);

template<class T> T symbol(HMODULE module, const char* name) {
    const auto raw = GetProcAddress(module, name); T result{};
    static_assert(sizeof(result) == sizeof(raw)); std::memcpy(&result, &raw, sizeof(result));
    if (!result) { std::fprintf(stderr, "Missing export: %s\n", name); std::exit(10); }
    return result;
}
void require(bool value, const char* message) {
    if (!value) { std::fprintf(stderr, "REJECTED %s\n", message); std::exit(11); }
}
FfxApiConstantBufferAllocation pattern(void* data, uint64_t size) {
    FfxApiConstantBufferAllocation out{};
    out.resource.resource = data;
    auto& d = out.resource.description;
    d.type=0x10000001; d.format=0x20000002; d.width=0x30000003; d.height=0x40000004;
    d.depth=0x50000005; d.mipCount=0x60000006; d.flags=0x70000007; d.usage=0x80000008;
    out.resource.state=0x90000009; out.handle=0xf0e1d2c3b4a59687ull ^ size;
    return out;
}
FfxApiConstantBufferAllocation wrong_pattern(void* data, uint64_t size) {
    auto out=pattern(data,size); out.resource.state^=1; return out;
}
void abi(const wchar_t* dll) {
    HMODULE module=LoadLibraryW(dll); require(module!=nullptr,"Rust fixture load");
    const auto layout=symbol<uint64_t(*)(uint32_t)>(module,"callback_layout");
    const auto callback=symbol<FfxApiConstantBufferAllocator>(module,"callback_return");
    const auto invoke=symbol<uint32_t(*)(FfxApiConstantBufferAllocator,void*,uint64_t)>(module,"callback_invoke");
    const size_t expected[]={sizeof(FfxApiConstantBufferAllocation),alignof(FfxApiConstantBufferAllocation),
        offsetof(FfxApiConstantBufferAllocation,handle),sizeof(FfxApiResource),alignof(FfxApiResource),
        offsetof(FfxApiResource,description),offsetof(FfxApiResource,state)};
    for(uint32_t i=0;i<7;++i) require(layout(i)==expected[i],"Rust/C++ layout");
    uint8_t payload[8]={1,2,3,4,5,6,7,8};
    const uint64_t sizes[]={0,1,257,0x1234567800000011ull};
    for(auto size:sizes) {
        const auto got=callback(payload,size), want=pattern(payload,size);
        require(got.resource.resource==payload && got.resource.state==want.resource.state
            && got.handle==want.handle,"Rust aggregate outer fields");
        const auto& a=got.resource.description; const auto& b=want.resource.description;
        require(a.type==b.type && a.format==b.format && a.width==b.width && a.height==b.height
            && a.depth==b.depth && a.mipCount==b.mipCount && a.flags==b.flags && a.usage==b.usage,"Rust aggregate description");
        require(invoke(pattern,payload,size)==1,"C++ aggregate returned to Rust");
        require(invoke(wrong_pattern,payload,size)==0,"Corrupted-field negative control");
    }
    std::printf("ABI {\"size\":56,\"alignment\":8,\"handle_offset\":48,\"exchanges_each_direction\":4,\"negative_controls\":4}\n");
    FreeLibrary(module);
}

#ifndef CALLBACK_ABI_ONLY
std::mutex events_mutex;
std::atomic<const char*> phase{"startup"};
std::map<DWORD,uint32_t> threads;
uint64_t sequence=0, next_allocation=0;
thread_local uint32_t callback_depth=0;
struct Block {uint64_t id, bytes; uint32_t owner;};
std::map<void*,Block> host_blocks, resource_blocks, heap_blocks;
std::vector<ComPtr<ID3D12Resource>> cb_backing;
ID3D12Device* callback_device=nullptr;
ComPtr<ID3D12Device> retained_device;
uint32_t host_state=0x1234abcd;

// One short logging lock; no native call is made while it is held. Thread labels
// and allocation IDs preserve identity without retaining addresses or native TIDs.
void event(const char* kind, uint32_t set, uint64_t id=0, uint64_t bytes=0,
           uint32_t effect=0, uint32_t owner=0) {
    std::lock_guard<std::mutex> lock(events_mutex);
    const DWORD tid=GetCurrentThreadId();
    auto it=threads.find(tid);
    if(it==threads.end()) it=threads.emplace(tid,static_cast<uint32_t>(threads.size()+1)).first;
    std::printf("EVENT {\"seq\":%llu,\"phase\":\"%s\",\"kind\":\"%s\",\"set\":%u,\"allocation\":%llu,\"bytes\":%llu,\"effect\":%u,\"allocation_set\":%u,\"thread\":%u,\"depth\":%u}\n",
        ++sequence,phase.load(),kind,set,id,bytes,effect,owner,it->second,callback_depth);
    std::fflush(stdout);
}
struct CallbackScope {CallbackScope(){++callback_depth;} ~CallbackScope(){--callback_depth;}};
Block add(std::map<void*,Block>& blocks,void* pointer,uint64_t bytes,uint32_t set) {
    std::lock_guard<std::mutex> lock(events_mutex);
    Block block{++next_allocation,bytes,set};
    require(blocks.emplace(pointer,block).second,"Duplicate live allocation");return block;
}
Block remove(std::map<void*,Block>& blocks,void* pointer) {
    std::lock_guard<std::mutex> lock(events_mutex);
    const auto it=blocks.find(pointer); if(it==blocks.end())return {};
    const Block block=it->second;blocks.erase(it);return block;
}
void* host_alloc(void* state,uint64_t size) noexcept {
    CallbackScope scope; require(state==&host_state,"Host userdata round trip");
    void* pointer=_aligned_malloc(static_cast<size_t>(size?size:1),16);
    require(pointer!=nullptr,"Probe host allocation exhausted");
    const auto block=add(host_blocks,pointer,size,1);event("host_alloc",1,block.id,size);return pointer;
}
void host_free(void* state,void* pointer) noexcept {
    CallbackScope scope;require(state==&host_state,"Host free userdata");
    const auto block=remove(host_blocks,pointer);require(!pointer||block.id!=0,"Unknown host free");
    event("host_free",1,block.id,block.bytes);_aligned_free(pointer);
}
ffxAllocationCallbacks host_callbacks{&host_state,host_alloc,host_free};

template<uint32_t Set> ffxReturnCode_t resource_alloc(uint32_t effect,D3D12_RESOURCE_STATES state,
    const D3D12_HEAP_PROPERTIES* heap,const D3D12_RESOURCE_DESC* desc,
    const FfxApiResourceDescription* ffx_desc,const D3D12_CLEAR_VALUE* clear,ID3D12Resource** out) noexcept {
    CallbackScope scope;(void)ffx_desc;
    const HRESULT hr=callback_device->CreateCommittedResource(heap,D3D12_HEAP_FLAG_NONE,desc,state,clear,IID_PPV_ARGS(out));
    require(SUCCEEDED(hr),"Probe resource allocation failed");
    const auto bytes=callback_device->GetResourceAllocationInfo(0,1,desc).SizeInBytes;
    const auto block=add(resource_blocks,*out,bytes,Set);event("resource_alloc",Set,block.id,block.bytes,effect,Set);
    return FFX_API_RETURN_OK;
}
template<uint32_t Set> ffxReturnCode_t resource_free(uint32_t effect,ID3D12Resource* resource) noexcept {
    CallbackScope scope;const auto block=remove(resource_blocks,resource);
    event("resource_free",Set,block.id,block.bytes,effect,block.owner);
    if(resource)resource->Release();return FFX_API_RETURN_OK;
}
template<uint32_t Set> ffxReturnCode_t heap_alloc(uint32_t effect,const D3D12_HEAP_DESC* desc,bool aliasable,
    ID3D12Heap** out,uint64_t* offset) noexcept {
    CallbackScope scope;(void)aliasable;*offset=0;
    require(SUCCEEDED(callback_device->CreateHeap(desc,IID_PPV_ARGS(out))),"Probe heap allocation failed");
    const auto block=add(heap_blocks,*out,desc->SizeInBytes,Set);event("heap_alloc",Set,block.id,block.bytes,effect,Set);
    return FFX_API_RETURN_OK;
}
template<uint32_t Set> ffxReturnCode_t heap_free(uint32_t effect,ID3D12Heap* heap,uint64_t offset,uint64_t size) noexcept {
    CallbackScope scope;require(offset==0,"Unexpected probe heap offset");
    const auto block=remove(heap_blocks,heap);event("heap_free",Set,block.id,size,effect,block.owner);
    if(heap)heap->Release();return FFX_API_RETURN_OK;
}
template<uint32_t Set> FfxApiConstantBufferAllocation cb_alloc(void* data,uint64_t size) noexcept {
    CallbackScope scope;require(data!=nullptr && size>0 && size<=1048576,"CB payload bounds");
    D3D12_HEAP_PROPERTIES heap{};heap.Type=D3D12_HEAP_TYPE_UPLOAD;heap.CreationNodeMask=heap.VisibleNodeMask=1;
    D3D12_RESOURCE_DESC desc{};desc.Dimension=D3D12_RESOURCE_DIMENSION_BUFFER;desc.Width=(size+255)&~255ull;
    desc.Height=desc.DepthOrArraySize=desc.MipLevels=1;desc.SampleDesc.Count=1;desc.Layout=D3D12_TEXTURE_LAYOUT_ROW_MAJOR;
    ComPtr<ID3D12Resource> resource;
    require(SUCCEEDED(callback_device->CreateCommittedResource(&heap,D3D12_HEAP_FLAG_NONE,&desc,
        D3D12_RESOURCE_STATE_GENERIC_READ,nullptr,IID_PPV_ARGS(&resource))),"CB upload resource");
    void* mapped=nullptr;require(SUCCEEDED(resource->Map(0,nullptr,&mapped)),"CB map");
    std::memcpy(mapped,data,static_cast<size_t>(size));resource->Unmap(0,nullptr);
    FfxApiConstantBufferAllocation out{};
    out.resource=ffxApiGetResourceDX12(resource.Get(),FFX_API_RESOURCE_STATE_GENERIC_READ);
    out.handle=resource->GetGPUVirtualAddress();require(out.handle!=0 && out.handle%256==0,"CB address alignment");
    uint64_t count=0;{std::lock_guard<std::mutex> lock(events_mutex);cb_backing.push_back(resource);count=cb_backing.size();}
    event("cb_alloc",Set,count,size);return out;
}
template<uint32_t Set> void message(uint32_t severity,const wchar_t* text) noexcept {
    CallbackScope scope;uint64_t units=0,hash=14695981039346656037ull;
    if(text)for(;units<4096 && text[units];++units){hash^=static_cast<uint16_t>(text[units]);hash*=1099511628211ull;}
    event("message",Set,hash,units,severity);
}

struct Api {
    HMODULE module;PfnFfxCreateContext create;PfnFfxDestroyContext destroy;
    PfnFfxDispatch dispatch;PfnFfxQuery query;PfnFfxConfigure configure;
    explicit Api(const wchar_t* path) {
        module=LoadLibraryExW(path,nullptr,LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR|LOAD_LIBRARY_SEARCH_DEFAULT_DIRS);
        require(module!=nullptr,"AMD loader");
        create=symbol<PfnFfxCreateContext>(module,"ffxCreateContext");destroy=symbol<PfnFfxDestroyContext>(module,"ffxDestroyContext");
        dispatch=symbol<PfnFfxDispatch>(module,"ffxDispatch");query=symbol<PfnFfxQuery>(module,"ffxQuery");
        configure=symbol<PfnFfxConfigure>(module,"ffxConfigure");
    }
    ~Api(){FreeLibrary(module);}
};

void global_debug(Api& api) {
    ffxQueryDescHeader unknown{0x00fefffe,nullptr};
    auto query=[&](){const auto code=api.query(nullptr,&unknown);
        std::printf("QUERY {\"phase\":\"%s\",\"code\":%u}\n",phase.load(),code);
        require(code==FFX_API_RETURN_NO_PROVIDER || code==FFX_API_RETURN_ERROR_UNKNOWN_DESCTYPE,"Unsupported query code");};
    ffxConfigureDescGlobalDebug desc{};desc.header.type=FFX_API_CONFIGURE_DESC_TYPE_GLOBALDEBUG;
    desc.effectId=FFX_API_EFFECT_ID_UPSCALE;desc.fpMessage=message<1>;desc.debugLevel=FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_VERBOSE;
    auto configure=[&](ffxConfigureDescHeader* header,uint64_t effect){
        const auto code=api.configure(nullptr,header);
        std::printf("CONFIGURE {\"phase\":\"%s\",\"code\":%u,\"effect\":%llu,\"type\":%llu}\n",phase.load(),code,effect,header->type);
        event("configure_return",0,0,0,code);return code;};
    phase="global-upscale";require(configure(&desc.header,desc.effectId)==0,"Global debug configure");query();
    phase="global-other-effect";desc.effectId=0x00fe0000;desc.fpMessage=message<2>;
    configure(&desc.header,desc.effectId);query();
    phase="global-deprecated";ffxConfigureDescGlobalDebug1 old{};old.header.type=FFX_API_CONFIGURE_DESC_TYPE_GLOBALDEBUG1;
    old.fpMessage=message<1>;old.debugLevel=FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_WARNINGS;
    configure(&old.header,0);query();
    phase="global-null";desc.effectId=FFX_API_EFFECT_ID_UPSCALE;desc.fpMessage=nullptr;configure(&desc.header,desc.effectId);query();
}

constexpr uint32_t debug_flags=FFX_UPSCALE_ENABLE_DEBUG_CHECKING|FFX_UPSCALE_ENABLE_DEBUG_VISUALIZATION;
struct Context {
    ffxContext handle=nullptr;
    ffxCreateContextDescUpscale root{};ffxCreateContextDescUpscaleVersion version{};
    ffxCreateBackendDX12Desc backend{};ffxCreateBackendDX12AllocationCallbacksDesc callbacks{};ffxOverrideVersion provider{};
};
void create(Api& api,Context& context,uint64_t provider,uint32_t set,bool partial) {
    auto& c=context;c.root.header.type=FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE;
    c.root.header.pNext=&c.version.header;c.root.flags=debug_flags;c.root.maxRenderSize={320,180};c.root.maxUpscaleSize={640,360};
    c.root.fpMessage=partial?message<3>:set==2?message<2>:message<1>;
    c.version.header.type=FFX_API_CREATE_CONTEXT_DESC_TYPE_UPSCALE_VERSION;c.version.version=FFX_UPSCALER_VERSION;
    c.version.header.pNext=&c.backend.header;c.backend.header.type=FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12;c.backend.device=callback_device;
    c.provider.header.type=FFX_API_DESC_TYPE_OVERRIDE_VERSION;c.provider.versionId=provider;
    c.backend.header.pNext=set?&c.callbacks.header:&c.provider.header;
    c.callbacks.header.type=FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12_ALLOCATION_CALLBACKS;c.callbacks.header.pNext=&c.provider.header;
    if(set==1){c.callbacks.pfnFfxResourceAllocator=resource_alloc<1>;c.callbacks.pfnFfxHeapAllocator=heap_alloc<1>;
        if(!partial){c.callbacks.pfnFfxResourceDeallocator=resource_free<1>;c.callbacks.pfnFfxHeapDeallocator=heap_free<1>;c.callbacks.pfnFfxConstantBufferAllocator=cb_alloc<1>;}}
    if(set==2){c.callbacks.pfnFfxResourceAllocator=resource_alloc<2>;c.callbacks.pfnFfxResourceDeallocator=resource_free<2>;
        c.callbacks.pfnFfxHeapAllocator=heap_alloc<2>;c.callbacks.pfnFfxHeapDeallocator=heap_free<2>;c.callbacks.pfnFfxConstantBufferAllocator=cb_alloc<2>;}
    const auto code=api.create(&c.handle,&c.root.header,&host_callbacks);
    std::printf("CREATE {\"phase\":\"%s\",\"code\":%u,\"set\":%u,\"partial\":%s,\"flags\":%u}\n",phase.load(),code,set,partial?"true":"false",debug_flags);
    // No failed-create handle operation. All experiment state remains alive up
    // to process exit on failure; require exits without unwinding this frame.
    require(code==0 && c.handle!=nullptr,"Native create");
    ffxQueryGetProviderVersion identity{};identity.header.type=FFX_API_QUERY_DESC_TYPE_GET_PROVIDER_VERSION;
    require(api.query(&c.handle,&identity.header)==0 && identity.versionId==provider,"Actual selected provider");
    std::printf("PROVIDER {\"id\":%llu,\"name\":\"%s\"}\n",identity.versionId,identity.versionName?identity.versionName:"");
}
void frame(Api& api,Context& context,bool view) {
    void* harness=nullptr;ID3D12GraphicsCommandList* raw_list=nullptr;ID3D12Resource* raw[5]{};
    require(SUCCEEDED(gpu_create(callback_device,&harness,&raw_list,raw)),"GPU workload setup");
    ComPtr<ID3D12GraphicsCommandList> list;list.Attach(raw_list);ComPtr<ID3D12Resource> resources[5];
    for(size_t i=0;i<5;++i)resources[i].Attach(raw[i]);
    ffxDispatchDescUpscale d{};d.header.type=FFX_API_DISPATCH_DESC_TYPE_UPSCALE;d.commandList=list.Get();
    d.color=ffxApiGetResourceDX12(raw[0]);d.depth=ffxApiGetResourceDX12(raw[1]);d.motionVectors=ffxApiGetResourceDX12(raw[2]);
    d.exposure=ffxApiGetResourceDX12(raw[3]);d.output=ffxApiGetResourceDX12(raw[4],FFX_API_RESOURCE_STATE_UNORDERED_ACCESS);
    d.renderSize={320,180};d.upscaleSize={640,360};d.motionVectorScale={1,1};d.frameTimeDelta=0.5f;d.preExposure=1;
    d.reset=true;d.cameraNear=0.1f;d.cameraFar=100;d.cameraFovAngleVertical=1.0471975512f;d.viewSpaceToMetersFactor=1;
    d.flags=view?FFX_UPSCALE_FLAG_DRAW_DEBUG_VIEW:0;
    const auto code=api.dispatch(&context.handle,&d.header);require(code==0,"Native dispatch");
    CpuMetrics metrics{};require(SUCCEEDED(gpu_finish(harness,&metrics)),"GPU submit/fence/readback");
    std::printf("FRAME {\"phase\":\"%s\",\"flags\":%u,\"finite\":%u,\"written\":%u,\"errors\":%llu,\"removed\":%d}\n",
        phase.load(),d.flags,metrics.finite,metrics.changed,metrics.errors,metrics.removed);
    require(metrics.errors==0 && metrics.removed==0 && metrics.changed==640*360 && metrics.finite==640*360,"GPU integration checks");
    gpu_destroy(harness);
}
void destroy(Api& api,Context& context) {
    const auto code=api.destroy(&context.handle,&host_callbacks);
    std::printf("DESTROY {\"phase\":\"%s\",\"code\":%u}\n",phase.load(),code);
    // Exactly one attempt. Failure exits with callback dependencies retained.
    require(code==0,"Native destroy");context.handle=nullptr;
}
void native(Api& api,const wchar_t* path,uint64_t provider,bool shared) {
    require(SUCCEEDED(gpu_enable_debug()),"D3D12 debug layer");ComPtr<ID3D12Device> device;
    require(SUCCEEDED(probe_create_device(&device)) && SUCCEEDED(gpu_require_amd(device.Get())),"Explicit AMD adapter");
    retained_device=device;callback_device=device.Get();Api second(path);require(api.module==second.module,"Shared loader image");
    event("main_thread",0);
    phase="estimate";FfxApiEffectMemoryUsage usage{};ffxOverrideVersion chosen{};
    chosen.header.type=FFX_API_DESC_TYPE_OVERRIDE_VERSION;chosen.versionId=provider;
    ffxQueryDescUpscaleGetGPUMemoryUsageV2 estimate{};estimate.header.type=FFX_API_QUERY_DESC_TYPE_UPSCALE_GPU_MEMORY_USAGE_V2;
    estimate.header.pNext=&chosen.header;estimate.device=device.Get();estimate.maxRenderSize={320,180};estimate.maxUpscaleSize={640,360};
    estimate.flags=debug_flags;estimate.gpuMemoryUsageUpscaler=&usage;
    require(api.query(nullptr,&estimate.header)==0,"Matching flag/provider memory estimate");
    std::printf("ESTIMATE {\"flags\":%u,\"total\":%llu,\"aliasable\":%llu}\n",estimate.flags,usage.totalUsageInBytes,usage.aliasableUsageInBytes);
    Context a,b,c,d;
    if(shared){
        phase="configure-shared-messages";ffxConfigureDescGlobalDebug global{};
        global.header.type=FFX_API_CONFIGURE_DESC_TYPE_GLOBALDEBUG;global.effectId=FFX_API_EFFECT_ID_UPSCALE;
        global.fpMessage=message<1>;global.debugLevel=FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_WARNINGS;
        const auto code=api.configure(nullptr,&global.header);
        std::printf("CONFIGURE {\"code\":%u,\"effect\":%llu,\"level\":%u}\n",code,global.effectId,global.debugLevel);
        require(code==0,"Enable shared provider diagnostic route");
    }
    phase="create-a";create(api,a,provider,shared?1:0,false);
    phase="frame-a-before";frame(api,a,false);
    if(shared) {
        phase="create-b";create(second,b,provider,2,false);
        phase="frame-a-after-b";frame(api,a,false);
        phase="frame-b";frame(second,b,true);
        phase="create-omitted";create(second,c,provider,0,false);
        phase="frame-omitted";frame(second,c,false);
        phase="create-partial";create(second,d,provider,1,true);
        phase="frame-partial";frame(second,d,false);
        phase="destroy-partial";destroy(second,d);
        phase="destroy-b";destroy(second,b);
        phase="frame-a-after-destroy-b";frame(api,a,true);
        phase="destroy-omitted";destroy(second,c);
    } else {
        phase="configure-loader-global";ffxConfigureDescGlobalDebug global{};
        global.header.type=FFX_API_CONFIGURE_DESC_TYPE_GLOBALDEBUG;global.effectId=FFX_API_EFFECT_ID_UPSCALE;
        global.fpMessage=message<2>;global.debugLevel=FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_VERBOSE;
        const auto code=api.configure(nullptr,&global.header);
        std::printf("CONFIGURE {\"code\":%u,\"effect\":%llu,\"level\":%u}\n",code,global.effectId,global.debugLevel);
        require(code==0,"Loader-global configuration with live context");
        phase="frame-a-view";frame(api,a,true);
    }
    phase="destroy-a";destroy(api,a);
    require(SUCCEEDED(gpu_check_device(device.Get())),"Final debug/device checkpoint");
    std::printf("RESULT {\"host_live\":%zu,\"resource_live\":%zu,\"heap_live\":%zu,\"cb_backing\":%zu,\"shared_module\":true}\n",
        host_blocks.size(),resource_blocks.size(),heap_blocks.size(),cb_backing.size());
    require(host_blocks.empty() && resource_blocks.empty() && heap_blocks.empty(),"Allocation/free balance");
    // Every submitted list is fenced and every native context destroyed. Keep
    // callback globals/code alive through final DLL release; free CB backing last.
    phase="native-complete";
}
#endif
int wmain(int argc,wchar_t** argv) {
    std::setvbuf(stdout,nullptr,_IONBF,0);
    if(argc==3 && std::wstring(argv[1])==L"abi"){abi(argv[2]);return 0;}
#ifdef CALLBACK_ABI_ONLY
    return 2;
#else
    require(argc==5,"Usage: callback-spike global|baseline|shared LOADER PROVIDER ADAPTER");
    SetEnvironmentVariableW(L"FSR_SDK_TEST_ADAPTER_INDEX",argv[4]);
    {Api api(argv[2]);const std::wstring mode=argv[1];
        if(mode==L"global")global_debug(api);
        else native(api,argv[2],std::stoull(argv[3]),mode==L"shared");}
    phase="after-loader-release";event("loader_released",0);cb_backing.clear();
    std::printf("COMPLETE\n");return 0;
#endif
}
