# External

This directory is an optional local SDK input for maintainer checks and native
experiments. Ordinary Cargo builds need no installed SDK and do not download one.

Expected local layout:

```text
external/FidelityFX-SDK/v2.3.0/
```

Use the pinned baseline from [D002](../docs/adr/D002.md) and the setup and
verification commands in [development](../docs/DEVELOPMENT.md#common-abi-verification).
Runtime loading and provider deployment follow [D007](../docs/adr/D007.md);
placing files here does not make them discoverable by an application or bundle
them in either Rust crate.
