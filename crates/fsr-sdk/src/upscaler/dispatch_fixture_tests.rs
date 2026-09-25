// SPDX-License-Identifier: MPL-2.0

//! Test-only COM objects for the public dispatch inspection path. No GPU work is recorded.
use super::*;
use crate::error::{
    DispatchValidation, InspectionObject, InspectionOperation, ResourceProperty, ResourceRole,
};
use std::{
    cell::Cell,
    ffi::c_void,
    mem::{offset_of, size_of},
    sync::OnceLock,
};
use windows::{
    Win32::Graphics::{
        Direct3D12::{
            D3D12_COMMAND_LIST_TYPE, D3D12_COMMAND_LIST_TYPE_COMPUTE,
            D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_RESOURCE_DESC,
            D3D12_RESOURCE_DIMENSION_TEXTURE2D, D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS,
            D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE, D3D12_RESOURCE_FLAG_NONE,
            ID3D12CommandList_Vtbl, ID3D12Device_Vtbl, ID3D12DeviceChild_Vtbl,
            ID3D12GraphicsCommandList, ID3D12GraphicsCommandList_Vtbl, ID3D12Pageable_Vtbl,
            ID3D12Resource, ID3D12Resource_Vtbl,
        },
        Dxgi::Common::{
            DXGI_FORMAT, DXGI_FORMAT_R16G16_FLOAT, DXGI_FORMAT_R16G16B16A16_FLOAT,
            DXGI_FORMAT_R32_FLOAT, DXGI_SAMPLE_DESC,
        },
    },
    core::{IUnknown, Interface},
};

const E_NOINTERFACE: HRESULT = HRESULT(0x8000_4002u32 as i32);
const GET_DEVICE_FAILURE: HRESULT = HRESULT(0x887a_0005u32 as i32);
const QUERY_FAILURE: HRESULT = HRESULT(0x8000_4005u32 as i32);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Device,
    List,
    Resource,
}

#[repr(C)]
struct ComObject {
    vtable: *const usize,
    references: Cell<u32>,
    kind: Kind,
    canonical: Cell<*mut ComObject>,
    device: Cell<*mut ComObject>,
    desc: Cell<D3D12_RESOURCE_DESC>,
    list_type: Cell<D3D12_COMMAND_LIST_TYPE>,
    query_failure: Cell<Option<HRESULT>>,
    get_device_failure: Cell<Option<HRESULT>>,
}

impl ComObject {
    fn set_device(&self, next: *mut ComObject) {
        // SAFETY: Both fixture devices are live; transfer the child's retained
        // reference before changing the device returned by GetDevice.
        unsafe {
            add_ref(next.cast());
            let previous = self.device.replace(next);
            release(previous.cast());
        }
    }

    fn set_canonical(&self, next: *mut ComObject) {
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
            add_ref(canonical.cast());
            *out = canonical.cast();
        } else if (object.kind == Kind::Device && requested == &ID3D12Device::IID)
            || (object.kind == Kind::List && requested == &ID3D12GraphicsCommandList::IID)
            || (object.kind == Kind::Resource && requested == &ID3D12Resource::IID)
        {
            add_ref(this);
            *out = this;
        } else {
            return E_NOINTERFACE;
        }
        HRESULT(0)
    }
}

unsafe extern "system" fn add_ref(this: *mut c_void) -> u32 {
    // SAFETY: This pointer carries an outstanding COM reference.
    let object = unsafe { &*this.cast::<ComObject>() };
    let next = object.references.get() + 1;
    object.references.set(next);
    next
}

unsafe extern "system" fn release(this: *mut c_void) -> u32 {
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
    // SAFETY: GetDevice receives a live list/resource and writable output.
    unsafe {
        *out = null_mut();
        let object = &*this.cast::<ComObject>();
        if let Some(error) = object.get_device_failure.get() {
            return error;
        }
        query_interface(object.device.get().cast(), iid, out)
    }
}

unsafe extern "system" fn get_type(this: *mut c_void) -> D3D12_COMMAND_LIST_TYPE {
    // SAFETY: GetType is installed only on list objects.
    unsafe { (&*this.cast::<ComObject>()).list_type.get() }
}

