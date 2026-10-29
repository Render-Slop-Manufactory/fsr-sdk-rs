# SPDX-License-Identifier: MPL-2.0

# Compile-only, independent check against pinned AMD headers. No AMD DLLs.
param([string]$SdkRoot, [string]$Stage = 'target/native-abi')
$ErrorActionPreference = 'Stop'
Push-Location (Resolve-Path (Join-Path $PSScriptRoot '..'))
try {
    $manifest = Get-Content -LiteralPath "$PSScriptRoot/native-headers.json" -Raw | ConvertFrom-Json
    New-Item -ItemType Directory -Force $Stage | Out-Null
    if (-not $SdkRoot) {
        $SdkRoot = Join-Path $Stage 'headers'
        foreach ($header in $manifest.headers) {
            $destination = Join-Path $SdkRoot $header.path
            New-Item -ItemType Directory -Force (Split-Path $destination) | Out-Null
            $url = "https://raw.githubusercontent.com/GPUOpen-LibrariesAndSDKs/FidelityFX-SDK/$($manifest.commit)/Kits/FidelityFX/$($header.path)"
            Invoke-WebRequest -Uri $url -OutFile $destination
        }
    }
    # Normalize checkout CRLF to upstream LF; retain all other content verbatim.
    # This permits both raw downloads and Git checkouts with autocrlf enabled.
    foreach ($header in $manifest.headers) {
        $file = Join-Path $SdkRoot $header.path
        $content = [IO.File]::ReadAllText((Resolve-Path $file).Path).Replace("`r`n", "`n")
        $sha = [Security.Cryptography.SHA256]::Create()
        try {
            $actual = [BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($content))).Replace('-', '').ToLowerInvariant()
        } finally { $sha.Dispose() }
        if ($actual -ne $header.sha256) { throw "Pinned header hash mismatch: $($header.path)" }
    }
    if (-not (Get-Command cl -ErrorAction SilentlyContinue)) {
        $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
        $installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        if (-not $installation) { throw 'Install the x64 MSVC tools or run in an x64 VS developer shell.' }
        Import-Module "$installation/Common7/Tools/Microsoft.VisualStudio.DevShell.dll"
        Enter-VsDevShell -VsInstallPath $installation -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'
    }
    foreach ($source in @('native_abi.cpp', 'context_lifecycle/native_abi.cpp', 'dispatch_native_abi.cpp')) {
        $object = $source.Replace('/', '_').Replace('.cpp', '.obj')
        & cl /nologo /std:c++17 /W4 /WX /c /I"$SdkRoot/api/include" /I"$SdkRoot/upscalers/include" "crates/fsr-sdk-sys/tests/$source" /Fo"$Stage/$object"
        if ($LASTEXITCODE -ne 0) { throw "Native ABI check failed: $source (exit=$LASTEXITCODE)" }
    }
} finally { Pop-Location }
