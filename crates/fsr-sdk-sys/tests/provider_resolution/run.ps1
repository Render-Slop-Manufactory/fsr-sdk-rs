# SPDX-License-Identifier: MPL-2.0

# Bounded signed-v2.3.0 experiment. Each run uses a new, never reused directory.
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../../../..')).Path
Push-Location $repo
try {
    $messages = & cargo test -p fsr-sdk-sys --features dx12 --test query --locked --offline --no-run --message-format=json
    if ($LASTEXITCODE -ne 0) { throw 'Query build failed' }
    $artifact = $messages | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'query' -and $_.executable } | Select-Object -Last 1
    if (!$artifact) { throw 'No query executable' }
    $stage = Join-Path $repo ('target/provider-resolution/' + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $stage | Out-Null
    $sdk = Join-Path $repo 'external/FidelityFX-SDK/v2.3.0/Kits/FidelityFX/signedbin'
    $loader = 'amd_fidelityfx_loader_dx12.dll'
    $provider = 'amd_fidelityfx_upscaler_dx12.dll'
    # Inspect inherited search locations without retaining personal PATH entries.
    # Relative entries are evaluated against each child's deliberately empty cwd.
    $pathEntries = @($env:PATH -split ';')
    $absoluteEntries = @($pathEntries | Where-Object { $_ -and [IO.Path]::IsPathRooted($_.Trim('"')) })
    $hits = @($absoluteEntries | Where-Object { Test-Path -LiteralPath (Join-Path $_.Trim('"') $provider) })
    $systemHits = @(@("$env:SystemRoot/System32", "$env:SystemRoot/System", $env:SystemRoot) | Where-Object { Test-Path -LiteralPath (Join-Path $_ $provider) })
    if ($hits.Count -or $systemHits.Count) { throw 'Provider found in inherited PATH or Windows directories; resolve contamination before running' }
    $metadata = @("PATH entries=$($pathEntries.Count); rooted=$($absoluteEntries.Count); provider hits=0; Windows-directory hits=0", 'Child PATH inherited unchanged; no DLL search-directory APIs called by probe/runner.', 'Child-only inputs: FSR_SDK_TEST_DLL=absolute staged loader; FSR_SDK_RESOLUTION_DIAGNOSTICS=1; RUST_BACKTRACE=0.')
    foreach ($name in @($loader, $provider)) {
        $file = Join-Path $sdk $name
        $sig = Get-AuthenticodeSignature -LiteralPath $file
        $metadata += "$name SHA256=$((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash); signature=$($sig.Status); signer=$($sig.SignerCertificate.Subject)"
        if ($sig.Status -ne 'Valid') { throw "Signature verification failed: $name" }
    }
    $metadata | Set-Content -LiteralPath (Join-Path $stage 'environment.txt')
    foreach ($case in @('a', 'b', 'c', 'd')) {
        $base = Join-Path $stage "case-$case"
        foreach ($dir in @('app', 'runtime', 'cwd')) { New-Item -ItemType Directory -Path (Join-Path $base $dir) | Out-Null }
        Copy-Item -LiteralPath $artifact.executable -Destination "$base/app/query.exe"
        $loaderDir = if ($case -eq 'a') { 'app' } else { 'runtime' }
        Copy-Item -LiteralPath (Join-Path $sdk $loader) -Destination "$base/$loaderDir/$loader"
        if ($case -ne 'd') {
            $providerDir = if ($case -eq 'b') { 'runtime' } else { 'app' }
            Copy-Item -LiteralPath (Join-Path $sdk $provider) -Destination "$base/$providerDir/$provider"
        }
        $start = New-Object System.Diagnostics.ProcessStartInfo
        $start.FileName = "$base/app/query.exe"
        $start.Arguments = '--ignored --exact queries_upscaler_version_count_without_context --nocapture --test-threads=1'
        $start.WorkingDirectory = "$base/cwd"
        $start.UseShellExecute = $false
        $start.CreateNoWindow = $true
        $start.RedirectStandardOutput = $true
        $start.RedirectStandardError = $true
        $start.EnvironmentVariables['FSR_SDK_TEST_DLL'] = "$base/$loaderDir/$loader"
        $start.EnvironmentVariables['FSR_SDK_RESOLUTION_DIAGNOSTICS'] = '1'
        $start.EnvironmentVariables['RUST_BACKTRACE'] = '0'
        $child = [Diagnostics.Process]::Start($start)
        $out = $child.StandardOutput.ReadToEndAsync()
        $err = $child.StandardError.ReadToEndAsync()
        $timeout = !$child.WaitForExit(60000)
        if ($timeout) { $child.Kill(); $child.WaitForExit() }
        $status = "case=$case exit_code=$($child.ExitCode) timeout=$timeout"
        $stdout = $out.Result.Replace($repo, '<repo>')
        $stderr = $err.Result.Replace($repo, '<repo>')
        [IO.File]::WriteAllText("$base/stdout.txt", $stdout)
        [IO.File]::WriteAllText("$base/stderr.txt", $stderr)
        @($status, "loader=$($start.EnvironmentVariables['FSR_SDK_TEST_DLL'].Replace($repo, '<repo>'))", "cwd=$($start.WorkingDirectory.Replace($repo, '<repo>'))", 'files:') + @(Get-ChildItem -LiteralPath $base -Recurse -File | Where-Object { $_.Extension -in '.dll', '.exe' } | ForEach-Object { $_.FullName.Substring($base.Length + 1) }) | Set-Content -LiteralPath "$base/status.txt"
        Write-Host $status
        Write-Host $stderr
        $child.Dispose()
        if ($case -eq 'a' -and ($timeout -or $stderr -notmatch 'return_code=0, upscaler_version_count=[1-9]' -or $status -notmatch 'exit_code=0 ')) { throw 'Baseline failed; later cases not run' }
    }
    Write-Host "Evidence directory: $stage"
} finally { Pop-Location }
