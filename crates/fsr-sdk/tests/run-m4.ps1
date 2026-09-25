# SPDX-License-Identifier: MPL-2.0

# Ordinary checks need Rust/MSVC only. -Native additionally needs the trusted
# local SDK, an x64 VS developer shell and a hardware DX12 device.
param([switch]$Native)
$ErrorActionPreference = 'Stop'
Push-Location (Resolve-Path (Join-Path $PSScriptRoot '../../..'))
$previousLoader = $env:FSR_SDK_TEST_DLL
$previousHelper = $env:FSR_SDK_TEST_DEVICE_DLL
try {
    & cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'Formatting failed' }
    & cargo clippy --workspace --all-targets --locked -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Clippy failed' }
    & cargo test --workspace --locked
    if ($LASTEXITCODE -ne 0) { throw 'Tests failed' }
    & cargo check --workspace --no-default-features --locked
    if ($LASTEXITCODE -ne 0) { throw 'No-default-feature check failed' }
    if ($Native) {
        $stage = 'target/m4-lifecycle'
        New-Item -ItemType Directory -Force $stage | Out-Null
        & cl /nologo /std:c++17 /W4 /WX /c /Iexternal/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/include crates/fsr-sdk-sys/tests/context_lifecycle/native_abi.cpp /Fotarget/m4-lifecycle/native_abi.obj
        if ($LASTEXITCODE -ne 0) { throw 'Native ABI verification failed' }
        & cl /nologo /std:c++17 /W4 /WX /MT /LD crates/fsr-sdk-sys/tests/context_lifecycle/device.cpp /Fotarget/m4-lifecycle/device.obj /link /OUT:target/m4-lifecycle/device.dll /IMPLIB:target/m4-lifecycle/device.lib d3d12.lib dxgi.lib
        if ($LASTEXITCODE -ne 0) { throw 'Device helper build failed' }
        $messages = & cargo test -p fsr-sdk --lib --features dx12 --locked --no-run --message-format=json
        if ($LASTEXITCODE -ne 0) { throw 'Wrapper test build failed' }
        $artifact = $messages | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'fsr_sdk' -and $_.executable } | Select-Object -Last 1
        if (!$artifact) { throw 'No wrapper test executable' }
        Copy-Item -LiteralPath $artifact.executable -Destination "$stage/m4.exe"
        Copy-Item external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/signedbin/amd_fidelityfx_loader_dx12.dll,external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/signedbin/amd_fidelityfx_upscaler_dx12.dll -Destination $stage
        $env:FSR_SDK_TEST_DLL = (Resolve-Path "$stage/amd_fidelityfx_loader_dx12.dll").Path
        $env:FSR_SDK_TEST_DEVICE_DLL = (Resolve-Path "$stage/device.dll").Path
        $start = New-Object System.Diagnostics.ProcessStartInfo
        $start.FileName = (Resolve-Path "$stage/m4.exe").Path
        $start.Arguments = '--ignored --exact upscaler::tests::native_lifecycle --nocapture --test-threads=1'
        $start.UseShellExecute = $false
        $start.CreateNoWindow = $true
        $start.RedirectStandardOutput = $true
        $start.RedirectStandardError = $true
        $child = [System.Diagnostics.Process]::Start($start)
        $stdout = $child.StandardOutput.ReadToEndAsync()
        $stderr = $child.StandardError.ReadToEndAsync()
        $timeout = !$child.WaitForExit(60000)
        if ($timeout) { $child.Kill() }
        $child.WaitForExit()
        $exitCode = $child.ExitCode
        $child.Dispose()
        [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/native.stdout.txt"), $stdout.Result)
        [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/native.stderr.txt"), $stderr.Result)
        [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/native.status.txt"), "timeout=$timeout exit=$exitCode")
        Write-Host $stdout.Result
        Write-Host $stderr.Result
        if ($timeout -or $exitCode -ne 0 -or $stderr.Result -notmatch 'Production owner: destroy A/B OK') { throw "Native lifecycle failed: timeout=$timeout exit=$exitCode" }
    }
} finally {
    $env:FSR_SDK_TEST_DLL = $previousLoader
    $env:FSR_SDK_TEST_DEVICE_DLL = $previousHelper
    Pop-Location
}
