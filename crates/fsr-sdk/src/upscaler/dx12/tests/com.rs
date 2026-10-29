// SPDX-License-Identifier: MPL-2.0

//! Test-only COM objects for the public dispatch inspection path. No GPU work is recorded.
use super::super::*;
use std::ptr::null_mut;
use std::{
    cell::Cell,
    ffi::c_void,
    mem::{offset_of, size_of},
    sync::OnceLock,
};
use windows::core::{GUID, HRESULT};
use windows::{
    Win32::Graphics::{
        Direct3D12::{
            D3D12_COMMAND_LIST_TYPE, D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_FEATURE,
            D3D12_FEATURE_DATA_FORMAT_SUPPORT, D3D12_FEATURE_FORMAT_SUPPORT, D3D12_FORMAT_SUPPORT1,
            D3D12_FORMAT_SUPPORT1_SHADER_LOAD, D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE,
            D3D12_FORMAT_SUPPORT1_TEXTURE2D, D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW,
            D3D12_FORMAT_SUPPORT2, D3D12_FORMAT_SUPPORT2_UAV_TYPED_STORE, D3D12_RESOURCE_DESC,
            D3D12_RESOURCE_DIMENSION_TEXTURE2D, D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            D3D12_RESOURCE_FLAG_NONE, ID3D12CommandList_Vtbl, ID3D12Device_Vtbl,
            ID3D12DeviceChild_Vtbl, ID3D12GraphicsCommandList, ID3D12GraphicsCommandList_Vtbl,
            ID3D12Pageable_Vtbl, ID3D12Resource, ID3D12Resource_Vtbl,
        },
        Dxgi::Common::{
            DXGI_FORMAT, DXGI_FORMAT_R16G16_FLOAT, DXGI_FORMAT_R16G16B16A16_FLOAT,
            DXGI_FORMAT_R32_FLOAT, DXGI_SAMPLE_DESC,
        },
    },
    core::{IUnknown, Interface},
};

pub(super) const E_NOINTERFACE: HRESULT = HRESULT(0x8000_4002u32 as i32);
pub(super) const GET_DEVICE_FAILURE: HRESULT = HRESULT(0x887a_0005u32 as i32);
pub(super) const QUERY_FAILURE: HRESULT = HRESULT(0x8000_4005u32 as i32);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Device,
    List,
    Resource,
}

#[repr(C)]
pub(super) struct ComObject {
    vtable: *const usize,
    pub(super) references: Cell<u32>,
    kind: Kind,
    pub(super) canonical: Cell<*mut ComObject>,
    pub(super) device: Cell<*mut ComObject>,
    pub(super) desc: Cell<D3D12_RESOURCE_DESC>,
    pub(super) list_type: Cell<D3D12_COMMAND_LIST_TYPE>,
    pub(super) query_failure: Cell<Option<HRESULT>>,
    pub(super) get_device_failure: Cell<Option<HRESULT>>,
    pub(super) format_support: Cell<D3D12_FORMAT_SUPPORT1>,
    pub(super) format_support2: Cell<D3D12_FORMAT_SUPPORT2>,
    pub(super) format_support_overrides:
        std::cell::RefCell<Vec<(DXGI_FORMAT, D3D12_FORMAT_SUPPORT1, D3D12_FORMAT_SUPPORT2)>>,
    pub(super) format_queries: std::cell::RefCell<Vec<DXGI_FORMAT>>,
    pub(super) format_query_failure: Cell<Option<HRESULT>>,
    pub(super) format_query_calls: Cell<u32>,
    pub(super) typed_casting: Cell<bool>,
    pub(super) casting_query_calls: Cell<u32>,
    pub(super) casting_query_failure: Cell<Option<HRESULT>>,
    pub(super) format_fail_at: Cell<Option<DXGI_FORMAT>>,
}

impl ComObject {
    pub(super) fn set_device(&self, next: *mut ComObject) {
        // SAFETY: Both fixture devices are live; transfer the child's retained
        // reference before changing the device returned by GetDevice.
        unsafe {
            add_ref(next.cast());
            let previous = self.device.replace(next);
            release(previous.cast());
        }
    }

    pub(super) fn set_canonical(&self, next: *mut ComObject) {
        let own = std::ptr::from_ref(self).cast_mut();
        // SAFETY: An alias retains its canonical object. Self identity uses the
        // object's own reference and must not form a reference cycle.
        unsafe {
            if next != own {
                add_ref(next.cast());
            }
            let previous = self.canonical.replace(next);
            if previous != own {
                release(previous.cast());
            }
        }
    }
}

unsafe extern "system" fn unsupported_method() -> HRESULT {
    // All slots have a non-null function pointer. Inspection calls only the
    // correctly typed methods installed below; any other call is a test bug.
    std::process::abort()
}

