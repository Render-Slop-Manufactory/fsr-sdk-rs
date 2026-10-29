// SPDX-License-Identifier: MPL-2.0

//! Explicit unsafe integration of native callbacks. Tables own no callback targets.
//!
//! # Message semantics
//!
//! Messages use Windows UTF-16 and may carry unknown severity values, which
//! consumers must preserve. Transient message-pointer validity and copying
//! belong to the lasting safety contract below.
//!
//! # Safety contract
//! Installation through setup `create_with_callbacks` or a diagnostic setter requires
//! valid callback code, userdata, allocator bookkeeping and independent device
//! references for every possible native use. Registration can affect sibling
//! contexts, other runtime acquisitions and later ordinary constructors. Omitted
//! DX12 groups do not reset previous module registrations. Neither a successful
//! destroy, an error return nor replacing/nulling a pointer proves quiescence.
//! Retain targets until no active or future native invocation can reach them;
//! process-lifetime targets are a valid strategy. This module installs no guard,
//! closure registry, automatic unregister or target destructor.
//!
//! Coordinate every affected native user, including implicit Drop and unwinding;
//! native workers must not race registration changes. Callbacks must not reenter
//! the affected SDK domain or unwind across C. Establish actual native callback
//! affinity; local Rust ownership does not prevent native worker callbacks.
//! Movable construction additionally requires callback/target compatibility with
//! every permitted destination thread and cleanup there, without concurrent use.
//!
//! Host allocation supplies at least the requested bytes aligned for any native
//! type; host free accepts null and matches its allocator. Replacement resource
//! and heap frees must remain compatible with every outstanding allocation.
//! Error/null returns permit recovery only where the actual native consumer
//! safely supports them; otherwise allocation must succeed or terminate.
//! Resource/heap callbacks transfer a usable COM reference whose matching free
//! performs release/accounting. Borrowed pointers do not create an owned COM ref.
//! CB data is payload: copy it into suitably aligned GPU-visible backing and
//! retain backing/ranges/resources through GPU completion. COM retention does
//! not prevent suballocation reuse. Renderer submission/residency/fences remain
//! external. Message text is transient Windows UTF-16, may be null, and must be
//! consumed/copied during invocation.
//!
//! ## Native assumptions and uncertainty
//!
//! These raw callback parameter types are an explicit public compatibility
//! boundary. The wrapper retains tables/descriptors, not their external pointees.
//! It retains its bundle on callback-bearing native creation failure, OK/null
//! creation and failed destruction, without touching a failed-create handle.
//! After successful destroy, native code must stop accessing wrapper-owned
//! descriptor/table storage even if copied global callback functions remain live.
//! Callback code/state lifetime, compatible allocation and CPU synchronization
//! protect against dangling pointers, invalid frees and data races. General
//! native quiescence, callback affinity and registration protection are not
//! supplied by the wrapper. They remain actual-provider assumptions; neither
//! missing evidence nor a successful call discharges the obligations above.

use fsr_sdk_sys::{api, dx12};
use std::{ffi::c_void, marker::PhantomData, ptr::null_mut, rc::Rc};

/// Exact native callback payloads and constants, without SDK context handles.
/// These types do not own COM references; CB addresses are explicit payload.
/// Pointer-only Microsoft metadata mirrors the system ABI. Cast to the matching
/// `windows` 0.62 metadata pointer only after establishing layout compatibility.
pub mod abi {
    pub use fsr_sdk_sys::api::{
        FFX_API_MESSAGE_TYPE_COUNT, FFX_API_MESSAGE_TYPE_ERROR, FFX_API_MESSAGE_TYPE_WARNING,
        FFX_API_RESOURCE_FLAGS_NONE, FFX_API_RESOURCE_STATE_GENERIC_READ,
        FFX_API_RESOURCE_TYPE_BUFFER, FFX_API_RESOURCE_USAGE_READ_ONLY, FFX_API_RESOURCE_USAGE_UAV,
        FFX_API_RETURN_ERROR, FFX_API_RETURN_ERROR_MEMORY, FFX_API_RETURN_ERROR_PARAMETER,
        FFX_API_RETURN_OK, FFX_API_SURFACE_FORMAT_UNKNOWN, FfxApiConstantBufferAllocation,
        FfxApiResource, FfxApiResourceDepth, FfxApiResourceDescription, FfxApiResourceHeight,
        FfxApiResourceWidth,
    };
    pub use fsr_sdk_sys::dx12::{
        D3D12_CLEAR_VALUE, D3D12_CLEAR_VALUE_0, D3D12_DEPTH_STENCIL_VALUE, D3D12_HEAP_DESC,
        D3D12_HEAP_PROPERTIES, D3D12_RESOURCE_DESC, D3D12_RESOURCE_STATES, DXGI_SAMPLE_DESC,
        ID3D12Heap, ID3D12Resource,
    };
}
pub use api::{
    FfxApiConstantBufferAllocatorFn as ConstantBufferAllocator, ffxAllocFn as HostAllocate,
    ffxApiMessageFn as MessageCallback, ffxDeallocFn as HostDeallocate,
};
pub use dx12::{
    FfxHeapAllocatorFn as HeapAllocate, FfxHeapDeallocatorFn as HeapDeallocate,
    FfxResourceAllocatorFn as ResourceAllocate, FfxResourceDeallocatorFn as ResourceDeallocate,
};

