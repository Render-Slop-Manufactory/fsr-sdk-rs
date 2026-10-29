# Upscaler setup and composition spike

Bounded U07/early-U08 experiment for the
[contract draft](../../../docs/design/upscaler-setup.md). This is experiment
equipment, not another public library or a production API implementation.
The [experiment record](../../../docs/research/records/2026-10-26-exp-upscaler-setup-contract-spike.md)
contains the tested identities, observations, unsuccessful preparation and limits.

From the repository root on Windows x64/MSVC:

```sh
python tools/experiments/upscaler_setup/run.py
```

Prerequisites: Python 3.11+, working Rust/MSVC linking, installed `stable` and
`1.88.0` toolchains, and the workspace's existing dependencies in the Cargo cache.
All Cargo commands use `--offline`; no SDK, AMD DLL, GPU or additional dependency
is required. `--toolchain stable` or `--toolchain 1.88.0` selects one toolchain;
`--prepare-only` materializes without compiling or running the candidate.

The coordinator creates a fresh isolated workspace beneath
`target/upscaler-setup-spike/`, recording input hashes before applying its explicit
patches. It copies the existing two crates there, adds experiment modules and
builds immutable external consumer targets. The production files and workspace
membership stay unchanged. Repeated runs create distinct directories; the runner
does not delete earlier evidence.
Generated candidate dev/test profiles disable debug information and incremental
compilation to bound temporary disk use; production profiles remain unchanged.

- `setup.rs` narrows the experimental loaded Runtime and introduces unsafe
  effect acquisition, borrowed setup and owned context wrappers. Private bridges
  reuse production routing and context ownership after effect trust is acquired.
- `composition.rs` provides portable parameters, a provider-free configuration
  adapter and named upscale resource binding. It privately assembles the current
  descriptor input so production inspection/mapping actually runs.
- `controlled.rs` uses the existing controlled DLL, fake COM scene and query
  observers. These implement no GPU work and no AMD provider behaviour.
- `run.py` materializes the candidate and five successful consumer targets plus
  twenty expected compiler failures per toolchain. Expected failures require
  the specific diagnostic on the corresponding consumer target; an arbitrary
  dependency/build failure cannot satisfy them.
  The coordinator also runs strict stable library Clippy with `-D warnings`.

The coordinator writes individual logs, `inputs.json` and `results.json` into
the run directory. It distinguishes successful checks from expected nonzero
compiler exits. Selected panic tests intentionally unwind inside `catch_unwind`;
their cleanup assertions must still pass. Exceptional native-create/teardown
retention intentionally leaves bounded test-only bundles alive until process exit.

No ordinary CI/release profile is extended. The reserved Vulkan failure, final
API migration, U09 tuning request, U10 error taxonomy and full generator resource
builder remain outside this spike. Keep the final native/package verification
separate from these source/compile/controlled results.