unsafe extern "system" fn query_interface(
    this: *mut c_void,
    iid: *const GUID,
    out: *mut *mut c_void,
) -> HRESULT {
    super::performance::com_call(0);
    // SAFETY: The COM entry point supplies the implementation's live arguments.
    unsafe { query_interface_impl(this, iid, out) }
}

unsafe fn query_interface_impl(
    this: *mut c_void,
    iid: *const GUID,
    out: *mut *mut c_void,
) -> HRESULT {
    // SAFETY: The typed COM vtables pass live fixture pointers and valid outputs.
    unsafe {
        *out = null_mut();
        let object = &*this.cast::<ComObject>();
        let requested = &*iid;
        if requested == &IUnknown::IID {
            if let Some(error) = object.query_failure.get() {
                return error;
            }
            let canonical = object.canonical.get();
            add_ref_impl(canonical.cast());
            *out = canonical.cast();
        } else if (object.kind == Kind::Device && requested == &ID3D12Device::IID)
            || (object.kind == Kind::List && requested == &ID3D12GraphicsCommandList::IID)
            || (object.kind == Kind::Resource && requested == &ID3D12Resource::IID)
        {
            add_ref_impl(this);
            *out = this;
        } else {
            return E_NOINTERFACE;
        }
        HRESULT(0)
    }
}

unsafe extern "system" fn add_ref(this: *mut c_void) -> u32 {
    super::performance::com_call(1);
    // SAFETY: COM supplied a live object.
    unsafe { add_ref_impl(this) }
}

unsafe fn add_ref_impl(this: *mut c_void) -> u32 {
    // SAFETY: This pointer carries an outstanding COM reference.
    let object = unsafe { &*this.cast::<ComObject>() };
    let next = object.references.get() + 1;
    object.references.set(next);
    next
}

unsafe extern "system" fn release(this: *mut c_void) -> u32 {
    super::performance::com_call(2);
    // SAFETY: Every release balances an adopted or added COM reference.
    let object = unsafe { &*this.cast::<ComObject>() };
    let next = object
        .references
        .get()
        .checked_sub(1)
        .expect("balanced COM Release");
    object.references.set(next);
    if next == 0 {
        let device = if object.kind == Kind::Device {
            None
        } else {
            Some(object.device.get())
        };
        let canonical = object.canonical.get();
        let alias = canonical != this.cast::<ComObject>();
        // SAFETY: The zero count owns the final allocation reference.
        unsafe { drop(Box::from_raw(this.cast::<ComObject>())) };
        if let Some(device) = device {
            // SAFETY: Each child retained a device reference at construction.
            unsafe { release(device.cast()) };
        }
        if alias {
            // SAFETY: The alias retained its canonical object's reference.
            unsafe { release(canonical.cast()) };
        }
    }
    next
}

unsafe extern "system" fn get_device(
    this: *mut c_void,
    iid: *const GUID,
    out: *mut *mut c_void,
) -> HRESULT {
    super::performance::com_call(3);
    // SAFETY: GetDevice receives a live list/resource and writable output.
    unsafe {
        *out = null_mut();
        let object = &*this.cast::<ComObject>();
        if let Some(error) = object.get_device_failure.get() {
            return error;
        }
        query_interface_impl(object.device.get().cast(), iid, out)
    }
}

unsafe extern "system" fn get_type(this: *mut c_void) -> D3D12_COMMAND_LIST_TYPE {
    super::performance::com_call(4);
    // SAFETY: GetType is installed only on list objects.
    unsafe { (&*this.cast::<ComObject>()).list_type.get() }
}

unsafe extern "system" fn get_desc(this: *mut c_void, out: *mut D3D12_RESOURCE_DESC) {
    super::performance::com_call(5);
    // SAFETY: GetDesc is installed only on resources and receives writable storage.
    unsafe { *out = (&*this.cast::<ComObject>()).desc.get() };
}

