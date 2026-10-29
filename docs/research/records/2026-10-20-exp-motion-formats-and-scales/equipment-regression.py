# SPDX-License-Identifier: MPL-2.0
"""Two existing geometry children check the unchanged helper entry point."""
from pathlib import Path
import sys

root = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(root/'tools/verification'))
from execution import Runner
from profiles import compose
from records import fresh
from run import parser

options = parser().parse_args(['run', 'experiment-variable-output', '--native', '--sdk-root',
                              'target/sdk-windows-verification/v2.3.0/Kits/FidelityFX', '--adapter-index', '0'])
directory = fresh(root/'target/verification', 'motion-equipment-regression')
print('Attempt:', directory, flush=True)
runner = Runner(root, directory, options)
runner.env['CARGO_TARGET_DIR'] = str(root/'target/verification-build/current')
sdk = root/options.sdk_root
runner.state['external_inputs'] = [('regression-driver', Path(__file__).resolve())] + [
    ('sdk/'+path.relative_to(sdk).as_posix(), path) for path in sorted(sdk.rglob('*'))
    if path.is_file() and (path.suffix == '.h' or path.name in ('amd_fidelityfx_loader_dx12.dll', 'amd_fidelityfx_upscaler_dx12.dll'))]
selected = [case for case in compose('experiment-variable-output')
            if not case.id.startswith('variable-output.') or case.id in
            ('variable-output.build.fsr_sdk', 'variable-output.control', 'variable-output.fsr4.stable-max')]
code = runner.execute(selected, 'motion-equipment-regression', require_passed=True)
print('Result:', directory/'run.json', flush=True)
sys.exit(code)
