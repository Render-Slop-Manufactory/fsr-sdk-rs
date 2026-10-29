// SPDX-License-Identifier: MPL-2.0

//! Renderer-owned generation → UAV ordering/transition → supplied-mask upscale.
//! `main` starts no native or GPU work. Options are supplied by the renderer.

#[cfg(all(windows, feature = "dx12"))]
pub mod renderer_integration {
    use fsr_sdk::{
        error::Error,
        upscaler::{
            FrameParameters, ID3D12GraphicsCommandList, ReactiveMaskParameters,
            ReactiveMaskResources, UpscaleResources, Upscaler,
            dx12::{
                D3D12_RESOURCE_BARRIER, D3D12_RESOURCE_BARRIER_0,
                D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES, D3D12_RESOURCE_BARRIER_FLAG_NONE,
                D3D12_RESOURCE_BARRIER_TYPE_TRANSITION, D3D12_RESOURCE_BARRIER_TYPE_UAV,
                D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                D3D12_RESOURCE_STATE_UNORDERED_ACCESS, D3D12_RESOURCE_TRANSITION_BARRIER,
                D3D12_RESOURCE_UAV_BARRIER,
            },
        },
    };
    use std::mem::ManuallyDrop;

    /// Record generation and upscaling in order on one list, without a CPU wait.
    ///
    /// `resources.color()` must be the generator's full colour input; the frame's
    /// signals must match `jitter`, camera and timing. This example uses the
    /// default linear/exposure/motion profile with no sharpening/pre-exposure
    /// override. The active render region is taken from `generation` for both
    /// operations. Required transparency/composition input stays renderer-owned.
    ///
    /// # Safety
    ///
    /// Uphold both public recording contracts, including actual-provider support
    /// and native non-retention/state-restoration trust, matching signals, correct
    /// initial states, residency, live list/allocator and non-overlapping output
    /// storage. Enter with generator inputs and other upscale inputs shader-readable,
    /// reactive output in UAV state and upscale output in UAV state. Prior readers
    /// of the reactive texture must have finished. The generator inputs may alias.
    ///
    /// On OK, close/submit this list once, signal/wait a renderer-owned fence and
    /// retain all GPU dependencies through completion. Output mask ends shader-readable;
    /// transition it back to UAV only after earlier consumers finish. Restore
    /// application compute bindings/heaps as needed. Before safe `request_tuning`,
    /// complete work from both operations with no executable pending recording.
    /// The `dx12_upscale` example shows a renderer-supplied submit/wait callback.
    ///
    /// A native error requires invalidating the failed list, completing earlier
    /// submitted work and terminal context cleanup. Do not submit partial failed
    /// recordings. Later preflight errors do not undo earlier generation: either
    /// satisfy its successful-recording duties or abandon the list and terminally
    /// clean up every affected context. On callback error/panic or device loss,
    /// keep ownership until renderer cleanup is safe; `dx12_recovery` demonstrates
    /// retention in `Option<Upscaler>`. This function never destroys the context.
    pub unsafe fn record_with_generated_mask<'a>(
        upscaler: &mut Upscaler,
        list: &'a ID3D12GraphicsCommandList,
        generation_resources: &ReactiveMaskResources<'a>,
        generation: &ReactiveMaskParameters,
        mut resources: UpscaleResources<'a>,
        frame: &FrameParameters,
    ) -> Result<(), Error> {
        let mask = generation_resources.output();
        resources.set_reactive_mask(Some(mask));
        let mut frame = frame.clone();
        frame.set_active_render_size(Some(generation.render_size()));
        // Optional metadata checks before any generation; they do not check states
        // or capability, and both recording calls still inspect independently.
        upscaler.validate_reactive_mask_inputs(list, generation_resources, generation)?;
        upscaler.validate_upscale_inputs(list, &resources, &frame)?;
        // SAFETY: Renderer establishes generation's state/domain/lifetime contract.
        unsafe {
            upscaler.record_reactive_mask(list, generation_resources, generation)?;
        }

        // These two temporary COM clones are released immediately after recording
        // barriers; the renderer still retains the real mask through GPU completion.
        let mut barriers = [
            D3D12_RESOURCE_BARRIER {
                Type: D3D12_RESOURCE_BARRIER_TYPE_UAV,
                Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                    UAV: ManuallyDrop::new(D3D12_RESOURCE_UAV_BARRIER {
                        pResource: ManuallyDrop::new(Some(mask.clone())),
                    }),
                },
            },
            D3D12_RESOURCE_BARRIER {
                Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                    Transition: ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                        pResource: ManuallyDrop::new(Some(mask.clone())),
                        Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                        StateBefore: D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
                        StateAfter: D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
                    }),
                },
            },
        ];
        // SAFETY: Both tagged union members match their barrier types. Native
        // recording consumes them synchronously, then each COM clone is released
        // exactly once. Successful generation restores the declared UAV boundary
        // under the caller-established provider trust.
        unsafe {
            list.ResourceBarrier(&barriers);
            ManuallyDrop::drop(&mut (*barriers[0].Anonymous.UAV).pResource);
            ManuallyDrop::drop(&mut (*barriers[1].Anonymous.Transition).pResource);
        }
        // SAFETY: Same frame/region/list, ordered mask writes and shader-read
        // transition; caller retains submission/fencing and all other obligations.
        unsafe { upscaler.record(list, &resources, &frame) }
    }
}

fn main() {
    println!("DX12 generated-mask integration reference; no runtime or GPU work is started.");
}