unsafe extern "system" fn check_feature_support(
    this: *mut c_void,
    feature: D3D12_FEATURE,
    data: *mut c_void,
    size: u32,
) -> HRESULT {
    // SAFETY: This device slot receives live fixture storage and the initialized
    // feature structure from the production inspection path.
    let object = unsafe { &*this.cast::<ComObject>() };
    if feature == windows::Win32::Graphics::Direct3D12::D3D12_FEATURE_D3D12_OPTIONS3 {
        use windows::Win32::Graphics::Direct3D12::D3D12_FEATURE_DATA_D3D12_OPTIONS3;
        assert_eq!(
            size as usize,
            size_of::<D3D12_FEATURE_DATA_D3D12_OPTIONS3>()
        );
        object
            .casting_query_calls
            .set(object.casting_query_calls.get() + 1);
        if let Some(error) = object.casting_query_failure.get() {
            return error;
        }
        let data = unsafe { &mut *data.cast::<D3D12_FEATURE_DATA_D3D12_OPTIONS3>() };
        data.CastingFullyTypedFormatSupported = object.typed_casting.get().into();
        return HRESULT(0);
    }
    object
        .format_query_calls
        .set(object.format_query_calls.get() + 1);
    assert_eq!(feature, D3D12_FEATURE_FORMAT_SUPPORT);
    assert_eq!(
        size as usize,
        size_of::<D3D12_FEATURE_DATA_FORMAT_SUPPORT>()
    );
    let data = unsafe { &mut *data.cast::<D3D12_FEATURE_DATA_FORMAT_SUPPORT>() };
    object.format_queries.borrow_mut().push(data.Format);
    if let Some(error) = object.format_query_failure.get().filter(|_| {
        object
            .format_fail_at
            .get()
            .is_none_or(|format| format == data.Format)
    }) {
        return error;
    }
    data.Support1 = object.format_support.get();
    data.Support2 = object.format_support2.get();
    if let Some((_, support1, support2)) = object
        .format_support_overrides
        .borrow()
        .iter()
        .find(|(format, _, _)| *format == data.Format)
    {
        data.Support1 = *support1;
        data.Support2 = *support2;
    }
    HRESULT(0)
}

fn vtable(kind: Kind) -> *const usize {
    static DEVICE: OnceLock<Box<[usize]>> = OnceLock::new();
    static LIST: OnceLock<Box<[usize]>> = OnceLock::new();
    static RESOURCE: OnceLock<Box<[usize]>> = OnceLock::new();
    let cell = match kind {
        Kind::Device => &DEVICE,
        Kind::List => &LIST,
        Kind::Resource => &RESOURCE,
    };
    cell.get_or_init(|| {
        let bytes = match kind {
            Kind::Device => size_of::<ID3D12Device_Vtbl>(),
            Kind::List => size_of::<ID3D12GraphicsCommandList_Vtbl>(),
            Kind::Resource => size_of::<ID3D12Resource_Vtbl>(),
        };
        assert_eq!(bytes % size_of::<usize>(), 0);
        let mut slots = vec![unsupported_method as *const () as usize; bytes / size_of::<usize>()];
        slots[0] = query_interface as *const () as usize;
        slots[1] = add_ref as *const () as usize;
        slots[2] = release as *const () as usize;
        if kind != Kind::Device {
            let child_offset = if kind == Kind::List {
                offset_of!(ID3D12GraphicsCommandList_Vtbl, base__)
                    + offset_of!(ID3D12CommandList_Vtbl, base__)
            } else {
                offset_of!(ID3D12Resource_Vtbl, base__) + offset_of!(ID3D12Pageable_Vtbl, base__)
            };
            let get_device_slot =
                (child_offset + offset_of!(ID3D12DeviceChild_Vtbl, GetDevice)) / size_of::<usize>();
            slots[get_device_slot] = get_device as *const () as usize;
        }
        match kind {
            Kind::List => {
                slots[offset_of!(ID3D12CommandList_Vtbl, GetType) / size_of::<usize>()] =
                    get_type as *const () as usize
            }
            Kind::Resource => {
                slots[offset_of!(ID3D12Resource_Vtbl, GetDesc) / size_of::<usize>()] =
                    get_desc as *const () as usize
            }
            Kind::Device => {
                slots[offset_of!(ID3D12Device_Vtbl, CheckFeatureSupport) / size_of::<usize>()] =
                    check_feature_support as *const () as usize;
            }
        }
        slots.into_boxed_slice()
    })
    .as_ptr()
}

fn object(kind: Kind, device: *mut ComObject, desc: D3D12_RESOURCE_DESC) -> *mut ComObject {
    let pointer = Box::into_raw(Box::new(ComObject {
        vtable: vtable(kind),
        references: Cell::new(1),
        kind,
        canonical: Cell::new(null_mut()),
        device: Cell::new(device),
        desc: Cell::new(desc),
        list_type: Cell::new(D3D12_COMMAND_LIST_TYPE_DIRECT),
        query_failure: Cell::new(None),
        get_device_failure: Cell::new(None),
        format_support: Cell::new(
            D3D12_FORMAT_SUPPORT1_TEXTURE2D
                | D3D12_FORMAT_SUPPORT1_SHADER_LOAD
                | D3D12_FORMAT_SUPPORT1_SHADER_SAMPLE
                | D3D12_FORMAT_SUPPORT1_TYPED_UNORDERED_ACCESS_VIEW,
        ),
        format_query_failure: Cell::new(None),
        format_support2: Cell::new(D3D12_FORMAT_SUPPORT2_UAV_TYPED_STORE),
        format_support_overrides: Default::default(),
        format_queries: Default::default(),
        format_query_calls: Cell::new(0),
        typed_casting: Cell::new(true),
        casting_query_calls: Cell::new(0),
        casting_query_failure: Cell::new(None),
        format_fail_at: Cell::new(None),
    }));
    // SAFETY: This Box has one initial COM reference, and a child retains its device.
    unsafe {
        (*pointer).canonical.set(pointer);
        if kind != Kind::Device {
            add_ref(device.cast());
        }
    }
    pointer
}

