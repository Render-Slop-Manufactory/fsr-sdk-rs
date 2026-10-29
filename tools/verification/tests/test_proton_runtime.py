# SPDX-License-Identifier: MPL-2.0
"""Offline runtime failure boundaries; GPU/network evidence belongs to the profile."""
import contextlib
import io
from pathlib import Path
import sys
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from execution import Blocked, Rejected
import proton
import proton_runtime as runtime
from profiles import compose
from run import main


class RuntimeControls(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        # Match prepare/verify's resolved paths, including Windows aliases and macOS /var.
        self.root = Path(self.temp.name).resolve()
        self.proton = self.root / 'source-proton'
        self.steamrt = self.root / 'data/umu/steamrt3'
        self.launcher = self.root / 'launcher'
        for root, files in ((self.proton, ('proton', 'version', 'toolmanifest.vdf', 'files/bin/wine')),
                            (self.steamrt, ('_v2-entry-point', 'VERSIONS.txt', '.installed.ok')),
                            (self.launcher, ('bin/umu-run', 'lib/launcher.py'))):
            for name in files:
                path = root / name; path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text('fixture, not an executable runtime')
        (self.proton / 'toolmanifest.vdf').write_text('"require_tool_appid" "1628350"')
        (self.steamrt / 'umu').write_text('fixture entry point')
        (self.steamrt.parent / 'umu-shim').write_text('fixture shim')
        self.umu = self.launcher / 'bin/umu-run'; self.umu.chmod(0o755)
        self.output = self.root / 'import'

    def prepare(self):
        result = runtime.prepare(self.proton, self.steamrt, self.umu, self.launcher,
                                 self.output, 'explicit local test fixture')
        return Path(result['manifest']), result['sha256']

    def test_explicit_import_verifies_exact_bytes_and_does_not_overwrite(self):
        path, checksum = self.prepare()
        manifest, roots, observed = runtime.verify(path, checksum)
        self.assertEqual(set(observed), set(runtime.ROLES))
        self.assertEqual(roots['proton'], self.output / 'proton')
        self.assertEqual(manifest['provenance'], 'explicit local test fixture')
        with self.assertRaises(FileExistsError):
            self.prepare()

    def test_tampered_bytes_and_extra_files_are_rejected(self):
        path, checksum = self.prepare()
        target = self.output / 'proton/files/bin/wine'
        original = target.read_bytes()
        for mutate, restore in (
            (lambda: target.write_bytes(b'changed'), lambda: target.write_bytes(original)),
            (lambda: (self.output / 'proton/extra').write_text('extra'), lambda: (self.output / 'proton/extra').unlink()),
        ):
            mutate()
            with self.assertRaisesRegex(Rejected, 'integrity mismatch'):
                runtime.verify(path, checksum)
            restore()
    @unittest.skipIf(sys.platform == 'win32', 'Linux mode/symlink identity; no Windows symlink privilege required')
    def test_tampered_modes_and_links_are_rejected(self):
        path, checksum = self.prepare()
        target = self.output / 'proton/files/bin/wine'
        mode = target.stat().st_mode & 0o777
        target.chmod(0o700)
        with self.assertRaisesRegex(Rejected, 'integrity mismatch'):
            runtime.verify(path, checksum)
        target.chmod(mode)
        link = self.output / 'data/umu/steamrt3/umu'
        link.unlink(); link.symlink_to('VERSIONS.txt')
        with self.assertRaisesRegex(Rejected, 'integrity mismatch'):
            runtime.verify(path, checksum)

    def test_only_declared_generated_runtime_cache_is_excluded(self):
        path, checksum = self.prepare()
        cache = self.output / 'data/umu/steamrt3/var/generated'
        cache.parent.mkdir(); cache.write_text('generated host cache')
        runtime.verify(path, checksum)
        extra = self.output / 'data/umu/steamrt3/extra'; extra.write_text('distribution change')
        with self.assertRaises(Rejected):
            runtime.verify(path, checksum)

    def test_manifest_and_shim_changes_are_rejected(self):
        path, checksum = self.prepare()
        with self.assertRaisesRegex(Rejected, 'checksum'):
            runtime.verify(path, '0' * 64)
        (self.output / 'data/umu/umu-shim').write_text('changed shim')
        with self.assertRaisesRegex(Rejected, 'umu-shim changed'):
            runtime.verify(path, checksum)

    def test_missing_input_cli_rejects_without_repair(self):
        path, checksum = self.prepare()
        absent = self.output / 'proton/files/bin/wine'; absent.unlink()
        with contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(main(['runtime', 'verify', '--manifest', str(path), '--sha256', checksum]), 1)
        self.assertFalse(absent.exists())
        with self.assertRaises(Blocked):
            runtime.verify(self.root / 'missing.json', checksum)

    def test_bounds_overlap_and_missing_provenance_prevent_import(self):
        with patch.object(runtime, 'MAX_BYTES', 1):
            with self.assertRaises(ValueError): self.prepare()
        self.assertFalse(self.output.exists())
        for output, provenance in ((self.proton / 'nested', 'known local source'), (self.output, '')):
            with self.assertRaises(ValueError):
                runtime.prepare(self.proton, self.steamrt, self.umu, self.launcher, output, provenance)

    def test_offline_equipment_rejects_missing_runtime_before_any_tool_process(self):
        options = SimpleNamespace(native=True, online=False, allow_launcher_network=False,
                                  runtime_manifest=str(self.root / 'missing.json'), runtime_sha256='0'*64)
        runner = SimpleNamespace(options=options, process=lambda *a, **k: self.fail('process launched'))
        with patch.object(proton.sys, 'platform', 'linux'):
            with self.assertRaises(Blocked): proton.cases(offline=True)[0].action(runner)
        with patch.object(proton.sys, 'platform', 'linux'):
            options.allow_launcher_network = True
            with self.assertRaisesRegex(Blocked, 'forbids'):
                proton.cases(offline=True)[0].action(runner)

    def test_isolation_cannot_silently_fall_back_and_same_namespace_is_rejected(self):
        with patch.object(runtime.shutil, 'which', return_value=None):
            with self.assertRaises(Blocked): runtime.isolated(['command'], {})
        with patch.object(runtime.os, 'readlink', return_value='same-namespace'):
            with self.assertRaisesRegex(Rejected, 'not isolated'):
                runtime.network_proof('same-namespace')

    def test_offline_profile_retains_both_queries_and_gpu_and_final_integrity(self):
        cases = compose('proton-offline')
        known = set()
        for case in cases:
            self.assertTrue(set(case.requires).issubset(known))
            known.add(case.id)
        self.assertIn('proton.queries', known)
        self.assertEqual(cases[-1].requires, ('equipment.toolchain', 'proton.dispatch'))
        self.assertEqual(cases[-1].id, 'proton.offline.integrity')

    def test_unavailable_isolation_is_reported_before_a_launcher_process(self):
        directory = self.root / 'cases/control'; directory.mkdir(parents=True)
        runner = SimpleNamespace(directory=self.root, env={},
                                 state=dict(offline_runtime=True, umu=self.umu, native_stage=self.root),
                                 current=dict(id='control', commands=[], observations={}),
                                 process=lambda *a, **k: self.fail('launcher process started'))
        with patch.object(runtime, 'isolated', side_effect=Blocked('missing isolation')):
            with self.assertRaisesRegex(Blocked, 'missing isolation'):
                proton.invoke(runner, ['argument'])
        self.assertEqual(runner.current['commands'], [])


if __name__ == '__main__':
    unittest.main()
