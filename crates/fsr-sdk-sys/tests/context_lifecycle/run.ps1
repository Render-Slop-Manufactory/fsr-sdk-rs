# SPDX-License-Identifier: MPL-2.0

# Run from an x64 VS 2022 developer shell. No SDK downloads or installation.
param([switch]$FailureInjection, [switch]$InputLifetime)
$ErrorActionPreference = 'Stop'
# The ordinary lifecycle now exercises the production wrapper owner.
if (!$FailureInjection -and !$InputLifetime) {
    & (Join-Path $PSScriptRoot '../../../fsr-sdk/tests/run-m4.ps1') -Native
    if (!$?) { throw 'M4 lifecycle checks failed' }
    exit 0
}
Push-Location (Resolve-Path (Join-Path $PSScriptRoot '../../../..'))
$previousLoader = $env:FSR_SDK_TEST_DLL
$previousHelper = $env:FSR_SDK_TEST_DEVICE_DLL
$previousFailure = $env:FSR_SDK_FAIL_AT
$previousLifetime = $env:FSR_SDK_INPUT_LIFETIME

function Invoke-InputProbe([string]$Mode) {
    $env:FSR_SDK_INPUT_LIFETIME = if ($Mode -eq 'control') { $null } else { $Mode }
    $test = if ($Mode -eq 'control') { 'upscaler::tests::native_lifecycle' } else { 'probes_create_error_input_lifetime' }
    $start = New-Object System.Diagnostics.ProcessStartInfo
    $executable = if ($Mode -eq 'control') { "$stage/m4.exe" } else { "$stage/context_lifecycle.exe" }
    $start.FileName = (Resolve-Path $executable).Path
    $start.Arguments = "--ignored --exact $test --nocapture --test-threads=1"
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $child = [System.Diagnostics.Process]::Start($start)
    $outputTask = $child.StandardOutput.ReadToEndAsync()
    $errorTask = $child.StandardError.ReadToEndAsync()
    $timedOut = !$child.WaitForExit(15000)
    if ($timedOut) { $child.Kill() }
    $child.WaitForExit()
    $unsigned = [BitConverter]::ToUInt32([BitConverter]::GetBytes([int]$child.ExitCode), 0)
    $status = "CASE mode=$Mode timeout=$timedOut exit_signed=$($child.ExitCode) exit_unsigned=$unsigned exit_hex=0x$($unsigned.ToString('X8'))"
    $base = Join-Path (Get-Location) "$stage/input-$Mode"
    [IO.File]::WriteAllText("$base.stdout.txt", $outputTask.Result)
    [IO.File]::WriteAllText("$base.stderr.txt", $errorTask.Result)
    [IO.File]::WriteAllText("$base.status.txt", $status)
    $observations = @($status) + @($errorTask.Result -split "`r?`n")
    $observations | ForEach-Object { Write-Host $_ }
    $child.Dispose()
    return $observations
}

