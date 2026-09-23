# SPDX-License-Identifier: MPL-2.0

# Run from an x64 VS 2022 developer shell with the local trusted v2.3.0 SDK.
param(
    [string[]]$Cases = @('C0', 'Z1', 'Z2', 'Z3', 'Z4', 'T1', 'N1', 'E1', 'D1')
)
$ErrorActionPreference = 'Stop'
Push-Location (Resolve-Path (Join-Path $PSScriptRoot '../../../..'))
$previousLoader = $env:FSR_SDK_TEST_DLL
$previousHelper = $env:FSR_SDK_TEST_DEVICE_DLL
$previousCase = $env:FSR_SDK_DIMENSION_CASE
$previousPath = $env:PATH
try {
    $stage = 'target/m5-dimensions'
    New-Item -ItemType Directory -Force $stage | Out-Null
    & cl /nologo /std:c++17 /W4 /WX /c /Iexternal/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/include crates/fsr-sdk-sys/tests/context_lifecycle/native_abi.cpp /Fotarget/m5-dimensions/native_abi.obj
    if ($LASTEXITCODE -ne 0) { throw 'Native ABI check failed' }
    & cl /nologo /std:c++17 /W4 /WX /MT /LD crates/fsr-sdk-sys/tests/context_lifecycle/device.cpp /Fotarget/m5-dimensions/device.obj /link /OUT:target/m5-dimensions/device.dll /IMPLIB:target/m5-dimensions/device.lib d3d12.lib dxgi.lib
    if ($LASTEXITCODE -ne 0) { throw 'Device helper build failed' }
    $messages = & cargo test -p fsr-sdk-sys --features dx12 --test context_dimensions --locked --offline --no-run --message-format=json
    if ($LASTEXITCODE -ne 0) { throw 'Probe build failed' }
    $artifact = $messages | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'context_dimensions' -and $_.executable } | Select-Object -Last 1
    if (!$artifact) { throw 'No probe executable' }
    Copy-Item -LiteralPath $artifact.executable -Destination "$stage/context_dimensions.exe"
    Copy-Item external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/signedbin/amd_fidelityfx_loader_dx12.dll,external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/signedbin/amd_fidelityfx_upscaler_dx12.dll -Destination $stage
    $stagePath = (Resolve-Path $stage).Path
    $env:FSR_SDK_TEST_DLL = Join-Path $stagePath 'amd_fidelityfx_loader_dx12.dll'
    $env:FSR_SDK_TEST_DEVICE_DLL = Join-Path $stagePath 'device.dll'
    $env:PATH = Join-Path $env:SystemRoot 'System32'
    foreach ($case in ($Cases | ForEach-Object { $_ -split ',' })) {
        $env:FSR_SDK_DIMENSION_CASE = $case
        $start = New-Object System.Diagnostics.ProcessStartInfo
        $start.FileName = Join-Path $stagePath 'context_dimensions.exe'
        $start.Arguments = '--ignored --exact signed_runtime_dimensions --nocapture --test-threads=1'
        $start.WorkingDirectory = $stagePath
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
        $signed = [int]$child.ExitCode
        $unsigned = [BitConverter]::ToUInt32([BitConverter]::GetBytes($signed), 0)
        $status = "case=$case timeout=$timeout exit_signed=$signed exit_hex=0x$($unsigned.ToString('X8'))"
        [IO.File]::WriteAllText((Join-Path $stagePath "$case.stdout.txt"), $stdout.Result)
        [IO.File]::WriteAllText((Join-Path $stagePath "$case.stderr.txt"), $stderr.Result)
        [IO.File]::WriteAllText((Join-Path $stagePath "$case.status.txt"), $status)
        Write-Host $status
        Write-Host $stderr.Result
        $child.Dispose()
    }
} finally {
    $env:FSR_SDK_TEST_DLL = $previousLoader
    $env:FSR_SDK_TEST_DEVICE_DLL = $previousHelper
    $env:FSR_SDK_DIMENSION_CASE = $previousCase
    $env:PATH = $previousPath
    Pop-Location
}
