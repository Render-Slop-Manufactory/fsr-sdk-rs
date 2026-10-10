# SPDX-License-Identifier: MPL-2.0
# Compatibility entry point; shared implementation lives under tests/native.
param([switch]$Native, [switch]$GpuValidation, [string]$Stage = 'target/m6a')
& "$PSScriptRoot/native/run.ps1" -Mode dispatch -Native:$Native -GpuValidation:$GpuValidation -Stage $Stage
