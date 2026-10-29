// SPDX-License-Identifier: MPL-2.0

// Reuse the bounded, externally owned allocator implementation without changing
// the historical spike source. This DLL is equipment, never part of either crate.
#define wmain callback_spike_unused_main
#include "callback_spike.cpp"
#undef wmain

extern "C" __declspec(dllexport) void callbacks_device(ID3D12Device* device) {
    retained_device=device; callback_device=device; phase="public-setup";
}
extern "C" __declspec(dllexport) void callbacks_tables(uint32_t set,
    ffxAllocationCallbacks* host, ffxCreateBackendDX12AllocationCallbacksDesc* dx12,
    ffxApiMessage* diagnostic) {
    *host=host_callbacks; *dx12={};
    dx12->header.type=FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12_ALLOCATION_CALLBACKS;
    if(set==1) {
        dx12->pfnFfxResourceAllocator=resource_alloc<1>; dx12->pfnFfxResourceDeallocator=resource_free<1>;
        dx12->pfnFfxHeapAllocator=heap_alloc<1>; dx12->pfnFfxHeapDeallocator=heap_free<1>;
        dx12->pfnFfxConstantBufferAllocator=cb_alloc<1>; *diagnostic=message<1>;
    } else {
        dx12->pfnFfxResourceAllocator=resource_alloc<2>; dx12->pfnFfxResourceDeallocator=resource_free<2>;
        dx12->pfnFfxHeapAllocator=heap_alloc<2>; dx12->pfnFfxHeapDeallocator=heap_free<2>;
        dx12->pfnFfxConstantBufferAllocator=cb_alloc<2>; *diagnostic=message<2>;
    }
}
extern "C" __declspec(dllexport) void callbacks_heap_control() {
    phase="controlled-heap";
    D3D12_HEAP_DESC desc{}; desc.SizeInBytes=65536;desc.Alignment=65536;
    desc.Properties.Type=D3D12_HEAP_TYPE_DEFAULT;
    desc.Properties.CreationNodeMask=desc.Properties.VisibleNodeMask=1;
    desc.Flags=D3D12_HEAP_FLAG_ALLOW_ONLY_BUFFERS;
    ID3D12Heap* heap=nullptr;uint64_t offset=0;
    require(heap_alloc<1>(0x10000,&desc,false,&heap,&offset)==0,"Controlled heap allocation");
    require(heap_free<1>(0x10000,heap,offset,desc.SizeInBytes)==0,"Controlled heap free");
    phase="public-sdk";
}
extern "C" __declspec(dllexport) void callbacks_balance() {
    require(host_blocks.empty() && resource_blocks.empty() && heap_blocks.empty(),"Public allocation/free balance");
    std::printf("PUBLIC_BALANCE {\"host_live\":%zu,\"resource_live\":%zu,\"heap_live\":%zu,\"cb_backing\":%zu}\n",
        host_blocks.size(),resource_blocks.size(),heap_blocks.size(),cb_backing.size());
    std::fflush(stdout);
    // CB backing, callback code, retained device and all target state deliberately
    // remain process-live. No successful native return is used as unregister.
}
