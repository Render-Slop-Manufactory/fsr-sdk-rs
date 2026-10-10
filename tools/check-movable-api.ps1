# SPDX-License-Identifier: MPL-2.0

# Compile real downstream consumers; expected failures require exact diagnostics.
param([string]$Stage = 'target/movable-api-controls')
$ErrorActionPreference = 'Stop'
$taskRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$stagePath = [IO.Path]::GetFullPath((Join-Path $taskRoot $Stage))
New-Item -ItemType Directory -Force (Join-Path $stagePath 'src') | Out-Null
$wrapper = (Join-Path $taskRoot 'crates/fsr-sdk').Replace('\', '/')
$raw = (Join-Path $taskRoot 'crates/fsr-sdk-sys').Replace('\', '/')
$manifest = @"
[package]
name = "movable-api-control"
version = "0.0.0"
edition = "2024"
[workspace]
[features]
dx12 = ["fsr-sdk/dx12"]
movable = ["fsr-sdk/movable"]
[dependencies.fsr-sdk]
path = "$wrapper"
default-features = false
[dependencies.fsr-sdk-sys]
path = "$raw"
default-features = false
"@
[IO.File]::WriteAllText((Join-Path $stagePath 'Cargo.toml'), $manifest)
$cases = @(
    @{ Name='common-positive'; Features=''; Diagnostic=$null; Source='use fsr_sdk::upscaler::{Dimensions, common}; use fsr_sdk_sys::{api::ffxApiHeader, upscale::ffxDispatchDescUpscale}; fn main() { let _: Dimensions = common::Dimensions::new(320, 180).unwrap(); let _: Option<ffxApiHeader> = None; let _: Option<ffxDispatchDescUpscale> = None; }' },
    @{ Name='common-movable-positive'; Features='movable'; Diagnostic=$null; Source='use fsr_sdk::upscaler::{CameraParameters, FrameTimeMillis, JitterOffsetPixels, UpscalerOptions, common}; fn main() { let d = common::Dimensions::new(320, 180).unwrap(); let _: UpscalerOptions = common::UpscalerOptions::new(d, d); let _: CameraParameters = common::CameraParameters::new(0.1, 100.0, 1.0, 1.0).unwrap(); let _: FrameTimeMillis = common::FrameTimeMillis::new(16.0).unwrap(); let _: JitterOffsetPixels = common::JitterOffsetPixels::new(0.0, 0.0).unwrap(); }' }
)
# Both configurations omit DX12 on Windows; even enabled native features must
# expose no native API on other hosts. Check each boundary independently.
$disabledFeatures = @('', 'movable')
if (-not $IsWindows) { $disabledFeatures += @('dx12', 'dx12,movable') }
foreach ($features in $disabledFeatures) {
    foreach ($boundary in @(
        @{ Name='runtime'; Import='fsr_sdk::runtime' },
        @{ Name='dx12-module'; Import='fsr_sdk::upscaler::dx12' },
        @{ Name='legacy-upscaler'; Import='fsr_sdk::upscaler::Upscaler' },
        @{ Name='legacy-movable'; Import='fsr_sdk::upscaler::MovableUpscaler' },
        @{ Name='raw-loader'; Import='fsr_sdk_sys::loader' },
        @{ Name='raw-dx12'; Import='fsr_sdk_sys::dx12' }
    )) {
        $cases += @{ Name="native-disabled-$($boundary.Name)-$($features.Replace(',', '-'))"; Features=$features; Diagnostic='E0432'; Source="use $($boundary.Import); fn main() {}" }
    }
}
if ($IsWindows) { $cases += @(
    @{ Name='local-positive'; Features='dx12'; Diagnostic=$null; Source='use fsr_sdk::runtime::Runtime; fn main() { let _: Option<Runtime> = None; }' },
    @{ Name='movable-positive'; Features='dx12,movable'; Diagnostic=$null; Source='use fsr_sdk::{runtime::MovableRuntime, upscaler::MovableUpscaler}; fn send<T: Send>() {} fn main() { send::<MovableRuntime>(); send::<MovableUpscaler>(); }' },
    @{ Name='feature-disabled'; Features='dx12'; Diagnostic='E0432'; Source='use fsr_sdk::runtime::MovableRuntime; fn main() { let _: Option<MovableRuntime> = None; }' },
    @{ Name='dx12-disabled'; Features='movable'; Diagnostic='E0432'; Source='use fsr_sdk::runtime::MovableRuntime; fn main() { let _: Option<MovableRuntime> = None; }' },
    @{ Name='local-to-movable-construction'; Features='dx12,movable'; Diagnostic='E0308'; Source='use fsr_sdk::{runtime::Runtime, upscaler::{MovableUpscaler, UpscalerOptions, ID3D12Device}}; fn wrong(r: &Runtime, d: &ID3D12Device, o: UpscalerOptions) { let _ = MovableUpscaler::new(r, d, o); } fn main() {}' },
    @{ Name='movable-to-local-construction'; Features='dx12,movable'; Diagnostic='E0308'; Source='use fsr_sdk::{runtime::MovableRuntime, upscaler::{Upscaler, UpscalerOptions, ID3D12Device}}; fn wrong(r: &MovableRuntime, d: &ID3D12Device, o: UpscalerOptions) { let _ = Upscaler::new(r, d, o); } fn main() {}' },
    @{ Name='no-local-to-movable-conversion'; Features='dx12,movable'; Diagnostic='E0277'; Source='use fsr_sdk::runtime::{Runtime, MovableRuntime}; fn wrong(r: Runtime) -> MovableRuntime { r.into() } fn main() {}' },
    @{ Name='no-movable-to-local-conversion'; Features='dx12,movable'; Diagnostic='E0277'; Source='use fsr_sdk::runtime::{Runtime, MovableRuntime}; fn wrong(r: MovableRuntime) -> Runtime { r.into() } fn main() {}' }
) }
foreach ($case in $cases) {
    [IO.File]::WriteAllText((Join-Path $stagePath 'src/main.rs'), $case.Source)
    $cargoArgs = @('check', '--manifest-path', (Join-Path $stagePath 'Cargo.toml'), '--offline', '--target-dir', (Join-Path $stagePath 'build'))
    if ($case.Features) { $cargoArgs += @('--features', $case.Features) }
    $output = & cargo @cargoArgs 2>&1 | Out-String
    $code = $LASTEXITCODE
    [IO.File]::WriteAllText((Join-Path $stagePath "$($case.Name).txt"), $output)
    if ($case.Diagnostic) {
        if ($code -eq 0 -or !$output.Contains("error[$($case.Diagnostic)]")) {
            throw "Invalid negative control $($case.Name): exit=$code`n$output"
        }
    } elseif ($code -ne 0) { throw "Positive control failed $($case.Name):`n$output" }
    Write-Host "$($case.Name): PASS (exit=$code; expected=$($case.Diagnostic))"
}

# All expected compiler failures were validated; report overall success to CI.
exit 0