unsafe extern "system" fn get_desc(this: *mut c_void, out: *mut D3D12_RESOURCE_DESC) {
    // SAFETY: GetDesc is installed only on resources and receives writable storage.
    unsafe { *out = (&*this.cast::<ComObject>()).desc.get() };
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
            Kind::Device => {}
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

struct Scene {
    device: ID3D12Device,
    foreign_device: ID3D12Device,
    list: ID3D12GraphicsCommandList,
    resources: [ID3D12Resource; 5],
}

impl Scene {
    fn new() -> Self {
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

    fn input(&self) -> UpscaleDispatch<'_> {
        UpscaleDispatch {
            command_list: &self.list,
            color: &self.resources[0],
            depth: &self.resources[1],
            motion_vectors: &self.resources[2],
            exposure: &self.resources[3],
            output: &self.resources[4],
            jitter: JitterOffsetPixels::new(0.0, 0.0).unwrap(),
            frame_time: FrameTimeMillis::new(16.0).unwrap(),
            camera: CameraParameters::new(0.1, 1000.0, 1.0, 1.0).unwrap(),
            reset: true,
        }
    }

    fn list_object(&self) -> &ComObject {
        object_from(&self.list)
    }
    fn resource_object(&self, index: usize) -> &ComObject {
        object_from(&self.resources[index])
    }
    fn device_object(&self) -> &ComObject {
        object_from(&self.device)
    }

    fn reference_counts(&self) -> [u32; 8] {
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

fn object_from<T: Interface>(value: &T) -> &ComObject {
    // SAFETY: Every interface in Scene owns a live reference to its ComObject.
    unsafe { &*value.as_raw().cast::<ComObject>() }
}

fn observation(observer: &libloading::Library) -> DispatchObservation {
    // SAFETY: The locally compiled runtime fixture exports this exact signature.
    unsafe {
        observer
            .get::<extern "C" fn() -> DispatchObservation>(c"fixture_dispatch_observation")
            .unwrap()()
    }
}

fn invalid(reason: DispatchValidation) -> Error {
    Error::InvalidDispatch { reason }
}
fn resource_error(role: ResourceRole, property: ResourceProperty) -> Error {
    invalid(DispatchValidation::Resource { role, property })
}

#[test]
fn public_dispatch_inspects_com_and_recovers_after_each_preflight_error() {
    let (runtime, observer) = fixture_runtime();
    let scene = Scene::new();
    let mut upscaler = Upscaler::new(&runtime, &scene.device, sizes()).unwrap();
    let mut calls = 0;
    let mut check = |expected: Error, reset: &dyn Fn()| {
        let before = scene.reference_counts();
        // SAFETY: The fixture runtime records only the descriptor; no GPU work.
        assert_eq!(unsafe { upscaler.dispatch(scene.input()) }, Err(expected));
        assert_eq!(observation(&observer).calls, calls);
        assert_eq!(scene.reference_counts(), before);
        reset();
        let before = scene.reference_counts();
        // SAFETY: The fixture runtime records no GPU commands.
        assert_eq!(unsafe { upscaler.dispatch(scene.input()) }, Ok(()));
        calls += 1;
        assert_eq!(observation(&observer).calls, calls);
        assert_eq!(scene.reference_counts(), before);
    };

    scene
        .list_object()
        .list_type
        .set(D3D12_COMMAND_LIST_TYPE_COMPUTE);
    check(invalid(DispatchValidation::CommandListType), &|| {
        scene
            .list_object()
            .list_type
            .set(D3D12_COMMAND_LIST_TYPE_DIRECT)
    });

    scene
        .list_object()
        .set_device(scene.foreign_device.as_raw().cast());
    check(invalid(DispatchValidation::CommandListDevice), &|| {
        scene.list_object().set_device(scene.device.as_raw().cast())
    });

    scene
        .resource_object(2)
        .set_device(scene.foreign_device.as_raw().cast());
    check(
        resource_error(ResourceRole::MotionVectors, ResourceProperty::Device),
        &|| {
            scene
                .resource_object(2)
                .set_device(scene.device.as_raw().cast())
        },
    );

    // Different ID3D12Resource pointers report the same canonical IUnknown.
    assert_ne!(scene.resources[0].as_raw(), scene.resources[1].as_raw());
    scene
        .resource_object(1)
        .set_canonical(scene.resources[0].as_raw().cast());
    check(
        resource_error(ResourceRole::Depth, ResourceProperty::DistinctIdentity),
        &|| {
            scene
                .resource_object(1)
                .set_canonical(scene.resources[1].as_raw().cast())
        },
    );

    scene
        .list_object()
        .get_device_failure
        .set(Some(GET_DEVICE_FAILURE));
    check(
        Error::Dx12Inspection {
            object: InspectionObject::CommandList,
            operation: InspectionOperation::GetDevice,
            hresult: GET_DEVICE_FAILURE.0,
        },
        &|| scene.list_object().get_device_failure.set(None),
    );

    scene
        .resource_object(4)
        .get_device_failure
        .set(Some(GET_DEVICE_FAILURE));
    check(
        Error::Dx12Inspection {
            object: InspectionObject::Resource(ResourceRole::Output),
            operation: InspectionOperation::GetDevice,
            hresult: GET_DEVICE_FAILURE.0,
        },
        &|| scene.resource_object(4).get_device_failure.set(None),
    );

    scene
        .resource_object(2)
        .query_failure
        .set(Some(QUERY_FAILURE));
    check(
        Error::Dx12Inspection {
            object: InspectionObject::Resource(ResourceRole::MotionVectors),
            operation: InspectionOperation::QueryIdentity,
            hresult: QUERY_FAILURE.0,
        },
        &|| scene.resource_object(2).query_failure.set(None),
    );

    scene.device_object().query_failure.set(Some(QUERY_FAILURE));
    check(
        Error::Dx12Inspection {
            object: InspectionObject::ContextDevice,
            operation: InspectionOperation::QueryIdentity,
            hresult: QUERY_FAILURE.0,
        },
        &|| scene.device_object().query_failure.set(None),
    );

    let original = scene.resource_object(0).desc.get();
    let mut wrong = original;
    wrong.Width = 1279;
    scene.resource_object(0).desc.set(wrong);
    check(
        resource_error(ResourceRole::Color, ResourceProperty::Extent),
        &|| scene.resource_object(0).desc.set(original),
    );

    let original = scene.resource_object(1).desc.get();
    let mut denied = original;
    denied.Flags = D3D12_RESOURCE_FLAG_DENY_SHADER_RESOURCE;
    scene.resource_object(1).desc.set(denied);
    check(
        resource_error(ResourceRole::Depth, ResourceProperty::ShaderReadable),
        &|| scene.resource_object(1).desc.set(original),
    );

    let original = scene.resource_object(4).desc.get();
    let mut no_uav = original;
    no_uav.Flags = D3D12_RESOURCE_FLAG_NONE;
    scene.resource_object(4).desc.set(no_uav);
    check(
        resource_error(ResourceRole::Output, ResourceProperty::UnorderedAccess),
        &|| scene.resource_object(4).desc.set(original),
    );

    assert_eq!(
        observation(&observer).tag,
        FFX_API_DISPATCH_DESC_TYPE_UPSCALE
    );
    upscaler.destroy().unwrap();
}
