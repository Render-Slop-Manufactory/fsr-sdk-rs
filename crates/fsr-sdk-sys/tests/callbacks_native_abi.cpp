// SPDX-License-Identifier: MPL-2.0
// Independent assertions against the pinned AMD and installed Microsoft headers.
#include <type_traits>
#include <cstddef>
#include <dx12/ffx_api_dx12.h>
#include <ffx_upscale.h>
#define LAYOUT(T,S,A) static_assert(sizeof(T)==S && alignof(T)==A)
#define FIELD(T,F,FT,O) static_assert(std::is_same_v<decltype(T::F),FT>); static_assert(offsetof(T,F)==O)
LAYOUT(FfxApiConstantBufferAllocation,56,8);
FIELD(FfxApiConstantBufferAllocation,resource,FfxApiResource,0);
FIELD(FfxApiConstantBufferAllocation,handle,uint64_t,48);
LAYOUT(ffxConfigureDescGlobalDebug1,32,8);
FIELD(ffxConfigureDescGlobalDebug1,header,ffxConfigureDescHeader,0);
FIELD(ffxConfigureDescGlobalDebug1,fpMessage,ffxApiMessage,16);
FIELD(ffxConfigureDescGlobalDebug1,debugLevel,uint32_t,24);
LAYOUT(ffxConfigureDescGlobalDebug,40,8);
FIELD(ffxConfigureDescGlobalDebug,header,ffxConfigureDescHeader,0);
FIELD(ffxConfigureDescGlobalDebug,effectId,uint64_t,16);
FIELD(ffxConfigureDescGlobalDebug,fpMessage,ffxApiMessage,24);
FIELD(ffxConfigureDescGlobalDebug,debugLevel,uint32_t,32);
LAYOUT(ffxCreateBackendDX12AllocationCallbacksDesc,56,8);
FIELD(ffxCreateBackendDX12AllocationCallbacksDesc,header,ffxCreateContextDescHeader,0);
FIELD(ffxCreateBackendDX12AllocationCallbacksDesc,pfnFfxResourceAllocator,PfnFfxResourceAllocatorFunc,16);
FIELD(ffxCreateBackendDX12AllocationCallbacksDesc,pfnFfxResourceDeallocator,PfnFfxResourceDeallocatorFunc,24);
FIELD(ffxCreateBackendDX12AllocationCallbacksDesc,pfnFfxHeapAllocator,PfnFfxHeapAllocatorFunc,32);
FIELD(ffxCreateBackendDX12AllocationCallbacksDesc,pfnFfxHeapDeallocator,PfnFfxHeapDeallocatorFunc,40);
FIELD(ffxCreateBackendDX12AllocationCallbacksDesc,pfnFfxConstantBufferAllocator,FfxApiConstantBufferAllocator,48);
LAYOUT(D3D12_HEAP_PROPERTIES,20,4);
FIELD(D3D12_HEAP_PROPERTIES,Type,D3D12_HEAP_TYPE,0);
FIELD(D3D12_HEAP_PROPERTIES,CPUPageProperty,D3D12_CPU_PAGE_PROPERTY,4);
FIELD(D3D12_HEAP_PROPERTIES,MemoryPoolPreference,D3D12_MEMORY_POOL,8);
FIELD(D3D12_HEAP_PROPERTIES,CreationNodeMask,uint32_t,12);
FIELD(D3D12_HEAP_PROPERTIES,VisibleNodeMask,uint32_t,16);
LAYOUT(DXGI_SAMPLE_DESC,8,4);
FIELD(DXGI_SAMPLE_DESC,Count,uint32_t,0);
FIELD(DXGI_SAMPLE_DESC,Quality,uint32_t,4);
LAYOUT(D3D12_RESOURCE_DESC,56,8);
FIELD(D3D12_RESOURCE_DESC,Dimension,D3D12_RESOURCE_DIMENSION,0);
FIELD(D3D12_RESOURCE_DESC,Alignment,uint64_t,8);
FIELD(D3D12_RESOURCE_DESC,Width,uint64_t,16);
FIELD(D3D12_RESOURCE_DESC,Height,uint32_t,24);
FIELD(D3D12_RESOURCE_DESC,DepthOrArraySize,uint16_t,28);
FIELD(D3D12_RESOURCE_DESC,MipLevels,uint16_t,30);
FIELD(D3D12_RESOURCE_DESC,Format,DXGI_FORMAT,32);
FIELD(D3D12_RESOURCE_DESC,SampleDesc,DXGI_SAMPLE_DESC,36);
FIELD(D3D12_RESOURCE_DESC,Layout,D3D12_TEXTURE_LAYOUT,44);
FIELD(D3D12_RESOURCE_DESC,Flags,D3D12_RESOURCE_FLAGS,48);
LAYOUT(D3D12_DEPTH_STENCIL_VALUE,8,4);
FIELD(D3D12_DEPTH_STENCIL_VALUE,Depth,float,0);
FIELD(D3D12_DEPTH_STENCIL_VALUE,Stencil,uint8_t,4);
LAYOUT(D3D12_CLEAR_VALUE,20,4);
FIELD(D3D12_CLEAR_VALUE,Format,DXGI_FORMAT,0);
LAYOUT(D3D12_HEAP_DESC,48,8);
FIELD(D3D12_HEAP_DESC,SizeInBytes,uint64_t,0);
FIELD(D3D12_HEAP_DESC,Properties,D3D12_HEAP_PROPERTIES,8);
FIELD(D3D12_HEAP_DESC,Alignment,uint64_t,32);
FIELD(D3D12_HEAP_DESC,Flags,D3D12_HEAP_FLAGS,40);
static_assert(offsetof(D3D12_CLEAR_VALUE, Color)==4 && offsetof(D3D12_CLEAR_VALUE, DepthStencil)==4);
static_assert(FFX_API_EFFECT_ID_UPSCALE==65536);
static_assert(FFX_API_CONFIGURE_DESC_TYPE_GLOBALDEBUG1==1);
static_assert(FFX_API_CONFIGURE_DESC_TYPE_GLOBALDEBUG==7);
static_assert(FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12_ALLOCATION_CALLBACKS==3);
static_assert(FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_SILENCE==0);
static_assert(FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_ERRORS==1);
static_assert(FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_WARNINGS==2);
static_assert(FFX_API_CONFIGURE_GLOBALDEBUG_LEVEL_VERBOSE==268435455);
static_assert(FFX_UPSCALE_ENABLE_DEBUG_CHECKING==128);
static_assert(FFX_UPSCALE_ENABLE_DEBUG_VISUALIZATION==512);
static_assert(FFX_UPSCALE_FLAG_DRAW_DEBUG_VIEW==1);
static_assert(std::is_same_v<ffxAlloc,void* (__cdecl *)(void*,uint64_t)>);
static_assert(std::is_same_v<ffxDealloc,void (__cdecl *)(void*,void*)>);
static_assert(std::is_same_v<ffxApiMessage,void (__cdecl *)(uint32_t,const wchar_t*)>);
static_assert(std::is_same_v<FfxApiConstantBufferAllocator,FfxApiConstantBufferAllocation (__cdecl *)(void*,uint64_t)>);
static_assert(std::is_same_v<PfnFfxResourceAllocatorFunc,ffxReturnCode_t (__cdecl *)(uint32_t,D3D12_RESOURCE_STATES,const D3D12_HEAP_PROPERTIES*,const D3D12_RESOURCE_DESC*,const FfxApiResourceDescription*,const D3D12_CLEAR_VALUE*,ID3D12Resource**)>);
static_assert(std::is_same_v<PfnFfxResourceDeallocatorFunc,ffxReturnCode_t (__cdecl *)(uint32_t,ID3D12Resource*)>);
static_assert(std::is_same_v<PfnFfxHeapAllocatorFunc,ffxReturnCode_t (__cdecl *)(uint32_t,const D3D12_HEAP_DESC*,bool,ID3D12Heap**,uint64_t*)>);
static_assert(std::is_same_v<PfnFfxHeapDeallocatorFunc,ffxReturnCode_t (__cdecl *)(uint32_t,ID3D12Heap*,uint64_t,uint64_t)>);

static_assert(FFX_API_MESSAGE_TYPE_ERROR==0 && FFX_API_MESSAGE_TYPE_WARNING==1 && FFX_API_MESSAGE_TYPE_COUNT==2);
static_assert(FFX_API_RESOURCE_TYPE_BUFFER==0 && FFX_API_RESOURCE_STATE_GENERIC_READ==20);
static_assert(sizeof(D3D12_RESOURCE_STATES)==4 && alignof(D3D12_RESOURCE_STATES)==4);
static_assert(sizeof(D3D12_HEAP_TYPE)==4 && sizeof(D3D12_CPU_PAGE_PROPERTY)==4 && sizeof(D3D12_MEMORY_POOL)==4);
static_assert(sizeof(D3D12_RESOURCE_DIMENSION)==4 && sizeof(DXGI_FORMAT)==4 && sizeof(D3D12_TEXTURE_LAYOUT)==4 && sizeof(D3D12_RESOURCE_FLAGS)==4 && sizeof(D3D12_HEAP_FLAGS)==4);