fn resource_desc(
    format: DXGI_FORMAT,
    width: u64,
    height: u32,
    output: bool,
) -> D3D12_RESOURCE_DESC {
    D3D12_RESOURCE_DESC {
        Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
        Width: width,
        Height: height,
        DepthOrArraySize: 1,
        MipLevels: 1,
        Format: format,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Flags: if output {
            D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS
        } else {
            D3D12_RESOURCE_FLAG_NONE
        },
        ..Default::default()
    }
}

// Each windows interface adopts its object's initial COM reference. Clones and
// QI results are balanced by Release; zero references reclaim the allocation.
unsafe fn adopt<T: Interface>(pointer: *mut ComObject) -> T {
    unsafe { T::from_raw(pointer.cast()) }
}

pub(super) struct Scene {
    pub(super) device: ID3D12Device,
    pub(super) foreign_device: ID3D12Device,
    pub(super) list: ID3D12GraphicsCommandList,
    pub(super) resources: [ID3D12Resource; 5],
}

impl Scene {
    pub(super) fn new() -> Self {
        let device = object(Kind::Device, null_mut(), D3D12_RESOURCE_DESC::default());
        let foreign = object(Kind::Device, null_mut(), D3D12_RESOURCE_DESC::default());
        let list = object(Kind::List, device, D3D12_RESOURCE_DESC::default());
        let specs = [
            (DXGI_FORMAT_R16G16B16A16_FLOAT, 1280, 720, false),
            (DXGI_FORMAT_R32_FLOAT, 1280, 720, false),
            (DXGI_FORMAT_R16G16_FLOAT, 1280, 720, false),
            (DXGI_FORMAT_R32_FLOAT, 1, 1, false),
            (DXGI_FORMAT_R16G16B16A16_FLOAT, 1920, 1080, true),
        ];
        let resources = specs.map(|(format, width, height, output)| {
            let pointer = object(
                Kind::Resource,
                device,
                resource_desc(format, width, height, output),
            );
            // SAFETY: The new COM object transfers its initial reference.
            unsafe { adopt(pointer) }
        });
        Self {
            // SAFETY: Each newly allocated object transfers its initial reference.
            device: unsafe { adopt(device) },
            foreign_device: unsafe { adopt(foreign) },
            list: unsafe { adopt(list) },
            resources,
        }
    }

    pub(super) fn mask(&self) -> ID3D12Resource {
        let pointer = object(
            Kind::Resource,
            self.device.as_raw().cast(),
            resource_desc(
                windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R8_UNORM,
                1280,
                720,
                false,
            ),
        );
        // SAFETY: The new resource transfers its initial COM reference.
        unsafe { adopt(pointer) }
    }

    pub(super) fn input(&self) -> UpscaleTestInputs<'_> {
        UpscaleTestInputs::new(
            &self.list,
            UpscaleResources::builder()
                .color(&self.resources[0])
                .depth(&self.resources[1])
                .motion_vectors(&self.resources[2])
                .exposure(crate::upscaler::dx12::ExposureInput::Texture(
                    &self.resources[3],
                ))
                .output(&self.resources[4])
                .build()
                .unwrap(),
            JitterOffsetPixels::new(0.0, 0.0).unwrap(),
            FrameTimeMillis::new(16.0).unwrap(),
            CameraParameters::new(0.1, 1000.0, 1.0, 1.0).unwrap(),
            true,
        )
    }

    pub(super) fn list_object(&self) -> &ComObject {
        object_from(&self.list)
    }
    pub(super) fn resource_object(&self, index: usize) -> &ComObject {
        object_from(&self.resources[index])
    }
    pub(super) fn device_object(&self) -> &ComObject {
        object_from(&self.device)
    }

    pub(super) fn reference_counts(&self) -> [u32; 8] {
        [
            self.device_object().references.get(),
            object_from(&self.foreign_device).references.get(),
            self.list_object().references.get(),
            self.resource_object(0).references.get(),
            self.resource_object(1).references.get(),
            self.resource_object(2).references.get(),
            self.resource_object(3).references.get(),
            self.resource_object(4).references.get(),
        ]
    }
}

pub(super) fn object_from<T: Interface>(value: &T) -> &ComObject {
    // SAFETY: Every interface in Scene owns a live reference to its ComObject.
    unsafe { &*value.as_raw().cast::<ComObject>() }
}
