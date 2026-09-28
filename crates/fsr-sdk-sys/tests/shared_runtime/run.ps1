# SPDX-License-Identifier: MPL-2.0

# Run from an x64 VS developer shell. The child is the only process that loads
# AMD code; staged binaries follow D007's executable-adjacent layout.
$ErrorActionPreference = 'Stop'
Push-Location (Resolve-Path (Join-Path $PSScriptRoot '../../../..'))
try {
    $sdk = 'external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX'
    $stage = 'target/shared-runtime'
    $loader = Join-Path $sdk 'signedbin/amd_fidelityfx_loader_dx12.dll'
    $provider = Join-Path $sdk 'signedbin/amd_fidelityfx_upscaler_dx12.dll'
    $expected = @{
        'amd_fidelityfx_loader_dx12.dll' = 'E2D85AA05A9BD9ED8B38935FDF5199372CCA6F74C12015143BB6F945EE1608AA'
        'amd_fidelityfx_upscaler_dx12.dll' = 'D0DCCCC74A43C44BA435B7A369B456E0970D8A4464E4BD683119B374F2C9FB46'
    }
    foreach ($file in @($loader, $provider)) {
        $hash = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash
        $signature = Get-AuthenticodeSignature -LiteralPath $file
        $name = Split-Path -Leaf $file
        if ($hash -ne $expected[$name] -or $signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'Advanced Micro Devices') {
            throw "Runtime identity/signature check failed for $name"
        }
        Write-Host "$name SHA256=$hash Authenticode=Valid signer=Advanced Micro Devices"
    }
    New-Item -ItemType Directory -Force -Path $stage | Out-Null
    & cl /nologo /std:c++17 /W4 /WX /c /Iexternal/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/include crates/fsr-sdk-sys/tests/context_lifecycle/native_abi.cpp /Fotarget/shared-runtime/native_abi.obj
    if ($LASTEXITCODE -ne 0) { throw 'Paired native ABI check failed' }
    & cl /nologo /std:c++17 /W4 /WX /MT /LD crates/fsr-sdk-sys/tests/context_lifecycle/device.cpp /Fotarget/shared-runtime/device.obj /link /OUT:target/shared-runtime/device.dll /IMPLIB:target/shared-runtime/device.lib d3d12.lib dxgi.lib
    if ($LASTEXITCODE -ne 0) { throw 'Device helper build failed' }
    $messages = & cargo test -p fsr-sdk-sys --features dx12 --test shared_runtime --locked --offline --no-run --message-format=json
    if ($LASTEXITCODE -ne 0) { throw 'Rust probe build failed' }
    $artifact = $messages | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'shared_runtime' -and $_.executable } | Select-Object -Last 1
    if (!$artifact) { throw 'Probe executable was not reported' }
    Copy-Item -LiteralPath $artifact.executable -Destination "$stage/shared_runtime.exe"
    Copy-Item -LiteralPath $loader,$provider -Destination $stage
    foreach ($name in $expected.Keys) {
        if ((Get-FileHash -LiteralPath (Join-Path $stage $name) -Algorithm SHA256).Hash -ne $expected[$name]) {
            throw "Staged runtime hash mismatch for $name"
        }
    }

    $start = New-Object System.Diagnostics.ProcessStartInfo
    $start.FileName = (Resolve-Path "$stage/shared_runtime.exe").Path
    $start.Arguments = '--ignored --exact sequential_shared_runtime_lifetime --nocapture --test-threads=1'
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.EnvironmentVariables['FSR_SDK_TEST_DLL'] = (Resolve-Path "$stage/amd_fidelityfx_loader_dx12.dll").Path
    $start.EnvironmentVariables['FSR_SDK_TEST_PROVIDER_DLL'] = (Resolve-Path "$stage/amd_fidelityfx_upscaler_dx12.dll").Path
    $start.EnvironmentVariables['FSR_SDK_TEST_DEVICE_DLL'] = (Resolve-Path "$stage/device.dll").Path
    $child = [System.Diagnostics.Process]::Start($start)
    $stdout = $child.StandardOutput.ReadToEndAsync()
    $stderr = $child.StandardError.ReadToEndAsync()
    $timedOut = !$child.WaitForExit(60000)
    if ($timedOut) { $child.Kill() }
    $child.WaitForExit()
    $status = "timeout=$timedOut exit=$($child.ExitCode)"
    $child.Dispose()
    [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/native.stdout.txt"), $stdout.Result)
    [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/native.stderr.txt"), $stderr.Result)
    [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/native.status.txt"), $status)
    Write-Host $stdout.Result
    Write-Host $stderr.Result
    Write-Host $status
} finally {
    Pop-Location
}
