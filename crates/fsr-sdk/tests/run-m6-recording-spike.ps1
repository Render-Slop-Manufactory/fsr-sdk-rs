# SPDX-License-Identifier: MPL-2.0

# Run in an x64 VS developer shell. Builds and verifies the M6a control first.
$ErrorActionPreference = 'Stop'
& (Join-Path $PSScriptRoot 'run-m6.ps1') -Native
if ($LASTEXITCODE -ne 0) { throw 'M6a control failed' }

$root = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$stage = Join-Path $root 'target/m6a'
$previousLoader = $env:FSR_SDK_TEST_DLL
$previousHelper = $env:FSR_SDK_TEST_M6_DLL
$previousCount = $env:FSR_SDK_TEST_RECORD_COUNT
$previousMode = $env:FSR_SDK_TEST_RECORD_MODE
$previousGpuValidation = $env:FSR_SDK_TEST_GBV
try {
    $env:FSR_SDK_TEST_DLL = (Resolve-Path (Join-Path $stage 'amd_fidelityfx_loader_dx12.dll')).Path
    $env:FSR_SDK_TEST_M6_DLL = (Resolve-Path (Join-Path $stage 'm6_gpu.dll')).Path
    Remove-Item Env:FSR_SDK_TEST_GBV -ErrorAction SilentlyContinue
    $exe = (Resolve-Path (Join-Path $stage 'm6.exe')).Path
    $cases = @(
        @{ Mode = 'same'; Count = 1 },
        @{ Mode = 'same'; Count = 2 },
        @{ Mode = 'same'; Count = 5 },
        @{ Mode = 'same'; Count = 8 },
        @{ Mode = 'sibling'; Count = 2 },
        @{ Mode = 'sibling'; Count = 5 }
    )
    $failed = @()
    foreach ($case in $cases) {
        $mode = $case.Mode
        $count = $case.Count
        $env:FSR_SDK_TEST_RECORD_MODE = $mode
        $env:FSR_SDK_TEST_RECORD_COUNT = [string]$count
        $label = "recording-$mode-$count"
        Write-Host "Running $label"
        $start = New-Object System.Diagnostics.ProcessStartInfo
        $start.FileName = $exe
        $start.Arguments = '--ignored --exact upscaler::m6_recording::native_recording_ahead_spike --nocapture --test-threads=1'
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
        [IO.File]::WriteAllText((Join-Path $stage "$label.stdout.txt"), $stdout.Result)
        [IO.File]::WriteAllText((Join-Path $stage "$label.stderr.txt"), $stderr.Result)
        [IO.File]::WriteAllText((Join-Path $stage "$label.status.txt"), "timeout=$timeout exit=$exitCode")
        Write-Host $stdout.Result
        Write-Host $stderr.Result
        if ($timeout -or $exitCode -ne 0 -or $stderr.Result -notmatch 'GPU readback') {
            $failed += $label
        }
    }
    if ($failed.Count -gt 0) { throw "Recording spike failed: $($failed -join ', ')" }
} finally {
    $env:FSR_SDK_TEST_DLL = $previousLoader
    $env:FSR_SDK_TEST_M6_DLL = $previousHelper
    $env:FSR_SDK_TEST_RECORD_COUNT = $previousCount
    $env:FSR_SDK_TEST_RECORD_MODE = $previousMode
    $env:FSR_SDK_TEST_GBV = $previousGpuValidation
}
