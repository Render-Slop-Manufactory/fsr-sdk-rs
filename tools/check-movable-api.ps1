# SPDX-License-Identifier: MPL-2.0

# Compile real downstream consumers; expected failures require exact diagnostics.
param([string]$Stage = 'target/movable-api-controls')
$ErrorActionPreference = 'Stop'
if (-not $IsWindows) { throw 'These consumer controls require Windows/DX12.' }
$taskRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$stagePath = [IO.Path]::GetFullPath((Join-Path $taskRoot $Stage))
New-Item -ItemType Directory -Force (Join-Path $stagePath 'src') | Out-Null
$wrapper = (Join-Path $taskRoot 'crates/fsr-sdk').Replace('\', '/')
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
"@
[IO.File]::WriteAllText((Join-Path $stagePath 'Cargo.toml'), $manifest)
$cases = @(
    @{ Name='local-positive'; Features='dx12'; Diagnostic=$null; Source='use fsr_sdk::runtime::Runtime; fn main() { let _: Option<Runtime> = None; }' },
    @{ Name='movable-positive'; Features='dx12,movable'; Diagnostic=$null; Source='use fsr_sdk::{runtime::MovableRuntime, upscaler::MovableUpscaler}; fn send<T: Send>() {} fn main() { send::<MovableRuntime>(); send::<MovableUpscaler>(); }' },
    @{ Name='feature-disabled'; Features='dx12'; Diagnostic='E0432'; Source='use fsr_sdk::runtime::MovableRuntime; fn main() { let _: Option<MovableRuntime> = None; }' },
    @{ Name='dx12-disabled'; Features='movable'; Diagnostic='E0432'; Source='use fsr_sdk::runtime::MovableRuntime; fn main() { let _: Option<MovableRuntime> = None; }' },
    @{ Name='local-to-movable-construction'; Features='dx12,movable'; Diagnostic='E0308'; Source='use fsr_sdk::{runtime::Runtime, upscaler::{MovableUpscaler, UpscalerOptions, ID3D12Device}}; fn wrong(r: &Runtime, d: &ID3D12Device, o: UpscalerOptions) { let _ = MovableUpscaler::new(r, d, o); } fn main() {}' },
    @{ Name='movable-to-local-construction'; Features='dx12,movable'; Diagnostic='E0308'; Source='use fsr_sdk::{runtime::MovableRuntime, upscaler::{Upscaler, UpscalerOptions, ID3D12Device}}; fn wrong(r: &MovableRuntime, d: &ID3D12Device, o: UpscalerOptions) { let _ = Upscaler::new(r, d, o); } fn main() {}' },
    @{ Name='no-local-to-movable-conversion'; Features='dx12,movable'; Diagnostic='E0277'; Source='use fsr_sdk::runtime::{Runtime, MovableRuntime}; fn wrong(r: Runtime) -> MovableRuntime { r.into() } fn main() {}' },
    @{ Name='no-movable-to-local-conversion'; Features='dx12,movable'; Diagnostic='E0277'; Source='use fsr_sdk::runtime::{Runtime, MovableRuntime}; fn wrong(r: MovableRuntime) -> Runtime { r.into() } fn main() {}' }
)
foreach ($case in $cases) {
    [IO.File]::WriteAllText((Join-Path $stagePath 'src/main.rs'), $case.Source)
    $output = & cargo check --manifest-path (Join-Path $stagePath 'Cargo.toml') --features $case.Features --offline --target-dir (Join-Path $stagePath 'build') 2>&1 | Out-String
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
