# Public 0.1.0 publication check — 2026-09-29

Both crates were published on 2026-09-28 in sys-then-wrapper order. Their public
registry entries are not yanked. The GitHub `v0.1.0` release is public, with
`draft = false` and `prerelease = false`.

## Registry and release observations

The public sparse-index entries returned the following values:

| Crate | Version | Published (UTC) | Registry archive SHA-256 |
|---|---|---|---|
| [`fsr-sdk-sys`](https://crates.io/crates/fsr-sdk-sys) | `0.1.0` | `2026-09-28T19:30:01Z` | `a7e0ba27a9cc3b95dcb8c4d26c37bc04b2961babd6cd089d63ecde0e705f734e` |
| [`fsr-sdk`](https://crates.io/crates/fsr-sdk) | `0.1.0` | `2026-09-28T19:34:03Z` | `4a20a64583400cbc39cadb72b73d05c7ba6e6367eedb91fc39ae203aa20a5314` |

The wrapper entry declares `fsr-sdk-sys ^0.1.0` with default features disabled,
and enables `dx12` by default. The sys entry has no default features.

The [GitHub release](https://github.com/Render-Slop-Manufactory/fsr-sdk-rs/releases/tag/v0.1.0),
**v0.1.0: escaping containment**, reports tag `v0.1.0` and publication time
`2026-09-28T19:47:00Z`.

## Hosted documentation

Both versioned Rustdoc pages returned successfully and contain Windows/DX12 APIs:

- [`fsr-sdk 0.1.0`](https://docs.rs/fsr-sdk/0.1.0/fsr_sdk/): `error`, `runtime`
  and `upscaler` modules.
- [`fsr-sdk-sys 0.1.0`](https://docs.rs/fsr-sdk-sys/0.1.0/fsr_sdk_sys/): `api`,
  `dx12`, `loader` and `upscale` modules.

## Reproduction and limits

The observations above came from these read-only requests on 2026-09-29:

```sh
curl -fLsS https://index.crates.io/fs/r-/fsr-sdk-sys
curl -fLsS https://index.crates.io/fs/r-/fsr-sdk
curl -fLsS https://api.github.com/repos/Render-Slop-Manufactory/fsr-sdk-rs/releases/tags/v0.1.0
curl -fLsS https://docs.rs/fsr-sdk/0.1.0/fsr_sdk/
curl -fLsS https://docs.rs/fsr-sdk-sys/0.1.0/fsr_sdk_sys/
```

The checksums are registry-reported values; published archives were not downloaded
or compared with the [pre-publication candidate](2026-09-28-final-artifact-recheck.md).
That record's archive hashes identify its local candidate and differ from the
public registry hashes. This check establishes publication and hosted-documentation
availability; it adds no compilation, runtime or GPU verification.
