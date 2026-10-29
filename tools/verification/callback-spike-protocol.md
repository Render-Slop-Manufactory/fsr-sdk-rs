# Callback ABI and native ownership spike

This direct-SDK experiment answers R03/U04's remaining bounded questions before
the API decision. It changes no production bindings or public contract. Run on
Windows x64/MSVC with the trusted pinned SDK, Rust stable and exact 1.88.0:

```sh
python tools/verification/callback_spike.py --native --sdk-root external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX --adapter-index 0
```

The selected adapter index must identify the intended AMD hardware. The script
reuses the coordinator's environment, pinned-header/signature checks, GPU helper,
watchdogs and input/evidence inventories. It downloads nothing. It is excluded
from ordinary/release profiles. Build products and verbose evidence stay under
`target/verification`; permanent findings belong in a dated experiment record.

## Questions and fixed sequence

1. Independently compare the SDK C++ aggregate against the Rust callback type,
   reusing the existing raw Rust resource declaration. Compare size, alignment,
   offsets and every field without comparing padding. Exchange four argument
   values in both callback directions on stable and Rust 1.88; a deliberately
   corrupted field must be rejected by the Rust consumer. Large argument values
   test integer transport only; neither fixture dereferences them as slice sizes.
2. On the signed loader, set global diagnostics for upscaling, then another effect,
   then the deprecated descriptor, then null. A well-formed unknown query header
   (`0x00fefffe`) supplies an unsupported-query stimulus. Record its actual
   error code and delivery separately from setter success; an unknown-descriptor
   error without a message does not exercise the source's no-provider diagnostic.
   The other-effect request deliberately supplies an unknown effect ID; preserve
   a rejection code rather than requiring the open source's ignored-field outcome.
3. On selected providers 3.1.5 and 4.1.1, run a default-DX12-allocation control
   with custom host allocation and creation messages. Create with checking and
   visualization flags (`640`), query a matching pre-create estimate, then record
   two individually fenced frames, including one debug-view request. Before the
   second frame, install a distinct loader-global message callback; observe
   whether it affects the live provider's warning route.
4. In a fresh process per provider, enable the upscaling-global warnings route,
   then create A with callback set 1; frame A; acquire
   the same loader image again; create B with set 2; frame A and B; create/frame C
   with an omitted allocation extension; create/frame D with incomplete pairs
   from set 1 and no CB callback but a third distinct message callback; destroy
   D and B; frame A; destroy C and A. A message from callback 3 after D's destroy
   distinguishes registration survival from context A's original callback.

All context descriptors have stable stack storage until native destruction.
All host userdata and callback functions are static. Both callback sets use the
same device and allocation protocols, so replacement can be observed without
mismatched frees. The heap callback returns offset zero; this tests no renderer
suballocator. CB payloads are copied to separate 256-byte-aligned upload backing,
retained through GPU completion and context destruction. Device ownership remains
retained through final application loader release. Callbacks never reenter the
SDK. Short event locks do not span native calls; they support logging, not SDK
thread safety or callback routing. Native threads receive semantic numeric labels.

The valid positive `0.5 ms` frame delta provides a diagnostic warning stimulus in
the inspected open checker; callback delivery from a signed provider remains an
observation. Messages are copied into bounded UTF-16 hash/length measurements
during invocation; no native string pointer is retained. The probe records code,
callback set, operation phase, effect/severity, depth, thread and allocation ID.
Resource events report allocation-info bytes; heap free reports the native logical
size, which may differ from the allocated heap size. Host frees verify userdata
and allocation identity. Resource/heap frees record whether another set allocated
the object; uninstrumented objects are labelled allocation zero, then released
using the same COM protocol. No artificial allocation failure is injected.

## Evaluation and limits

Build/ABI assertions, actual provider identities, successful create/destroy,
complete frame/cleanup markers, all 230,400 output texels written and finite,
completed fences, clean ordinary debug/device checks and zero remaining tracked
host/resource/heap blocks must pass. Callback absence remains `not_observed`;
successful registration is not a callback coverage claim. Routing differences
from source expectations are observations, not silently relabelled guarantees.
Every unsuccessful attempt is retained. No failed-create handle is operated on,
no failed destruction is retried and no failed recording is submitted. Fatal
probe failures exit without unwinding caller-owned callback state.

This establishes neither per-context callback identity nor native quiescence,
absence of workers/reentry, arbitrary concurrency, signed-provider rollback,
recoverable OOM, nonzero heap offsets, Rust wrapper integration or a safe closure
adapter. GPU checks establish integration only; there are no image-quality,
algorithm, provider-output equality or performance criteria. The spike does not
accept a public API or extend support to other systems.
