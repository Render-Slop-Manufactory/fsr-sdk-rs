# SPDX-License-Identifier: MPL-2.0
"""Exact-package native/ABI/example bindings; native processes are mocked."""
import io
import json
from pathlib import Path
import sys
import tarfile
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from execution import Rejected
from native import build_case
from profiles import ARTIFACT_NATIVE_PROFILES, compose
from public_callbacks import cases as callback_cases
from records import sha
from release import package_cases, verify_archive_integrity
from run import parser
import proton


class PackageCoverage(unittest.TestCase):
    @patch('examples.os', SimpleNamespace(name='nt'))
    def test_complete_profiles_and_final_integrity_have_no_source_fallback(self):
        for name in ('release', 'release-artifacts', 'release-coverage'):
            cases = compose(name)
            ids = {case.id for case in cases}
            for prefix in ('release.consumer', 'release.consumer-msrv'):
                self.assertIn(prefix+'.examples.dx12-utils', ids)
                self.assertIn(prefix+'.examples.dx12-movable-utils', ids)
                for family in ('local', 'movable'):
                    for workflow in ('simple', 'provider', 'abort', 'utility'):
                        self.assertIn(prefix+'.walkthrough.'+family+'.'+workflow, ids)
            if name != 'release-artifacts':
                self.assertIn('artifact-callbacks.native', ids)
                self.assertIn('artifact-callbacks.abi-msrv', ids)
                self.assertEqual(cases[-1].id, 'release.integrity')
                self.assertEqual(set(cases[-1].requires), ids-{'release.integrity'})
        artifact_ids = {case.id for case in compose('release-artifacts')}
        for toolchain in ('current', '1.88.0'):
            for feature in ('utils', 'movable-utils', 'no-default-utils'):
                self.assertIn(f'release.artifact.{toolchain}.{feature}.check', artifact_ids)

    def test_all_supported_native_graphs_require_the_archive_gate(self):
        for name in ARTIFACT_NATIVE_PROFILES:
            with self.subTest(name=name):
                cases = compose(name, artifacts=True)
                seen = set()
                for case in cases:
                    self.assertNotIn(case.id, seen)
                    self.assertTrue(set(case.requires) <= seen)
                    if 'artifact.bind' in seen and case.id != 'release.integrity':
                        self.assertIn('artifact.bind', case.requires)
                    seen.add(case.id)
                self.assertEqual(cases[-1].id, 'release.integrity')
                self.assertIn('release.registry', seen)
        with self.assertRaisesRegex(ValueError, 'unsupported'):
            compose('experiment-performance-pilot', artifacts=True)
        self.assertTrue(parser().parse_args(['plan', 'proton-offline', '--artifacts']).artifacts)

    def test_partial_archive_selection_is_rejected_before_any_build(self):
        for wrapper, raw in (('wrapper.crate', None), (None, 'raw.crate'), ('', 'raw.crate')):
            runner = SimpleNamespace(directory=Path('unused'), state={},
                options=SimpleNamespace(wrapper_archive=wrapper, sys_archive=raw))
            with self.assertRaisesRegex(Rejected, 'both'):
                package_cases()[0].action(runner)

    def test_native_builder_uses_selected_manifest_environment_and_binary(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            stage = root/'native'; stage.mkdir()
            binary = root/'original.exe'; binary.write_bytes(b'archive-test')
            calls, notes = [], {}
            state = {'artifact_native': True, 'packages': root/'packages',
                     'versions': {'fsr-sdk': '0.3.0'}, 'registry_env': {'CARGO_HOME': 'isolated'},
                     'archive_hashes': {'fsr-sdk': 'selected'}, 'native_stage': stage}

            def process(args, **kwargs):
                calls.append((args, kwargs))
                return json.dumps({'reason': 'compiler-artifact', 'target': {'name': 'fsr_sdk'},
                                   'executable': str(binary)}), '', {}

            runner = SimpleNamespace(state=state, repository=root/'checkout', cargo=lambda args: args,
                                     process=process, note=lambda **kw: notes.update(kw))
            build_case('artifact-test').action(runner)
            self.assertEqual(calls[0][0][2], root/'packages/fsr-sdk-0.3.0/Cargo.toml')
            self.assertEqual(calls[0][1]['env'], state['registry_env'])
            self.assertEqual((stage/'artifact-test-fsr_sdk.exe').read_bytes(), b'archive-test')
            self.assertTrue(notes['registry_artifact'])
            self.assertEqual(notes['archive_hashes'], state['archive_hashes'])

    def test_callback_aggregate_import_comes_from_selected_raw_archive(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            raw = root/'packages/fsr-sdk-sys-0.1.2/src/api.rs'
            raw.parent.mkdir(parents=True); raw.write_text('// selected archive\n', encoding='utf-8', newline='\n')
            calls, notes = [], {}
            runner = SimpleNamespace(directory=root, repository=root/'checkout',
                state={'artifact_native': True, 'packages': root/'packages',
                       'versions': {'fsr-sdk-sys': '0.1.2'}, 'headers': root/'headers',
                       'archive_hashes': {'fsr-sdk-sys': 'selected'}},
                process=lambda args, **kw: calls.append(args), note=lambda **kw: notes.update(kw))
            case = next(case for case in callback_cases() if case.id.endswith('.abi-build'))
            case.action(runner)
            source = (root/'public-callbacks-abi/exchange.rs').read_text(encoding='utf-8')
            self.assertIn(json.dumps(raw.as_posix(), ensure_ascii=False), source)
            self.assertNotIn('../../crates/fsr-sdk-sys/src/api.rs', source)
            self.assertEqual(notes['raw_api_sha256'], sha(raw))
            self.assertTrue(notes['registry_artifact'])
            self.assertEqual(calls[0][0], 'rustc')
            self.assertIn('+1.88.0', calls[1])

    def test_changed_archives_bytes_and_added_extracted_files_are_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive = root/'wrapper.crate'
            with tarfile.open(archive, 'w') as output:
                member = tarfile.TarInfo('fsr-sdk-0.3.0/src/lib.rs')
                member.size = 3; output.addfile(member, io.BytesIO(b'abc'))
            extracted = root/'packages/fsr-sdk-0.3.0/src/lib.rs'
            extracted.parent.mkdir(parents=True); extracted.write_bytes(b'abc')
            runner = SimpleNamespace(state={'archives': {'fsr-sdk': archive},
                'archive_hashes': {'fsr-sdk': sha(archive)}, 'versions': {'fsr-sdk': '0.3.0'},
                'packages': root/'packages'}, note=lambda **kw: None)
            verify_archive_integrity(runner)
            extracted.write_bytes(b'bad')
            with self.assertRaisesRegex(Rejected, 'Extracted'):
                verify_archive_integrity(runner)
            extracted.write_bytes(b'abc')
            (extracted.parent/'injected.rs').write_bytes(b'extra')
            with self.assertRaisesRegex(Rejected, 'inventory'):
                verify_archive_integrity(runner)
            archive.write_bytes(b'changed')
            with self.assertRaisesRegex(Rejected, 'Selected archive'):
                verify_archive_integrity(runner)

    def test_proton_cross_build_uses_package_manifest_and_isolated_registry(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            stage = root/'native'; stage.mkdir()
            binary = root/'built.exe'; binary.write_bytes(b'cross-built archive')
            calls, notes = [], {}

            def process(args, **kwargs):
                calls.append((args, kwargs))
                return json.dumps({'reason': 'compiler-artifact', 'target': {'name': 'fsr_sdk'},
                                   'executable': str(binary)}), '', {}

            runner = SimpleNamespace(repository=root/'checkout', env={}, options=SimpleNamespace(toolchain=None),
                state={'artifact_native': True, 'packages': root/'packages', 'cargo_xwin': 'cargo-xwin',
                       'registry_env': {'CARGO_HOME': 'isolated'}, 'archive_hashes': {'fsr-sdk': 'selected'},
                       'native_stage': stage, 'xwin_cache': root/'xwin'},
                cargo=lambda args: ['cargo', *args], process=process, note=lambda **kw: notes.update(kw))
            next(case for case in proton.cases() if case.id == 'proton.build').action(runner)
            args, options = calls[0]
            self.assertEqual(args[args.index('--manifest-path')+1], root/'packages/Cargo.toml')
            self.assertEqual(options['cwd'], root/'packages')
            self.assertEqual(options['env']['CARGO_HOME'], 'isolated')
            self.assertEqual(options['env']['XWIN_HTTP_RETRIES'], '0')
            self.assertEqual((stage/'proton-fsr_sdk.exe').read_bytes(), b'cross-built archive')
            self.assertTrue(notes['registry_artifact'])

    def test_proton_helper_inputs_are_from_the_same_selected_package_pair(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            stage = root/'native'; stage.mkdir()
            calls = []

            def process(args, **kwargs):
                calls.append(args); (stage/'gpu_helper.dll').write_bytes(b'helper')

            runner = SimpleNamespace(repository=root/'checkout', state={'native_stage': stage,
                'headers': root/'sdk', 'xwin_cache': root/'xwin', 'clang': 'clang-cl',
                'artifact_native': True, 'packages': root/'packages',
                'versions': {'fsr-sdk': '0.3.0', 'fsr-sdk-sys': '0.1.2'}},
                process=process, note=lambda **kw: None)
            next(case for case in proton.cases() if case.id == 'proton.helper').action(runner)
            self.assertIn(root/'packages/fsr-sdk-0.3.0/tests/native/gpu.cpp', calls[0])
            self.assertIn(root/'packages/fsr-sdk-sys-0.1.2/tests/context_lifecycle/device.cpp', calls[0])


if __name__ == '__main__':
    unittest.main()
