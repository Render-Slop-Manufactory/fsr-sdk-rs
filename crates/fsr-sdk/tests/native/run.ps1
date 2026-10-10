# SPDX-License-Identifier: MPL-2.0

# Run in an x64 VS developer shell. -Native opts into the trusted SDK and GPU.
param(
    [ValidateSet('lifecycle', 'dispatch', 'd013', 'd013-cleanup', 'movable', 'sharpening', 'm9')][string]$Mode = 'dispatch',
    [switch]$Native, [switch]$GpuValidation,
    [string]$Stage = 'target/native-verification',
    # Verify an extracted wrapper artifact while keeping SDK/equipment in this checkout.
    [string]$TestManifest
)
$ErrorActionPreference = 'Stop'
if ($Mode -in @('d013', 'd013-cleanup', 'movable') -and $GpuValidation) { throw 'D013 excludes GBV; use ordinary debug layer only' }
if ($Mode -in @('sharpening', 'm9') -and $GpuValidation) { throw 'Sharpening protocol excludes GBV; use ordinary debug layer only' }
Push-Location (Resolve-Path (Join-Path $PSScriptRoot '../../../..'))
$previousLoader = $env:FSR_SDK_TEST_DLL
$previousHelper = $env:FSR_SDK_TEST_M6_DLL
$previousDeviceHelper = $env:FSR_SDK_TEST_DEVICE_DLL
$previousGpuValidation = $env:FSR_SDK_TEST_GBV
$previousControl = $env:FSR_SDK_TEST_M6_CONTROL
$stage = $Stage
if ($Mode -eq 'm9' -and (Test-Path -LiteralPath "$stage/runner.log")) {
    throw 'M9 requires a fresh stage; preserve prior attempts without overwriting or retrying'
}
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
        $candidatePatch = & git diff --no-ext-diff --binary -- crates Cargo.toml Cargo.lock
        [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/candidate.patch"), ($candidatePatch -join "`n") + "`n")
        [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/source-state.txt"),
            "Commit=$candidateCommit`nBaseTree=$candidateTree`nCaptured=$(Get-Date -Format o)`nStatus=$($candidateStatus -join "`n")")
        $sourceFiles = @(Get-ChildItem crates -File -Recurse | ForEach-Object { [IO.Path]::GetRelativePath((Get-Location).Path, $_.FullName) }) + @('Cargo.toml', 'Cargo.lock')
        $sourceHashes = $sourceFiles | ForEach-Object {
            [pscustomobject]@{File=$_; SHA256=(Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash}
        }
        [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/source-inputs.json"), ($sourceHashes | ConvertTo-Json))
        if ($Mode -eq 'm9') {
            $protocol = 'docs/release/2026-10-09-m9-native-sharpening-protocol.md'
            [pscustomobject]@{
                Command="./crates/fsr-sdk/tests/native/run.ps1 -Mode m9 -Native -Stage $stage" + $(if ($TestManifest) { " -TestManifest $TestManifest" } else { '' })
                Protocol=$protocol; ProtocolSHA256=(Get-FileHash -LiteralPath $protocol).Hash
                IntegrationTest='upscaler::dx12::m6::native_sharpening_integration'
                Repetitions=3; OutputsPerRepetition=48; WatchdogMilliseconds=180000
                DebugLayer=$true; GPUValidation=$false; ImageComparisons='diagnostics only'
            } | ConvertTo-Json | Set-Content -LiteralPath "$stage/execution-plan.json"
        }
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
    & ./tools/check-native-abi.ps1 -SdkRoot $sdk -Stage "$stage/abi"
    & cl /nologo /std:c++17 /W4 /WX /MT /LD /I"$sdk/api/include/dx12" crates/fsr-sdk/tests/native/gpu.cpp crates/fsr-sdk-sys/tests/context_lifecycle/device.cpp /Fo"$stage/" /link /OUT:"$stage/m6_gpu.dll" /IMPLIB:"$stage/m6_gpu.lib" d3d12.lib dxgi.lib
    Save-CheckResult "cl /nologo /std:c++17 /W4 /WX /MT /LD /I$sdk/api/include/dx12 crates/fsr-sdk/tests/native/gpu.cpp crates/fsr-sdk-sys/tests/context_lifecycle/device.cpp /Fo$stage/ /link /OUT:$stage/m6_gpu.dll /IMPLIB:$stage/m6_gpu.lib d3d12.lib dxgi.lib" $LASTEXITCODE
    $publicFeatures = if ($Mode -eq 'movable') { 'dx12,movable' } else { 'dx12' }
    $testArguments = @('test', '-p', 'fsr-sdk', '--lib', '--features', $publicFeatures, '--locked', '--offline', '--no-run', '--message-format=json')
    if ($TestManifest) {
        $testArguments += @('--manifest-path', $TestManifest)
        $packageRoot = Split-Path (Resolve-Path -LiteralPath $TestManifest).Path
        $packageHashes = Get-ChildItem -LiteralPath $packageRoot -File -Recurse | Where-Object {
            -not [IO.Path]::GetRelativePath($packageRoot, $_.FullName).StartsWith('target' + [IO.Path]::DirectorySeparatorChar)
        } | ForEach-Object {
            [pscustomobject]@{File=[IO.Path]::GetRelativePath($packageRoot,$_.FullName).Replace('\','/'); SHA256=(Get-FileHash -LiteralPath $_.FullName).Hash}
        }
        $packageHashes | ConvertTo-Json | Set-Content -LiteralPath "$stage/test-package-inputs.json"
    }
    $messages = & cargo @testArguments
    Save-CheckResult ('cargo ' + ($testArguments -join ' ')) $LASTEXITCODE
    $artifact = $messages | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object {
        $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'fsr_sdk' -and $_.executable
    } | Select-Object -Last 1
    if (!$artifact) { throw 'No wrapper test executable' }
    Copy-Item -LiteralPath $artifact.executable -Destination "$stage/m6.exe"
    Copy-Item "$sdk/signedbin/amd_fidelityfx_loader_dx12.dll","$sdk/signedbin/amd_fidelityfx_upscaler_dx12.dll" -Destination $stage
    $env:FSR_SDK_TEST_DLL = (Resolve-Path "$stage/amd_fidelityfx_loader_dx12.dll").Path
    $env:FSR_SDK_TEST_M6_DLL = (Resolve-Path "$stage/m6_gpu.dll").Path
    $env:FSR_SDK_TEST_DEVICE_DLL = $env:FSR_SDK_TEST_M6_DLL
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
    $gates = if ($Mode -eq 'lifecycle') {
        @(
            @{Name='native-explicit'; Test='upscaler::dx12::tests::native_lifecycle'; Marker='Production owner: destroy A/B OK'},
            @{Name='native-drop'; Test='upscaler::dx12::tests::native_lifecycle_drop'; Marker='Production owner: destroy A/B OK'}
        )
    } elseif ($Mode -eq 'movable') {
        @(
            @{Name='native-explicit'; Test='upscaler::dx12::tests::native_lifecycle'; Marker='Production owner: destroy A/B OK'},
            @{Name='native-drop'; Test='upscaler::dx12::tests::native_lifecycle_drop'; Marker='Production owner: destroy A/B OK'},
            @{Name='gate-a'; Test='upscaler::dx12::m6::native_dispatch_readback'; Marker='Gate A: PASSED'},
            @{Name='gate-b'; Test='upscaler::dx12::m6::native_temporal_smoke'; Marker='Gate B: PASSED'},
            @{Name='gate-c'; Test='upscaler::dx12::m6::native_semantic_comparisons'; Marker='Gate C: PASSED'},
            @{Name='public-movable'; Test='upscaler::dx12::m6::movable_public::public_siblings_dispatch_and_cleanup_after_thread_handoffs'; Marker='Public movable: PASSED'; Trace=$true}
        )
    } elseif ($Mode -eq 'd013') {
        @(
            @{Name='d013-control-abc'; Test='upscaler::dx12::m6::thread_transfer_baseline::same_thread_load_create_dispatch_and_cleanup'; Marker='D013 scenario=1 transfer=false: PASSED'},
            @{Name='d013-transfer-abc'; Test='upscaler::dx12::m6::thread_transfer_baseline::load_create_then_dispatch_and_cleanup_on_other_threads'; Marker='D013 scenario=1 transfer=true: PASSED'},
            @{Name='d013-control-three-frames'; Test='upscaler::dx12::m6::thread_transfer_baseline::same_thread_three_moving_frames'; Marker='D013 scenario=2 transfer=false: PASSED'},
            @{Name='d013-transfer-three-frames'; Test='upscaler::dx12::m6::thread_transfer_baseline::three_moving_frames_on_successive_threads'; Marker='D013 scenario=2 transfer=true: PASSED'},
            @{Name='d013-control-siblings'; Test='upscaler::dx12::m6::thread_transfer_baseline::same_thread_siblings_survive_runtime_release'; Marker='D013 scenario=3 transfer=false: PASSED'},
            @{Name='d013-transfer-siblings'; Test='upscaler::dx12::m6::thread_transfer_baseline::siblings_survive_runtime_release_and_thread_transfers'; Marker='D013 scenario=3 transfer=true: PASSED'}
        )
    } elseif ($Mode -eq 'd013-cleanup') {
        @(
            @{Name='native-explicit'; Test='upscaler::dx12::tests::native_lifecycle'; Marker='Production owner: destroy A/B OK'},
            @{Name='native-drop'; Test='upscaler::dx12::tests::native_lifecycle_drop'; Marker='Production owner: destroy A/B OK'},
            @{Name='gate-a'; Test='upscaler::dx12::m6::native_dispatch_readback'; Marker='Gate A: PASSED'},
            @{Name='gate-b'; Test='upscaler::dx12::m6::native_temporal_smoke'; Marker='Gate B: PASSED'},
            @{Name='gate-c'; Test='upscaler::dx12::m6::native_semantic_comparisons'; Marker='Gate C: PASSED'},
            @{Name='d013-control-abc'; Test='upscaler::dx12::m6::thread_transfer_baseline::same_thread_load_create_dispatch_and_cleanup'; Marker='D013 scenario=1 transfer=false: PASSED'},
            @{Name='d013-transfer-abc'; Test='upscaler::dx12::m6::thread_transfer_baseline::load_create_then_dispatch_and_cleanup_on_other_threads'; Marker='D013 scenario=1 transfer=true: PASSED'},
            @{Name='d013-control-three-frames'; Test='upscaler::dx12::m6::thread_transfer_baseline::same_thread_three_moving_frames'; Marker='D013 scenario=2 transfer=false: PASSED'},
            @{Name='d013-transfer-three-frames'; Test='upscaler::dx12::m6::thread_transfer_baseline::three_moving_frames_on_successive_threads'; Marker='D013 scenario=2 transfer=true: PASSED'},
            @{Name='d013-control-siblings'; Test='upscaler::dx12::m6::thread_transfer_baseline::same_thread_siblings_survive_runtime_release'; Marker='D013 scenario=3 transfer=false: PASSED'},
            @{Name='d013-transfer-siblings'; Test='upscaler::dx12::m6::thread_transfer_baseline::siblings_survive_runtime_release_and_thread_transfers'; Marker='D013 scenario=3 transfer=true: PASSED'}
            $cases = @(
                @{Case=1; Test='create_after_transfer'; ControlTest='same_thread_runtime_transfer_before_context_creation'; TransferTest='runtime_transfer_before_context_creation_on_another_thread'; Payloads=1},
                @{Case=2; Test='history'; ControlTest='same_thread_continuous_history_across_ownership_handoffs'; TransferTest='continuous_history_across_worker_thread_transfers'; Payloads=6},
                @{Case=3; Test='independent_siblings'; ControlTest='same_thread_independent_siblings_share_cleanup_coordination'; TransferTest='independent_siblings_share_cleanup_coordination_across_threads'; Payloads=5},
                @{Case=4; Test='unwind'; ControlTest='same_thread_automatic_context_cleanup_during_unwind'; TransferTest='transferred_context_automatic_cleanup_during_unwind'; Payloads=1},
                @{Case=5; Test='sibling_unwind'; ControlTest='same_thread_sibling_cleanup_during_unwind_preserves_survivor'; TransferTest='transferred_sibling_cleanup_during_unwind_preserves_survivor'; Payloads=3}
            )
            foreach ($case in $cases) {
                foreach ($arm in @('control', 'transfer')) {
                    foreach ($repeat in 1..3) {
                        $moving = if ($arm -eq 'transfer') { 'true' } else { 'false' }
                        $testName = if ($arm -eq 'transfer') { $case.TransferTest } else { $case.ControlTest }
                        @{Name="d013-2-$($case.Test)-$arm-r$repeat";
                          Test="upscaler::dx12::m6::thread_transfers::$testName";
                          Marker="case=$($case.Case) transfer=$($moving): PASSED";
                          NewCase=$true; Case=$case.Test; Arm=$arm; Repeat=$repeat; Payloads=$case.Payloads}
                    }
                }
            }
        )
    } elseif ($Mode -eq 'm9') {
        @(
            @{Name='native-explicit'; Test='upscaler::dx12::tests::native_lifecycle'; Marker='Production owner: destroy A/B OK'},
            @{Name='native-drop'; Test='upscaler::dx12::tests::native_lifecycle_drop'; Marker='Production owner: destroy A/B OK'},
            @{Name='gate-a'; Test='upscaler::dx12::m6::native_dispatch_readback'; Marker='Gate A: PASSED'},
            @{Name='gate-b'; Test='upscaler::dx12::m6::native_temporal_smoke'; Marker='Gate B: PASSED'},
            @{Name='gate-c'; Test='upscaler::dx12::m6::native_semantic_comparisons'; Marker='Gate C: PASSED'}
            foreach ($repeat in 1..3) {
                @{Name="m9-repeat-$repeat"; Test='upscaler::dx12::m6::native_sharpening_integration';
                  Marker='M9 integration: PASSED outputs=48'; M9=$true; Repeat=$repeat}
            }
        )
    } elseif ($Mode -eq 'sharpening') {
        @(1..3 | ForEach-Object {
            @{Name="sharpening-repeat-$_"; Test='upscaler::dx12::m6::native_sharpening_comparisons'; Marker='Sharpening gate: PASSED'}
        })
    } else {
        @(
            @{Name='gate-a'; Test='upscaler::dx12::m6::native_dispatch_readback'; Marker='Gate A: PASSED'},
            @{Name='gate-b'; Test='upscaler::dx12::m6::native_temporal_smoke'; Marker='Gate B: PASSED'}
            @{Name='gate-c'; Test='upscaler::dx12::m6::native_semantic_comparisons'; Marker='Gate C: PASSED'}
        )
    }
    $payloadResults = @()
    $newMatrixStarted = $false
    foreach ($gate in $gates) {
        if ($gate.M9 -and $failed.Count) { throw "M9 prerequisite failed: $($failed -join '; ')" }
        if ($gate.NewCase -and -not $newMatrixStarted) {
            if ($failed.Count) { throw "D013-2 prerequisite failed: $($failed -join '; ')" }
            $newMatrixStarted = $true
        }
        $name = $gate.Name
        [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/$name.status.txt"), 'not_started')
        $start = New-Object System.Diagnostics.ProcessStartInfo
        $start.FileName = (Resolve-Path "$stage/m6.exe").Path
        $start.Arguments = "--ignored --exact $($gate.Test) --nocapture --test-threads=1"
        $start.UseShellExecute = $false
        $start.CreateNoWindow = $true
        $start.RedirectStandardOutput = $true
        $start.RedirectStandardError = $true
        if ($Mode -eq 'sharpening') {
            $imageDirectory = New-Item -ItemType Directory -Force "$stage/$name.images"
            $start.Environment['FSR_SDK_TEST_SHARPENING_OUTPUT'] = $imageDirectory.FullName
        }
        if ($gate.M9) {
            $imageDirectory = New-Item -ItemType Directory -Force "$stage/$name.images"
            if (Get-ChildItem -LiteralPath $imageDirectory.FullName -File) {
                throw "M9 requires fresh output: $name"
            }
            $start.Environment['FSR_SDK_TEST_M9_OUTPUT'] = $imageDirectory.FullName
        }
        if ($gate.NewCase) {
            $imageDirectory = New-Item -ItemType Directory -Force "$stage/$name.images"
            if (Get-ChildItem -LiteralPath $imageDirectory.FullName -File) {
                throw "Use a fresh stage: payload directory already populated: $name"
            }
            $start.Environment['FSR_SDK_TEST_D013_OUTPUT'] = $imageDirectory.FullName
            $start.Environment['FSR_SDK_TEST_D013_TRACE'] = '1'
        }
        if ($gate.Trace) { $start.Environment['FSR_SDK_TEST_D013_TRACE'] = '1' }
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
            if ($gate.M9) { throw "M9 failed; preserved attempt: $($failed -join '; ')" }
            if ($gate.NewCase) { throw "D013-2 case failed; preserved attempt: $($failed -join '; ')" }
        }
        if ($gate.M9) {
            $payloads = @(Get-ChildItem -LiteralPath $imageDirectory.FullName -Filter '*.red.u16le' -File |
                Sort-Object Name | ForEach-Object {
                    [pscustomobject]@{File=$_.Name; Bytes=$_.Length; SHA256=(Get-FileHash -LiteralPath $_.FullName).Hash}
                })
            if ($payloads.Count -ne 48 -or ($payloads | Where-Object Bytes -ne 460800)) {
                throw "M9 payload oracle failed: $name"
            }
            $payloadResults += [pscustomobject]@{Repeat=$gate.Repeat; Files=$payloads}
            [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/payload-hashes.json"), ($payloadResults | ConvertTo-Json -Depth 6))
        }
        if ($gate.NewCase) {
            $payloads = @(Get-ChildItem -LiteralPath $imageDirectory.FullName -Filter '*.rgba16f' -File |
                Sort-Object Name | ForEach-Object {
                    [pscustomobject]@{File=$_.Name; Bytes=$_.Length; SHA256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash}
                })
            if ($payloads.Count -ne $gate.Payloads -or ($payloads | Where-Object Bytes -ne 1843200)) {
                throw "D013-2 payload oracle failed: $name"
            }
            $payloadResults += [pscustomobject]@{Case=$gate.Case; Arm=$gate.Arm; Repeat=$gate.Repeat; Files=$payloads}
            [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/payload-hashes.json"), ($payloadResults | ConvertTo-Json -Depth 6))
        }
    }
    if ($Mode -eq 'd013-cleanup') {
        $comparisons = foreach ($caseName in ($payloadResults.Case | Sort-Object -Unique)) {
            $controls = @($payloadResults | Where-Object { $_.Case -eq $caseName -and $_.Arm -eq 'control' } | Sort-Object Repeat)
            $transfers = @($payloadResults | Where-Object { $_.Case -eq $caseName -and $_.Arm -eq 'transfer' } | Sort-Object Repeat)
            if ($controls.Count -ne 3 -or $transfers.Count -ne 3) { throw "Incomplete D013-2 repetitions: $caseName" }
            $controlReference = $controls[0].Files | ConvertTo-Json -Depth 4 -Compress
            $controlStable = @($controls | Where-Object { ($_.Files | ConvertTo-Json -Depth 4 -Compress) -ne $controlReference }).Count -eq 0
            $transferReference = $transfers[0].Files | ConvertTo-Json -Depth 4 -Compress
            $transferStable = @($transfers | Where-Object { ($_.Files | ConvertTo-Json -Depth 4 -Compress) -ne $transferReference }).Count -eq 0
            $equal = @($transfers | Where-Object { ($_.Files | ConvertTo-Json -Depth 4 -Compress) -ne $controlReference }).Count -eq 0
            $result = if (-not $controlStable) { 'inconclusive-controls-vary' } elseif ($equal) { 'exact-payload-equivalence' } else { 'different-payloads' }
            [pscustomobject]@{Case=$caseName; ControlRepeatable=$controlStable; TransferRepeatable=$transferStable; AllMatchControl=$equal; ImageResult=$result}
        }
        [IO.File]::WriteAllText((Join-Path (Get-Location) "$stage/payload-comparison.json"), ($comparisons | ConvertTo-Json))
        Write-Host ($comparisons | Format-Table | Out-String)
    }
    if ($failed.Count) { throw "Windows verification failed: $($failed -join '; ')" }
} finally {
    $env:FSR_SDK_TEST_DLL = $previousLoader
    $env:FSR_SDK_TEST_M6_DLL = $previousHelper
    $env:FSR_SDK_TEST_DEVICE_DLL = $previousDeviceHelper
    $env:FSR_SDK_TEST_GBV = $previousGpuValidation
    $env:FSR_SDK_TEST_M6_CONTROL = $previousControl
    if ($transcriptStarted) { Stop-Transcript | Out-Null }
    Pop-Location
}
