# SPDX-License-Identifier: MPL-2.0

# Run in an x64 VS developer shell. -Native opts into the trusted SDK and GPU.
param([switch]$Native, [switch]$GpuValidation, [string]$Stage = 'target/m6a')
$ErrorActionPreference = 'Stop'
Push-Location (Resolve-Path (Join-Path $PSScriptRoot '../../..'))
$previousLoader = $env:FSR_SDK_TEST_DLL
$previousHelper = $env:FSR_SDK_TEST_M6_DLL
$previousGpuValidation = $env:FSR_SDK_TEST_GBV
$previousControl = $env:FSR_SDK_TEST_M6_CONTROL
$stage = $Stage
New-Item -ItemType Directory -Force $stage | Out-Null
$transcriptStarted = $false

function Save-CheckResult([string]$command, [int]$code) {
    Add-Content -LiteralPath "$stage/checks.status.txt" -Value "$command => exit=$code"
    if ($code -ne 0) { throw "Check failed (exit=$code): $command" }
}
try {
    Start-Transcript -LiteralPath "$stage/runner.log" -Force | Out-Null
    $transcriptStarted = $true
    [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/checks.status.txt"), '')
    if ($Native) {
        $candidateCommit = & git rev-parse HEAD
        $candidateTree = & git rev-parse 'HEAD^{tree}'
        $candidateStatus = & git status --short
        $candidatePatch = & git diff --no-ext-diff --binary -- crates/fsr-sdk/src/upscaler/m6.rs crates/fsr-sdk/tests/m6_gpu.cpp crates/fsr-sdk/tests/run-m6.ps1
        [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/candidate.patch"), ($candidatePatch -join "`n") + "`n")
        [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/source-state.txt"),
            "Commit=$candidateCommit`nBaseTree=$candidateTree`nCaptured=$(Get-Date -Format o)`nStatus=$($candidateStatus -join "`n")")
        $sourceHashes = & git ls-files -- crates Cargo.toml Cargo.lock | ForEach-Object {
            [pscustomobject]@{File=$_; SHA256=(Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash}
        }
        [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/source-inputs.json"), ($sourceHashes | ConvertTo-Json))
    }
    & cargo fmt --all -- --check
    Save-CheckResult 'cargo fmt --all -- --check' $LASTEXITCODE
    & cargo clippy --workspace --all-targets --locked --offline -- -D warnings
    Save-CheckResult 'cargo clippy --workspace --all-targets --locked --offline -- -D warnings' $LASTEXITCODE
    & cargo test --workspace --locked --offline
    Save-CheckResult 'cargo test --workspace --locked --offline' $LASTEXITCODE
    & cargo check --workspace --no-default-features --locked --offline
    Save-CheckResult 'cargo check --workspace --no-default-features --locked --offline' $LASTEXITCODE
    if (-not $Native) { return }

    $sdk = 'external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX'
    & cl /nologo /std:c++17 /W4 /WX /c /I"$sdk/upscalers/include" crates/fsr-sdk-sys/tests/dispatch_native_abi.cpp /Fo"$stage/dispatch_native_abi.obj"
    Save-CheckResult "cl /nologo /std:c++17 /W4 /WX /c /I$sdk/upscalers/include crates/fsr-sdk-sys/tests/dispatch_native_abi.cpp /Fo$stage/dispatch_native_abi.obj" $LASTEXITCODE
    & cl /nologo /std:c++17 /W4 /WX /MT /LD /I"$sdk/api/include/dx12" crates/fsr-sdk/tests/m6_gpu.cpp crates/fsr-sdk-sys/tests/context_lifecycle/device.cpp /Fo"$stage/" /link /OUT:"$stage/m6_gpu.dll" /IMPLIB:"$stage/m6_gpu.lib" d3d12.lib dxgi.lib
    Save-CheckResult "cl /nologo /std:c++17 /W4 /WX /MT /LD /I$sdk/api/include/dx12 crates/fsr-sdk/tests/m6_gpu.cpp crates/fsr-sdk-sys/tests/context_lifecycle/device.cpp /Fo$stage/ /link /OUT:$stage/m6_gpu.dll /IMPLIB:$stage/m6_gpu.lib d3d12.lib dxgi.lib" $LASTEXITCODE
    $messages = & cargo test -p fsr-sdk --lib --features dx12 --locked --offline --no-run --message-format=json
    Save-CheckResult 'cargo test -p fsr-sdk --lib --features dx12 --locked --offline --no-run --message-format=json' $LASTEXITCODE
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
    Remove-Item Env:FSR_SDK_TEST_M6_CONTROL -ErrorAction SilentlyContinue
    $hashes = Get-ChildItem $stage -Filter '*.dll' | Get-FileHash -Algorithm SHA256 |
        Select-Object @{Name='File';Expression={Split-Path $_.Path -Leaf}},Hash | ConvertTo-Json
    $system = Get-CimInstance Win32_OperatingSystem |
        Select-Object Caption,Version,BuildNumber | Format-List | Out-String
    $gpu = Get-CimInstance Win32_VideoController |
        Select-Object Name,DriverVersion | Format-List | Out-String
    $signatureObjects = Get-ChildItem $stage -Filter 'amd_*.dll' | Get-AuthenticodeSignature
    $signatures = $signatureObjects |
        Select-Object @{Name='File';Expression={Split-Path $_.Path -Leaf}},Status | ConvertTo-Json
    if ($signatureObjects | Where-Object Status -ne 'Valid') { throw 'ENVIRONMENT BLOCKER: staged AMD DLL signature is not valid' }
    $rustVersion = & rustc -vV | Out-String
    $compilerVersion = (Get-Item -LiteralPath (Get-Command cl).Source).VersionInfo.FileVersion
    $zone = Get-TimeZone | Select-Object Id,BaseUtcOffset | ConvertTo-Json
    [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/native.environment.txt"),
        "Started=$(Get-Date -Format o)`r`nTimezone=$zone`r`nSDK=v2.3.0`r`nGPUValidation=$GpuValidation`r`nRust=$rustVersion`r`nMSVC=$env:VCToolsVersion`r`nCompiler=$compilerVersion`r`nPowerShell=$($PSVersionTable.PSVersion)`r`nWindowsSDK=$env:WindowsSDKVersion`r`nTargetArch=$env:VSCMD_ARG_TGT_ARCH`r`nOS=$system`r`nGPU=$gpu`r`nSHA256=$hashes`r`nSignatures=$signatures`r`nFSR_SDK_TEST_DLL=$stage/amd_fidelityfx_loader_dx12.dll (absolute at execution)`r`nFSR_SDK_TEST_M6_DLL=$stage/m6_gpu.dll (absolute at execution)`r`nFSR_SDK_TEST_GBV=<unset unless -GpuValidation>`r`nFSR_SDK_TEST_M6_CONTROL=<unset>")
    $inputHashes = Get-ChildItem "$sdk/api/include","$sdk/upscalers/include" -File -Recurse | ForEach-Object {
        [pscustomobject]@{File=[IO.Path]::GetRelativePath((Get-Location).Path,$_.FullName).Replace('\','/'); SHA256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash}
    }
    $inputHashes += Get-Item "$stage/m6.exe","$stage/m6_gpu.dll" | ForEach-Object {
        [pscustomobject]@{File="$stage/$($_.Name)"; SHA256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash}
    }
    [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/native-inputs.json"), ($inputHashes | ConvertTo-Json))
    $failed = @()
    foreach ($gate in @(
        @{Name='gate-a'; Test='native_dispatch_readback'; Marker='Gate A: PASSED'},
        @{Name='gate-b'; Test='native_temporal_smoke'; Marker='Gate B: PASSED'}
    )) {
        $name = $gate.Name
        [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/$name.status.txt"), 'not_started')
        $start = New-Object System.Diagnostics.ProcessStartInfo
        $start.FileName = (Resolve-Path "$stage/m6.exe").Path
        $start.Arguments = "--ignored --exact upscaler::m6::$($gate.Test) --nocapture --test-threads=1"
        $start.UseShellExecute = $false
        $start.CreateNoWindow = $true
        $start.RedirectStandardOutput = $true
        $start.RedirectStandardError = $true
        $started = Get-Date -Format o
        Write-Host "Child command: $stage/m6.exe $($start.Arguments)"
        $child = [System.Diagnostics.Process]::Start($start)
        $stdout = $child.StandardOutput.ReadToEndAsync()
        $stderr = $child.StandardError.ReadToEndAsync()
        $timeout = !$child.WaitForExit(180000)
        if ($timeout) { $child.Kill() }
        $child.WaitForExit()
        $exitCode = $child.ExitCode
        $child.Dispose()
        [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/$name.stdout.txt"), $stdout.Result)
        [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/$name.stderr.txt"), $stderr.Result)
        [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/$name.status.txt"),
            "started=$started`nfinished=$(Get-Date -Format o)`ntimeout=$timeout exit=$exitCode")
        Write-Host $stdout.Result
        Write-Host $stderr.Result
        if ($timeout -or $exitCode -ne 0 -or !$stderr.Result.Contains($gate.Marker)) {
            $failed += "$name timeout=$timeout exit=$exitCode"
        }
    }
    if ($failed.Count) { throw "Windows verification failed: $($failed -join '; ')" }
} finally {
    $env:FSR_SDK_TEST_DLL = $previousLoader
    $env:FSR_SDK_TEST_M6_DLL = $previousHelper
    $env:FSR_SDK_TEST_GBV = $previousGpuValidation
    $env:FSR_SDK_TEST_M6_CONTROL = $previousControl
    if ($transcriptStarted) { Stop-Transcript | Out-Null }
    Pop-Location
}
