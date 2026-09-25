# SPDX-License-Identifier: MPL-2.0

# Run in an x64 VS developer shell. -Native opts into the trusted SDK and GPU.
param([switch]$Native, [switch]$GpuValidation)
$ErrorActionPreference = 'Stop'
Push-Location (Resolve-Path (Join-Path $PSScriptRoot '../../..'))
$previousLoader = $env:FSR_SDK_TEST_DLL
$previousHelper = $env:FSR_SDK_TEST_M6_DLL
$previousGpuValidation = $env:FSR_SDK_TEST_GBV
try {
    & cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'Formatting failed' }
    & cargo clippy --workspace --all-targets --locked --offline -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Clippy failed' }
    & cargo test --workspace --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'Tests failed' }
    & cargo check --workspace --no-default-features --locked --offline
    if ($LASTEXITCODE -ne 0) { throw 'No-default-feature check failed' }
    if (-not $Native) { return }

    $stage = 'target/m6a'
    New-Item -ItemType Directory -Force $stage | Out-Null
    $sdk = 'external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX'
    & cl /nologo /std:c++17 /W4 /WX /c /I"$sdk/upscalers/include" crates/fsr-sdk-sys/tests/dispatch_native_abi.cpp /Fo"$stage/dispatch_native_abi.obj"
    if ($LASTEXITCODE -ne 0) { throw 'Paired dispatch ABI check failed' }
    & cl /nologo /std:c++17 /W4 /WX /MT /LD /I"$sdk/api/include/dx12" crates/fsr-sdk/tests/m6_gpu.cpp crates/fsr-sdk-sys/tests/context_lifecycle/device.cpp /Fo"$stage/" /link /OUT:"$stage/m6_gpu.dll" /IMPLIB:"$stage/m6_gpu.lib" d3d12.lib dxgi.lib
    if ($LASTEXITCODE -ne 0) { throw 'DX12 helper build failed' }
    $messages = & cargo test -p fsr-sdk --lib --features dx12 --locked --offline --no-run --message-format=json
    if ($LASTEXITCODE -ne 0) { throw 'Wrapper test build failed' }
    $artifact = $messages | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object {
        $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'fsr_sdk' -and $_.executable
    } | Select-Object -Last 1
    if (!$artifact) { throw 'No wrapper test executable' }
    Copy-Item -LiteralPath $artifact.executable -Destination "$stage/m6.exe"
    Copy-Item "$sdk/signedbin/amd_fidelityfx_loader_dx12.dll","$sdk/signedbin/amd_fidelityfx_upscaler_dx12.dll" -Destination $stage
    $env:FSR_SDK_TEST_DLL = (Resolve-Path "$stage/amd_fidelityfx_loader_dx12.dll").Path
    $env:FSR_SDK_TEST_M6_DLL = (Resolve-Path "$stage/m6_gpu.dll").Path
    if ($GpuValidation) { $env:FSR_SDK_TEST_GBV = '1' }
    else { Remove-Item Env:FSR_SDK_TEST_GBV -ErrorAction SilentlyContinue }
    $hashes = Get-ChildItem $stage -Filter '*.dll' | Get-FileHash -Algorithm SHA256 |
        Select-Object @{Name='File';Expression={Split-Path $_.Path -Leaf}},Hash | ConvertTo-Json
    $system = Get-CimInstance Win32_OperatingSystem |
        Select-Object Caption,Version,BuildNumber | Format-List | Out-String
    $gpu = Get-CimInstance Win32_VideoController |
        Select-Object Name,DriverVersion | Format-List | Out-String
    [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/native.environment.txt"),
        "SDK=v2.3.0`r`nGPUValidation=$GpuValidation`r`nRust=$(& rustc --version)`r`nOS=$system`r`nGPU=$gpu`r`nSHA256=$hashes")
    $start = New-Object System.Diagnostics.ProcessStartInfo
    $start.FileName = (Resolve-Path "$stage/m6.exe").Path
    $start.Arguments = '--ignored --exact upscaler::m6::native_dispatch_readback --nocapture --test-threads=1'
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $child = [System.Diagnostics.Process]::Start($start)
    $stdout = $child.StandardOutput.ReadToEndAsync()
    $stderr = $child.StandardError.ReadToEndAsync()
    $timeout = !$child.WaitForExit(180000)
    if ($timeout) { $child.Kill() }
    $child.WaitForExit()
    $exitCode = $child.ExitCode
    $child.Dispose()
    [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/native.stdout.txt"), $stdout.Result)
    [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/native.stderr.txt"), $stderr.Result)
    [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/native.status.txt"), "timeout=$timeout exit=$exitCode")
    Write-Host $stdout.Result
    Write-Host $stderr.Result
    if ($timeout -or $exitCode -ne 0 -or $stderr.Result -notmatch 'GPU readback:') {
        throw "Native M6a dispatch failed: timeout=$timeout exit=$exitCode"
    }
} finally {
    $env:FSR_SDK_TEST_DLL = $previousLoader
    $env:FSR_SDK_TEST_M6_DLL = $previousHelper
    $env:FSR_SDK_TEST_GBV = $previousGpuValidation
    Pop-Location
}
