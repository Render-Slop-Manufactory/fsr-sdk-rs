# SPDX-License-Identifier: MPL-2.0

# Bounded diagnosis: same DX12 harness with and without native FSR dispatch.
# Run in an x64 VS developer shell with trusted SDK v2.3.0 staged locally.
$ErrorActionPreference = 'Stop'
Push-Location (Resolve-Path (Join-Path $PSScriptRoot '../../..'))
try {
    & "$PSScriptRoot/run-m6.ps1" -Native
    if ($LASTEXITCODE -ne 0) { throw 'M6 baseline/staging failed' }
    $stage = (Resolve-Path 'target/m6a').Path
    $output = Join-Path (Get-Location) 'target/m6-gbv-spike'
    New-Item -ItemType Directory -Force $output | Out-Null

    foreach ($case in @(
        @{ Name = 'control'; Control = $true; TimeoutSeconds = 45; Marker = 'DX12 control readback:' },
        @{ Name = 'dispatch'; Control = $false; TimeoutSeconds = 90; Marker = 'GPU readback:' }
    )) {
        $start = [System.Diagnostics.ProcessStartInfo]::new()
        $start.FileName = (Join-Path $stage 'm6.exe')
        $start.Arguments = '--ignored --exact upscaler::m6::native_dispatch_readback --nocapture --test-threads=1'
        $start.UseShellExecute = $false
        $start.CreateNoWindow = $true
        $start.RedirectStandardOutput = $true
        $start.RedirectStandardError = $true
        $start.Environment['FSR_SDK_TEST_DLL'] = (Join-Path $stage 'amd_fidelityfx_loader_dx12.dll')
        $start.Environment['FSR_SDK_TEST_M6_DLL'] = (Join-Path $stage 'm6_gpu.dll')
        $start.Environment['FSR_SDK_TEST_GBV'] = '1'
        if ($case.Control) { $start.Environment['FSR_SDK_TEST_M6_CONTROL'] = '1' }
        else { [void]$start.Environment.Remove('FSR_SDK_TEST_M6_CONTROL') }

        $child = [System.Diagnostics.Process]::Start($start)
        $stdout = $child.StandardOutput.ReadToEndAsync()
        $stderr = $child.StandardError.ReadToEndAsync()
        $samples = @()
        $elapsed = 0
        while ($elapsed -lt $case.TimeoutSeconds -and !$child.WaitForExit(10000)) {
            $elapsed += 10
            $child.Refresh()
            $samples += "elapsed_seconds=$elapsed cpu_seconds=$($child.TotalProcessorTime.TotalSeconds)"
        }
        $timedOut = !$child.HasExited
        if ($timedOut) { $child.Kill($true) }
        $child.WaitForExit()
        $exitCode = $child.ExitCode
        $child.Dispose()
        [IO.File]::WriteAllText((Join-Path $output "$($case.Name).stdout.txt"), $stdout.Result)
        [IO.File]::WriteAllText((Join-Path $output "$($case.Name).stderr.txt"), $stderr.Result)
        [IO.File]::WriteAllText((Join-Path $output "$($case.Name).status.txt"),
            "gpu_validation=True`r`ncontrol=$($case.Control)`r`ntimeout=$timedOut`r`nexit=$exitCode`r`n$($samples -join "`r`n")")
        Write-Host "$($case.Name): timeout=$timedOut exit=$exitCode"
        Write-Host $stderr.Result
        if ($case.Control -and ($timedOut -or $exitCode -ne 0 -or $stderr.Result -notmatch $case.Marker)) {
            throw 'GBV control failed; provider dispatch case skipped'
        }
    }
} finally {
    Pop-Location
}
