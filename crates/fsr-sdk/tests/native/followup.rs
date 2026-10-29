// SPDX-License-Identifier: MPL-2.0

//! Shared equipment for the bounded Configure and reactive-generation probes.
use super::*;

pub(super) struct Lab {
    pub device: ID3D12Device,
    pub loader: PathBuf,
    pub library: libloading::Library,
    create: CreateHarness,
    finish: Finish,
    destroy: Destroy,
    discard: Destroy,
    check: DeviceCheck,
}

pub(super) struct Frame {
    pub handle: *mut c_void,
    pub list: Option<ID3D12GraphicsCommandList>,
    pub resources: Vec<ID3D12Resource>,
}

impl Frame {
    pub fn input(&self, reset: bool) -> UpscaleTestInputs<'_> {
        UpscaleTestInputs::new(
            self.list.as_ref().unwrap(),
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
            JitterOffsetPixels::new(0.25, -0.25).unwrap(),
            FrameTimeMillis::new(16.667).unwrap(),
            CameraParameters::new(0.1, 100.0, std::f32::consts::FRAC_PI_3, 1.0).unwrap(),
            reset,
        )
    }
}

impl Lab {
    pub fn acquire() -> Self {
        assert!(
            std::env::var_os("FSR_SDK_TEST_GBV").is_none(),
            "GBV is a separate stalled protocol"
        );
        let loader = PathBuf::from(std::env::var_os("FSR_SDK_TEST_DLL").expect("trusted loader"));
        let path = std::env::var_os("FSR_SDK_TEST_GPU_DLL").expect("paired native helper");
        // SAFETY: Opt-in runner verifies the deployment and builds these exact exports.
        unsafe {
            let library = libloading::Library::new(path).unwrap();
            assert_eq!(
                library.get::<EnableDebug>(c"gpu_enable_debug").unwrap()(),
                0
            );
            let mut raw = null_mut();
            assert_eq!(
                library.get::<CreateDevice>(c"probe_create_device").unwrap()(&mut raw),
                0
            );
            assert!(!raw.is_null());
            let device = ID3D12Device::from_raw(raw);
            assert_eq!(
                library.get::<DeviceCheck>(c"gpu_require_amd").unwrap()(device.as_raw()),
                0
            );
            Self {
                device,
                loader,
                create: *library.get::<CreateHarness>(c"gpu_create").unwrap(),
                finish: *library.get::<Finish>(c"gpu_finish").unwrap(),
                destroy: *library.get::<Destroy>(c"gpu_destroy").unwrap(),
                discard: *library.get::<Destroy>(c"gpu_discard_recording").unwrap(),
                check: *library.get::<DeviceCheck>(c"gpu_check_device").unwrap(),
                library,
            }
        }
    }

    pub fn open(&self) -> Frame {
        let mut handle = null_mut();
        let mut list = null_mut();
        let mut resources = [null_mut(); 5];
        // SAFETY: Writable outputs; the paired helper transfers independent COM refs.
        unsafe {
            assert_eq!(
                (self.create)(
                    self.device.as_raw(),
                    &mut handle,
                    &mut list,
                    resources.as_mut_ptr()
                ),
                0
            );
            assert!(!handle.is_null() && !list.is_null());
            Frame {
                handle,
                list: Some(ID3D12GraphicsCommandList::from_raw(list)),
                resources: resources
                    .into_iter()
                    .map(|raw| {
                        assert!(!raw.is_null());
                        ID3D12Resource::from_raw(raw)
                    })
                    .collect(),
            }
        }
    }

    pub fn finish(&self, frame: &Frame, expect_output: bool) {
        let mut metrics = Metrics::default();
        // SAFETY: Caller retains the FSR context and frame through this submit/fence.
        let hr = unsafe { (self.finish)(frame.handle, &mut metrics) };
        if hr < 0 || metrics.device_removed != 0 {
            eprintln!("FOLLOWUP GPU COMPLETION FAILURE: hr={hr:#x}; metrics={metrics:?}");
            std::process::exit(1); // No unwinding while completion is uncertain.
        }
        eprintln!("Follow-up GPU readback: {metrics:?}");
        assert_eq!((metrics.width, metrics.height), (640, 360));
        assert_eq!(metrics.finite_pixels, 640 * 360);
        assert_eq!(metrics.debug_errors, 0);
        if expect_output {
            assert!(metrics.changed_pixels > 640 * 360 / 2);
        } else {
            assert_eq!(metrics.changed_pixels, 0);
        }
    }

    pub fn discard_unsubmitted(&self, frame: &mut Frame) {
        // This is only used before this fresh frame has ever been submitted.
        drop(frame.list.take());
        // SAFETY: Release the helper's remaining list reference, retaining textures
        // and allocator until terminal context destruction. There is no submission.
        unsafe { (self.discard)(frame.handle) };
    }

    pub fn release(&self, frame: Frame) {
        drop(frame.list);
        drop(frame.resources);
        // SAFETY: Caller fenced this frame or discarded it before any submission.
        unsafe { (self.destroy)(frame.handle) };
    }

    pub fn check(&self) {
        // SAFETY: All submitted work is complete and contexts have been destroyed.
        assert_eq!(unsafe { (self.check)(self.device.as_raw()) }, 0);
    }
}