/// Host callback pair and external userdata; this value does not free userdata.
pub struct HostAllocationCallbacks {
    user_data: *mut c_void,
    allocate: HostAllocate,
    deallocate: HostDeallocate,
}
impl HostAllocationCallbacks {
    /// Assemble a non-null pair without invoking native code or owning userdata.
    pub fn new(user_data: *mut c_void, allocate: HostAllocate, deallocate: HostDeallocate) -> Self {
        Self {
            user_data,
            allocate,
            deallocate,
        }
    }
}
/// Indivisible native resource allocation/free pair; owns no allocator state.
pub struct ResourceAllocationCallbacks {
    allocate: ResourceAllocate,
    deallocate: ResourceDeallocate,
}
impl ResourceAllocationCallbacks {
    /// Assemble a complete pair; installation requires the module safety contract.
    pub fn new(allocate: ResourceAllocate, deallocate: ResourceDeallocate) -> Self {
        Self {
            allocate,
            deallocate,
        }
    }
}
/// Indivisible native heap allocation/free pair; owns no heap or allocator state.
pub struct HeapAllocationCallbacks {
    allocate: HeapAllocate,
    deallocate: HeapDeallocate,
}
impl HeapAllocationCallbacks {
    /// Assemble a complete pair; installation requires the module safety contract.
    pub fn new(allocate: HeapAllocate, deallocate: HeapDeallocate) -> Self {
        Self {
            allocate,
            deallocate,
        }
    }
}
/// Optional DX12 callback groups. Omission does not restore native defaults.
#[derive(Default)]
pub struct Dx12AllocationCallbacks {
    resources: Option<ResourceAllocationCallbacks>,
    heaps: Option<HeapAllocationCallbacks>,
    constant_buffer: Option<ConstantBufferAllocator>,
}
impl Dx12AllocationCallbacks {
    /// Replace the requested resource pair, without changing native registration.
    pub fn set_resources(&mut self, value: Option<ResourceAllocationCallbacks>) {
        self.resources = value;
    }
    /// Replace the requested heap pair, without changing native registration.
    pub fn set_heaps(&mut self, value: Option<HeapAllocationCallbacks>) {
        self.heaps = value;
    }
    /// Request a CB callback. Its first argument is payload, not userdata.
    pub fn set_constant_buffer_allocator(&mut self, value: Option<ConstantBufferAllocator>) {
        self.constant_buffer = value;
    }
    fn is_empty(&self) -> bool {
        self.resources.is_none() && self.heaps.is_none() && self.constant_buffer.is_none()
    }
}
/// Native creation requests. Neither Copy, Clone, Send nor Sync; not a guard.
///
/// Dropping this value performs no native operation and frees no target state.
#[derive(Default)]
pub struct CreationCallbacks {
    host: Option<HostAllocationCallbacks>,
    dx12: Dx12AllocationCallbacks,
    pub(crate) message: Option<MessageCallback>,
    _local: PhantomData<Rc<()>>,
}
impl CreationCallbacks {
    /// Select host callbacks for both context creation and destruction.
    pub fn set_host_allocator(&mut self, value: Option<HostAllocationCallbacks>) {
        self.host = value;
    }
    /// Select creation-extension groups; no group denotes an unregister request.
    pub fn set_dx12_allocations(&mut self, value: Dx12AllocationCallbacks) {
        self.dx12 = value;
    }
    /// Select the creation message pointer. Provider-specific routing may be global.
    pub fn set_message_callback(&mut self, value: Option<MessageCallback>) {
        self.message = value;
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.host.is_none() && self.dx12.is_empty() && self.message.is_none()
    }
    pub(crate) fn host_native(&self) -> Option<api::ffxAllocationCallbacks> {
        self.host.as_ref().map(|h| api::ffxAllocationCallbacks {
            pUserData: h.user_data,
            alloc: Some(h.allocate),
            dealloc: Some(h.deallocate),
        })
    }
    pub(crate) fn dx12_native(&self) -> Option<dx12::ffxCreateBackendDX12AllocationCallbacksDesc> {
        (!self.dx12.is_empty()).then(|| dx12::ffxCreateBackendDX12AllocationCallbacksDesc {
            header: api::ffxApiHeader {
                r#type: dx12::FFX_API_CREATE_CONTEXT_DESC_TYPE_BACKEND_DX12_ALLOCATION_CALLBACKS,
                pNext: null_mut(),
            },
            pfnFfxResourceAllocator: self.dx12.resources.as_ref().map(|p| p.allocate),
            pfnFfxResourceDeallocator: self.dx12.resources.as_ref().map(|p| p.deallocate),
            pfnFfxHeapAllocator: self.dx12.heaps.as_ref().map(|p| p.allocate),
            pfnFfxHeapDeallocator: self.dx12.heaps.as_ref().map(|p| p.deallocate),
            pfnFfxConstantBufferAllocator: self.dx12.constant_buffer,
        })
    }
}