function Invoke-FailureProbe([int]$Index) {
    $env:FSR_SDK_FAIL_AT = "$Index"
    $stdout = Join-Path $stage "failure-$Index.stdout.txt"
    $stderr = Join-Path $stage "failure-$Index.stderr.txt"
    $start = New-Object System.Diagnostics.ProcessStartInfo
    $start.FileName = (Resolve-Path "$stage/context_lifecycle.exe").Path
    $start.Arguments = '--ignored --exact probes_create_allocation_failure --nocapture --test-threads=1'
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $child = [System.Diagnostics.Process]::Start($start)
    $outputTask = $child.StandardOutput.ReadToEndAsync()
    $errorTask = $child.StandardError.ReadToEndAsync()
    if (!$child.WaitForExit(60000)) {
        $child.Kill()
        $child.WaitForExit()
        $status = "CASE fail_at=$Index timeout=true"
    } else {
        $child.WaitForExit()
        $status = "CASE fail_at=$Index exit_code=$($child.ExitCode)"
    }
    Write-Host $status
    Set-Content -LiteralPath (Join-Path $stage "failure-$Index.status.txt") -Value $status
    [System.IO.File]::WriteAllText((Join-Path (Get-Location) $stdout), $outputTask.Result)
    [System.IO.File]::WriteAllText((Join-Path (Get-Location) $stderr), $errorTask.Result)
    $observations = Get-Content -LiteralPath $stderr
    $child.Dispose()
    $observations | ForEach-Object { Write-Host $_ }
    return @($status) + $observations
}
try {
    if ($FailureInjection -and $InputLifetime) { throw 'Select one experiment' }
    $env:FSR_SDK_INPUT_LIFETIME = $null
    $stage = 'target/context-lifecycle'
    New-Item -ItemType Directory -Force $stage | Out-Null
    & cl /nologo /std:c++17 /W4 /WX /c /Iexternal/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/api/include crates/fsr-sdk-sys/tests/context_lifecycle/native_abi.cpp /Fotarget/context-lifecycle/native_abi.obj
    if ($LASTEXITCODE -ne 0) { throw 'Native ABI verification failed' }
    & cl /nologo /std:c++17 /W4 /WX /MT /LD crates/fsr-sdk-sys/tests/context_lifecycle/device.cpp /Fotarget/context-lifecycle/device.obj /link /OUT:target/context-lifecycle/device.dll /IMPLIB:target/context-lifecycle/device.lib d3d12.lib dxgi.lib
    if ($LASTEXITCODE -ne 0) { throw 'Device helper build failed' }
    $messages = & cargo test -p fsr-sdk-sys --features dx12 --test context_lifecycle --locked --offline --no-run --message-format=json
    if ($LASTEXITCODE -ne 0) { throw 'Rust probe build failed' }
    $artifact = $messages | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'context_lifecycle' -and $_.executable } | Select-Object -Last 1
    if (!$artifact) { throw 'No probe executable produced' }
    Copy-Item -LiteralPath $artifact.executable -Destination "$stage/context_lifecycle.exe"
    Copy-Item external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/signedbin/amd_fidelityfx_loader_dx12.dll,external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/signedbin/amd_fidelityfx_upscaler_dx12.dll -Destination $stage
    $env:FSR_SDK_TEST_DLL = (Resolve-Path "$stage/amd_fidelityfx_loader_dx12.dll").Path
    $env:FSR_SDK_TEST_DEVICE_DLL = (Resolve-Path "$stage/device.dll").Path
    if ($InputLifetime) {
        $wrapperMessages = & cargo test -p fsr-sdk --lib --features dx12 --locked --offline --no-run --message-format=json
        if ($LASTEXITCODE -ne 0) { throw 'Wrapper test build failed' }
        $wrapper = $wrapperMessages | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'fsr_sdk' -and $_.executable } | Select-Object -Last 1
        if (!$wrapper) { throw 'No wrapper test executable' }
        Copy-Item -LiteralPath $wrapper.executable -Destination "$stage/m4.exe"
        $baseline = Invoke-InputProbe 'control'
        if (!($baseline -match '^CASE .*timeout=False exit_signed=0 ') -or !($baseline -match '^Production owner: destroy OK$')) {
            throw 'Successful lifecycle control required'
        }
        $retained = Invoke-InputProbe 'retain'
        if (!($retained -match '^CASE .*timeout=False exit_signed=0 ') -or !($retained -match '^Create: return_code=[1-9][0-9]*;') -or !($retained -match '^Observation completed:')) {
            throw 'No clean returned-error baseline; do not shorten lifetimes'
        }
        foreach ($mode in @('protect', 'cleanup')) { $null = Invoke-InputProbe $mode }
    } elseif ($FailureInjection) {
        $baseline = Invoke-FailureProbe 0
        $summary = $baseline | Where-Object { $_ -match '^SUMMARY phase=after_create ' } | Select-Object -Last 1
        if (!($baseline -match '^CASE fail_at=0 exit_code=0$') -or !$summary -or !($baseline -match '^Destroy: return_code=0;') -or !($baseline -match '^SUMMARY phase=after_destroy .*all_freed_once=true$')) {
            throw 'Instrumented baseline did not complete cleanly; do not inject failures'
        }
        if ($summary -notmatch 'attempts=(\d+)') { throw 'Missing baseline allocation count' }
        $count = [int]$Matches[1]
        if ($count -gt 64) { throw 'More than 64 callback allocations; refine the bounded sweep before running' }
        # The extra index is an unreached control. Crashes are observations and
        # must not prevent later independent processes from being investigated.
        foreach ($index in 1..($count + 1)) { $null = Invoke-FailureProbe $index }
    }
} finally {
    $env:FSR_SDK_TEST_DLL = $previousLoader
    $env:FSR_SDK_TEST_DEVICE_DLL = $previousHelper
    $env:FSR_SDK_FAIL_AT = $previousFailure
    $env:FSR_SDK_INPUT_LIFETIME = $previousLifetime
    Pop-Location
}
