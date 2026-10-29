# SPDX-License-Identifier: MPL-2.0
"""Proton environment, provenance and selection controls; no GPU execution."""
from pathlib import Path
import contextlib
import io
import sys
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import proton
from execution import Blocked, Case, Rejected, Runner
from profiles import compose


class ProtonControls(unittest.TestCase):
    def test_inherited_game_bypasses_and_runners_are_removed(self):
        bad = ('STEAM_COMPAT_DATA_PATH', 'VKD3D_CONFIG', 'DXVK_CONFIG', 'FSR_SDK_TEST_GBV',
               'XWIN_CACHE_DIR', 'WINEDLLOVERRIDES', 'PROTON_USE_WINED3D', 'UMU_NO_RUNTIME',
               'PROTON_NO_D3D12', 'UMU_RUNTIME_UPDATE',
               'CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUNNER', 'WINEPREFIX',
               'FSR4_UPGRADE', 'FSR4_WATERMARK', 'DXIL_SPIRV_CONFIG')
        original = {key: 'inherited' for key in bad}
        original.update(PATH='/controlled/tools', DISPLAY=':0')
        self.assertEqual(proton.clean_environment(original), {'PATH': '/controlled/tools', 'DISPLAY': ':0'})
        self.assertIn('WINEPREFIX', original)

    def test_missing_opt_ins_and_cache_block_before_a_process_is_started(self):
        equipment = proton.cases()[0]
        for native, network in ((False, False), (True, False), (False, True)):
            runner = SimpleNamespace(options=SimpleNamespace(native=native, allow_launcher_network=network))
            with patch.object(proton.sys, 'platform', 'linux'):
                with self.assertRaises(Blocked):
                    equipment.action(runner)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'proton').write_text('placeholder launcher')
            (root / 'version').write_text('concrete version')
            runner = SimpleNamespace(options=SimpleNamespace(native=True, allow_launcher_network=True,
                                      proton=directory, xwin_cache=directory, sdk_root=directory))
            with patch.object(proton.sys, 'platform', 'linux'):
                with self.assertRaisesRegex(Blocked, 'Missing prepared xwin cache'):
                    equipment.action(runner)

    def test_gpu_cases_reuse_frozen_selectors_and_do_not_bypass_queries(self):
        cases = compose('proton')
        preceding = set()
        for case in cases:
            self.assertTrue(set(case.requires).issubset(preceding))
            preceding.add(case.id)
        dispatch = next(case for case in cases if case.id == 'proton.dispatch')
        build = next(case for case in cases if case.id == 'proton.build')
        self.assertIn('proton.sdk', build.requires)
        self.assertIn('proton.queries', dispatch.requires)
        self.assertEqual(dispatch.selector, 'upscaler::dx12::native_dispatch::proton_dispatch_readback')
        self.assertFalse(dispatch.expectation['windows_case_unchanged'])

    def test_both_native_module_paths_are_required_and_other_directories_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            stage = Path(directory)
            for name in proton.DLL_HASHES:
                (stage / name).write_bytes(b'controlled bytes, not a real AMD DLL')
            def log(root):
                return '\n'.join('trace:loaddll:build_module Loaded L"Z:' + str(root / name).replace('/', '\\')
                                 + '" at 0000: native' for name in proton.DLL_HASHES)
            modules = proton.loaded_modules(log(stage), stage)
            self.assertEqual(set(modules), set(proton.DLL_HASHES))
            with self.assertRaisesRegex(Rejected, 'outside'):
                proton.loaded_modules(log(stage.parent), stage)
            with self.assertRaisesRegex(Rejected, 'both'):
                proton.loaded_modules('', stage)

    def test_redirected_child_markers_and_exit_are_both_required(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            launcher = root / 'launcher.py'
            launcher.write_text('import os, pathlib, sys\n'
                                'pathlib.Path(os.environ["PROTON_LOG_DIR"], "steam-test.log").write_text(sys.argv[1])\n'
                                'sys.exit(int(sys.argv[2]))\n')
            # A Python wrapper stands in for umu; it redirects its child output
            # exactly as Proton does, so stdout-only acceptance would fail.
            (root / 'run').mkdir()
            runner = Runner(root, root / 'run', SimpleNamespace(toolchain=None, online=False))
            runner.state.update(umu=sys.executable, native_stage=root)
            cases = []
            for name, marker, exit_code in (('success', 'proof', 0), ('empty', '', 0), ('badexit', 'proof', 1)):
                def action(r, marker=marker, exit_code=exit_code):
                    proton.invoke(r, [launcher, marker, str(exit_code)], markers=('proof',))
                cases.append(Case(name, 'redirected output and exit', action))
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(runner.execute(cases, 'test'), 1)
            self.assertEqual([runner.results[item]['evaluation'] for item in ('success', 'empty', 'badexit')],
                             ['passed', 'failed', 'failed'])
            self.assertEqual(runner.results['badexit']['observations']['proton_commands'][0]['exit'], 1)

    def test_mapped_drives_resolve_to_the_exact_stage_and_unknown_drives_fail(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            stage = root / 'stage'
            stage.mkdir()
            for name in proton.DLL_HASHES:
                (stage / name).write_bytes(b'controlled module bytes')
            def log(drive, folder):
                return '\n'.join(f'trace:loaddll:build_module Loaded L"{drive}:\\{folder}\\{name}" at 0000: native'
                                 for name in proton.DLL_HASHES)
            drives = {'s:': root}
            self.assertEqual(set(proton.loaded_modules(log('S', 'stage'), stage, drives)),
                             set(proton.DLL_HASHES))
            for drive, folder in (('S', 'other'), ('Z', 'stage')):
                with self.assertRaisesRegex(Rejected, 'outside'):
                    proton.loaded_modules(log(drive, folder), stage, drives)

    def test_optional_fsr_upgrade_payload_is_part_of_frozen_external_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            # Match external_inputs' canonical paths, including Windows 8.3 aliases.
            root = Path(directory).resolve()
            payload = root / 'contrib/amdxcffx64.dll'
            payload.parent.mkdir()
            payload.write_bytes(b'controlled upgrade bytes')
            options = SimpleNamespace(proton=directory, xwin_cache=None,
                                      vulkan_validation_manifest=None, cargo_xwin='missing-xwin',
                                      umu='missing-umu', vulkaninfo='missing-vulkaninfo')
            self.assertIn(('proton/contrib/amdxcffx64.dll', payload), proton.external_inputs(options))

    def test_regression_profiles_require_gpu_baseline_and_keep_windows_selectors(self):
        from retention import CALIBRATION_PROFILES
        for kind in ('quality', 'long', 'reset', 'jitter'):
            profile = 'proton-temporal-' + kind
            cases = compose(profile)
            preceding = set()
            for case in cases:
                self.assertTrue(set(case.requires).issubset(preceding))
                preceding.add(case.id)
            self.assertIn(profile, CALIBRATION_PROFILES)
            for case in cases:
                if case.id.endswith(('.r1', '.r2', '.r3')):
                    self.assertIn('proton.dispatch', case.requires)
                    self.assertEqual(case.expectation['diagnostics'], 'proton')
                    self.assertEqual(case.selector, 'upscaler::dx12::native_dispatch::proton_temporal_diagnosis')
            windows = compose('experiment-temporal-' + kind)
            matrix = next(case for case in windows if case.id.endswith('.r1'))
            self.assertEqual(matrix.selector, 'upscaler::dx12::native_dispatch::native_temporal_diagnosis')
            self.assertEqual(matrix.expectation['diagnostics'], 'windows')
        runner = SimpleNamespace(options=SimpleNamespace(native=True, allow_launcher_network=True,
                                                         vulkan_validation_manifest=None))
        with patch.object(proton.sys, 'platform', 'linux'):
            with self.assertRaisesRegex(Blocked, 'requires --vulkan-validation'):
                proton.cases(regression=True)[0].action(runner)

    def test_local_output_paths_use_longest_prefix_mapping_and_missing_mapping_blocks(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name).resolve()
            drives = {'z:': Path('/'), 's:': root}
            self.assertEqual(proton.windows_path(root / 'results/a.txt', drives), r's:\results\a.txt')
            self.assertEqual(proton.windows_path(root, drives), 's:\\')
            with self.assertRaises(Blocked):
                proton.windows_path(root.parent, {'s:': root})

    def test_proton_unavailable_debug_count_is_never_accepted_by_windows_or_as_real_error(self):
        from temporal_calibration import jitter_table
        from temporal_diagnosis import arms, validate
        arm = arms()[0]; j = jitter_table()[0]
        meta = dict(arm=arm['name'], frame=0, scene=arm['scene'], velocity=arm['velocity'],
                    motion=arm['motion'], image_jitter=j, dispatch_jitter=j, reset=True,
                    finish_hr=0, fence=1, debug_errors=2**64-1, device_removed=0,
                    finite_pixels=640*360, dispatch='ok')
        with patch('temporal_diagnosis.rows', return_value=[meta]):
            with self.assertRaisesRegex(ValueError, 'debug_errors'):
                validate('.')
            # A failure later in the oracle proves the unavailable diagnostic
            # did not bypass the input-byte validation.
            with patch('temporal_diagnosis.payload', return_value=b'bad'), patch('temporal_diagnosis.expected_inputs', return_value=b'good'):
                with self.assertRaisesRegex(ValueError, 'Input mismatch'):
                    validate('.', proton=True)
            meta['debug_errors'] = 1
            with self.assertRaisesRegex(ValueError, 'debug_errors'):
                validate('.', proton=True)

    def test_selected_jitter_matrix_keeps_controls_windows_and_frozen_group_criteria(self):
        import json
        from temporal_jitter import protocol
        from temporal_diagnosis import arms, frame_plan
        source = Path(proton.__file__).parent / 'proton-temporal-jitter-protocol.json'
        selected = json.loads(source.read_text()); original = protocol(True)
        self.assertEqual(len(arms(selected)), 12)
        self.assertEqual(len(frame_plan(selected)[1]), 9)
        for key in ('windows', 'frames', 'repeats', 'order_seeds', 'selection_rule'):
            self.assertEqual(selected[key], original[key])
        for scene, baseline in zip(selected['scenes'], original['scenes']):
            self.assertEqual(scene['controls'], baseline['controls'])
            self.assertEqual(scene['velocity'], baseline['velocity'])
            self.assertEqual(scene['jitter_phases'], [0])

    def test_wine_mapped_linux_paths_are_sanitized_in_records_and_environment(self):
        from records import Sanitizer
        with tempfile.TemporaryDirectory() as name:
            root = Path(name).resolve(); run = root / 'run'
            sanitizer = Sanitizer(root, run)
            windows = 'Z:' + str(run / 'native/provider.dll').replace('/', '\\')
            for spelling in (windows, windows.replace('\\', '\\\\')):
                result = sanitizer.value({'loaded_path': spelling, 'provider_id': '0xf5a5ca1e00c01005'})
                self.assertNotIn(str(root).replace('/', '\\'), result['loaded_path'])
                self.assertIn('Z:<run>', result['loaded_path'])
                self.assertEqual(result['provider_id'], '0xf5a5ca1e00c01005')


if __name__ == '__main__':
    unittest.main()
