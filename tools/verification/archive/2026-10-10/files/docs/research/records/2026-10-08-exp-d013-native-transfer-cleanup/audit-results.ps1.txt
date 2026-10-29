# SPDX-License-Identifier: MPL-2.0

# Preserve sanitized observations and independently audit the completed matrix.
param([string]$Stage = 'target/d013-native-cleanup/attempt-1')
$ErrorActionPreference = 'Stop'
$repo = (Get-Location).Path
$destination = $PSScriptRoot
$root = (Resolve-Path -LiteralPath $Stage).Path

function Normalize-Text([string]$value) {
    $value = $value.Replace($repo, '<repo>').Replace($repo.Replace('\','/'), '<repo>')
    $value = $value.Replace($env:USERPROFILE, '<user>').Replace($env:USERPROFILE.Replace('\','/'), '<user>')
    $value = $value -replace "(thread '[^']+') \(\d+\)", '$1 (<os-thread>)'
    return $value
}

foreach ($name in @('source-inputs.json','source-state.txt','native-inputs.json',
                    'native.environment.txt','checks.status.txt','payload-hashes.json','payload-comparison.json')) {
    $value = Get-Content -LiteralPath (Join-Path $root $name) -Raw
    [IO.File]::WriteAllText((Join-Path $destination $name), (Normalize-Text $value))
}
$console = Get-Content -LiteralPath "$root.console.txt" -Raw
$firstChild = $console.IndexOf('Child command:')
if ($firstChild -lt 0) { throw 'No native child execution' }
[IO.File]::WriteAllText((Join-Path $destination 'ordinary-checks.txt'), (Normalize-Text $console.Substring(0,$firstChild)))

$sourceInputs = Get-Content -LiteralPath "$root/source-inputs.json" -Raw | ConvertFrom-Json
$changed = @($sourceInputs | Where-Object { (Get-FileHash -LiteralPath $_.File -Algorithm SHA256).Hash -ne $_.SHA256 })
if ($changed.Count) { throw "Executed sources changed: $($changed.File -join ', ')" }
$normalizedInputs = foreach ($inputFile in $sourceInputs) {
    $content = [IO.File]::ReadAllText((Join-Path $repo $inputFile.File)).Replace("`r`n", "`n")
    $hash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData([Text.Encoding]::UTF8.GetBytes($content)))
    [pscustomobject]@{File=$inputFile.File.Replace('\','/'); RawSHA256=$inputFile.SHA256; UTF8LFSHA256=$hash}
}
$normalizedInputs | ConvertTo-Json | Set-Content -LiteralPath "$destination/source-fingerprints.json" -Encoding utf8

$summary = foreach ($status in (Get-ChildItem -LiteralPath $root -Filter '*.status.txt' -File | Sort-Object Name)) {
    if ($status.Name -eq 'checks.status.txt') { continue }
    $name = $status.Name.Replace('.status.txt','')
    $state = Get-Content -LiteralPath $status.FullName -Raw
    $stderr = Get-Content -LiteralPath "$root/$name.stderr.txt" -Raw
    $stdout = Get-Content -LiteralPath "$root/$name.stdout.txt" -Raw
    foreach ($suffix in @('status','stderr','stdout')) {
        $text = Get-Content -LiteralPath "$root/$name.$suffix.txt" -Raw
        [IO.File]::WriteAllText((Join-Path $destination "$name.$suffix.txt"), (Normalize-Text $text))
    }
    if ($state -notmatch 'timeout=False exit=0' -or $stdout -notmatch '1 passed; 0 failed') {
        throw "Unsuccessful native child: $name"
    }
    $new = $name.StartsWith('d013-2-')
    $frames = [regex]::Matches($stderr, 'D013-2 fenced context=(\d+) frame=(\d+) completed=(\d+) metrics=.*finite_pixels: (\d+), changed_pixels: (\d+).*debug_errors: (\d+), device_removed: (-?\d+)')
    foreach ($frame in $frames) {
        if ($frame.Groups[3].Value -ne '1' -or $frame.Groups[4].Value -ne '230400' -or
            [int]$frame.Groups[5].Value -le 115200 -or $frame.Groups[6].Value -ne '0' -or
            $frame.Groups[7].Value -ne '0') { throw "Frame oracle failed: $name" }
    }
    $destroys = [regex]::Matches($stderr, 'D013 native destroy: thread=ThreadId\((\d+)\) code=(0x[0-9a-f]+)')
    if (@($destroys | Where-Object { $_.Groups[2].Value -ne '0x00000000' }).Count) { throw "Destroy failed: $name" }
    $unwinds = [regex]::Matches($stderr, 'automatic Drop begin context=\d+ domain_held=true unwinding=true')
    $handoffs = [regex]::Matches($stderr, 'handoff context=\d+ from=ThreadId\((\d+)\).*thread=ThreadId\((\d+)\)')
    $crossThread = @($handoffs | Where-Object { $_.Groups[1].Value -ne $_.Groups[2].Value }).Count
    $checkpoint5 = [regex]::Match($stderr, 'residency checkpoint=5-[^\r\n]+').Value
    $checkpoint6 = [regex]::Match($stderr, 'residency checkpoint=6-[^\r\n]+').Value
    if ($new -and ($checkpoint5 -notmatch 'runtime_strong=Some\(0\) loader=false provider=(true|false) helper=false' -or
                  $checkpoint6 -notmatch 'runtime_strong=Some\(0\) loader=false provider=(true|false) helper=false')) {
        throw "Application cleanup observation failed: $name"
    }
    [pscustomobject]@{Child=$name; Exit=0; Timeout=$false; NewCase=$new;
        FencedFrames=$frames.Count; NativeDestroyOK=$destroys.Count;
        ContextHandoffs=$handoffs.Count; CrossThreadHandoffs=$crossThread; UnwindDrops=$unwinds.Count;
        FinalResidency=$checkpoint5; BeforeExitResidency=$checkpoint6}
}
$summary | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath "$destination/case-outcomes.json" -Encoding utf8
$summary | Select-Object Child,Exit,Timeout,FencedFrames,NativeDestroyOK,ContextHandoffs,CrossThreadHandoffs,UnwindDrops |
    Export-Csv -LiteralPath "$destination/case-outcomes.csv" -NoTypeInformation
if ($summary.Count -ne 41 -or @($summary | Where-Object NewCase).Count -ne 30) { throw 'Incomplete native matrix' }
$newCases = @($summary | Where-Object NewCase)
$payloadInputs = Get-Content -LiteralPath "$root/payload-hashes.json" -Raw | ConvertFrom-Json
$payloadCount = 0
foreach ($payloadCase in $payloadInputs) {
    $payloadDirectory = Join-Path $root "d013-2-$($payloadCase.Case)-$($payloadCase.Arm)-r$($payloadCase.Repeat).images"
    foreach ($payload in $payloadCase.Files) {
        $file = Join-Path $payloadDirectory $payload.File
        if ((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash -ne $payload.SHA256 -or
            (Get-Item -LiteralPath $file).Length -ne $payload.Bytes) { throw "Payload changed: $file" }
        ++$payloadCount
    }
}
if ($payloadCount -ne 96) { throw 'Incomplete selected payloads' }
[pscustomobject]@{NativeChildren=$summary.Count; NewChildren=$newCases.Count;
    FencedFrames=($newCases.FencedFrames | Measure-Object -Sum).Sum;
    DestroyOK=($newCases.NativeDestroyOK | Measure-Object -Sum).Sum;
    UnwindDrops=($newCases.UnwindDrops | Measure-Object -Sum).Sum;
    ExecutedSourceInputs=$sourceInputs.Count; SourcesUnchanged=$true; SelectedPayloadsVerified=$payloadCount} | ConvertTo-Json |
    Set-Content -LiteralPath "$destination/audit-summary.json" -Encoding utf8
Get-Content -LiteralPath "$destination/audit-summary.json"
