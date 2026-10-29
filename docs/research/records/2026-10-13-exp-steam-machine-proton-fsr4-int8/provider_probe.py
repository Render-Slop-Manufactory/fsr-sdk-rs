# SPDX-License-Identifier: MPL-2.0
"""Bounded provider experiment; reuse every normal Proton acceptance case."""
import argparse
from pathlib import Path
import sys
from unittest.mock import patch

parser = argparse.ArgumentParser()
parser.add_argument('--repository', required=True)
parser.add_argument('--upgrade-control', choices=('automatic', 'disabled'), required=True)
options, arguments = parser.parse_known_args()
repository = Path(options.repository).resolve()
sys.path.insert(0, str(repository / 'tools/verification'))
import proton
import run
import vulkan_validation

original_inputs = proton.external_inputs
original_invoke = proton.invoke
original_validation = vulkan_validation.environment

def inputs(configuration):
    return original_inputs(configuration) + [('experiment/provider_probe.py', Path(__file__).resolve())]

def invoke(runner, arguments, *, environment=None, markers=(), validation=False):
    controls = dict(environment or {})
    if validation:
        controls.update(FSR4_UPGRADE='0' if options.upgrade_control == 'disabled' else None,
                        WINEDEBUG='+loaddll,+amdxc')
        runner.note(provider_experiment={'upgrade_control': options.upgrade_control,
                                        'same_workload_and_acceptance': True})
    dispatch = validation and runner.current['id'] == 'proton.dispatch'
    if dispatch:
        dumps = runner.directory / 'shader-dumps'
        dumps.mkdir()
        controls.update(VKD3D_SHADER_DUMP_PATH=str(dumps), VKD3D_SHADER_CACHE_PATH='0')
        runner.note(shader_diagnostics={'directory': dumps, 'cache': 'disabled',
                                        'compiled_shaders_alone_do_not_prove_dispatch': True})
    def diagnostics(manifest, directory):
        values, logfile = original_validation(manifest, directory)
        if dispatch:
            values['VKD3D_DEBUG'] = 'trace'
        return values, logfile
    with patch.object(vulkan_validation, 'environment', diagnostics):
        return original_invoke(runner, arguments, environment=controls,
                               markers=markers, validation=validation)

with patch.object(proton, 'external_inputs', inputs), patch.object(proton, 'invoke', invoke):
    raise SystemExit(run.main(arguments))
