# SPDX-License-Identifier: MPL-2.0
# Narrow bridge: acquire toolchain environment or inspect signatures. No tests.
param([ValidateSet('environment', 'signatures')][string]$Mode = 'environment',
      [string]$Directory, [switch]$Strict)
$ErrorActionPreference = 'Stop'
function Test-OnlineChain($Certificate, [string]$Usage) {
    # Delegate to Windows' existing roots, chain builder and revocation caches.
    # No custom roots, downloaded CA bundles or ignored chain errors.
    if (-not $Certificate) { return @{ valid=$false; status=@('MissingCertificate') } }
    $chain = [System.Security.Cryptography.X509Certificates.X509Chain]::new()
    try {
        $chain.ChainPolicy.RevocationMode = 'Online'
        $chain.ChainPolicy.RevocationFlag = 'ExcludeRoot'
        $chain.ChainPolicy.VerificationFlags = 'NoFlag'
        $chain.ChainPolicy.UrlRetrievalTimeout = [TimeSpan]::FromSeconds(15)
        $chain.ChainPolicy.ApplicationPolicy.Add([System.Security.Cryptography.Oid]::new($Usage)) | Out-Null
        $valid = $chain.Build($Certificate)
        return @{ valid=$valid; status=@($chain.ChainStatus | ForEach-Object { [string]$_.Status }) }
    } finally { $chain.Dispose() }
}
if ($Mode -eq 'signatures') {
    $results = @(Get-ChildItem -LiteralPath $Directory -Filter 'amd_*.dll' | ForEach-Object {
        $signature = Get-AuthenticodeSignature -LiteralPath $_.FullName
        $record = [ordered]@{ file=$_.Name; status=[string]$signature.Status;
            signer=$signature.SignerCertificate.Subject;
            signature_type=[string]$signature.SignatureType;
            signer_thumbprint=$signature.SignerCertificate.Thumbprint;
            signer_issuer=$signature.SignerCertificate.Issuer;
            timestamp_subject=$signature.TimeStamperCertificate.Subject;
            timestamp_thumbprint=$signature.TimeStamperCertificate.Thumbprint;
            powershell_version=[string]$PSVersionTable.PSVersion }
        if ($Strict) {
            $tool = Get-Command signtool.exe -ErrorAction SilentlyContinue
            $record.strict_passed = $false
            if (-not $tool) {
                $record.reason = 'SignTool from the Windows SDK must be on PATH'
            } else {
                $record.signtool_version = $tool.FileVersionInfo.FileVersion
                $record.signtool_output = @(& $tool.Source verify /pa /all /v /tw $_.FullName 2>&1 | ForEach-Object { [string]$_ })
                $record.signtool_exit = $LASTEXITCODE
                $record.signer_chain = Test-OnlineChain $signature.SignerCertificate '1.3.6.1.5.5.7.3.3'
                $record.timestamp_chain = Test-OnlineChain $signature.TimeStamperCertificate '1.3.6.1.5.5.7.3.8'
                $publisher = if ($signature.SignerCertificate) {
                    $signature.SignerCertificate.GetNameInfo('SimpleName', $false)
                } else { '' }
                $record.strict_passed = ($signature.Status -eq 'Valid' -and
                    $signature.SignatureType -eq 'Authenticode' -and
                    $publisher -eq 'Advanced Micro Devices' -and
                    $null -ne $signature.TimeStamperCertificate -and
                    $record.signtool_exit -eq 0 -and
                    $record.signer_chain.valid -and $record.timestamp_chain.valid)
            }
        }
        [pscustomobject]$record
    })
    ConvertTo-Json -InputObject $results -Depth 6 -Compress
    exit 0
}
if (-not (Get-Command cl -ErrorAction SilentlyContinue)) {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    $installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (-not $installation) { throw 'x64 MSVC developer tools unavailable' }
    Import-Module "$installation/Common7/Tools/Microsoft.VisualStudio.DevShell.dll"
    Enter-VsDevShell -VsInstallPath $installation -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64' | Out-Null
}
if ($env:VSCMD_ARG_TGT_ARCH -and $env:VSCMD_ARG_TGT_ARCH -ne 'x64') { throw 'x64 MSVC environment required' }
$values = @{}
Get-ChildItem Env: | ForEach-Object { $values[$_.Name] = $_.Value }
ConvertTo-Json -InputObject $values -Compress
