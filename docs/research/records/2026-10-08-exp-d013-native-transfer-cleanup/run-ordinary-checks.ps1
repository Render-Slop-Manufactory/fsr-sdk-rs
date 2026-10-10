# SPDX-License-Identifier: MPL-2.0

# Additional ordinary verification; no AMD/GPU execution or source mutation.
$ErrorActionPreference = 'Stop'
$checkRoot = 'target/d013-native-cleanup/additional-checks'
New-Item -ItemType Directory -Force $checkRoot | Out-Null
$checks = @(
    @{Name='feature-disabled-all-targets'; Args=@('check','--workspace','--all-targets','--no-default-features','--locked','--offline')},
    @{Name='fixture-transfer-events'; Args=@('test','-p','fsr-sdk','--lib','d013_movable','--locked','--offline','--','--nocapture')},
    @{Name='fixture-host-adapter'; Args=@('test','-p','fsr-sdk','--lib','d013_movable','--no-default-features','--locked','--offline','--','--nocapture')},
    @{Name='msrv-check'; Args=@('+1.88.0','check','--workspace','--all-targets','--locked','--offline')},
    @{Name='msrv-tests'; Args=@('+1.88.0','test','--workspace','--locked','--offline')},
    @{Name='msrv-feature-disabled'; Args=@('+1.88.0','check','--workspace','--all-targets','--no-default-features','--locked','--offline')},
    @{Name='rustdoc'; Args=@('doc','--workspace','--no-deps','--locked','--offline')}
)
$previousDocFlags = $env:RUSTDOCFLAGS
try {
    foreach ($check in $checks) {
        $arguments = $check.Args
        if ($check.Name -eq 'rustdoc') { $env:RUSTDOCFLAGS = '-D warnings' }
        & cargo @arguments *>&1 | Tee-Object -FilePath "$checkRoot/$($check.Name).txt"
        $code = $LASTEXITCODE
        Add-Content -LiteralPath "$checkRoot/status.txt" -Value "cargo $($arguments -join ' ') => exit=$code"
        if ($code -ne 0) { throw "Check failed: $($check.Name), exit=$code" }
    }
} finally { $env:RUSTDOCFLAGS = $previousDocFlags }
