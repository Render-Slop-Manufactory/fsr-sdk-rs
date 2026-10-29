# SPDX-License-Identifier: MPL-2.0
# Compatibility entry point; shared implementation lives under tests/native.
param([switch]$Native, [switch]$GpuValidation, [string]$Stage = 'target/m5-lifecycle')
& "$PSScriptRoot/native/run.ps1" -Mode lifecycle -Native:$Native -GpuValidation:$GpuValidation -Stage $Stage
