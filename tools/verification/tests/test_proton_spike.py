# SPDX-License-Identifier: MPL-2.0
"""Host controls for an experimental bundle; these never execute Proton."""
from pathlib import Path
import sys
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import proton_spike
from records import inventory, save, sha


class SpikeControls(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)

    def tearDown(self):
        self.temporary.cleanup()

    def bundle(self):
        (self.root / 'app').mkdir()
        (self.root / 'app/test.exe').write_bytes(b'controlled placeholder, not executable')
        save(self.root / 'bundle.json', {'schema_version': 1, 'files': inventory(self.root)})

    def test_payload_corruption_missing_and_additional_files_are_rejected(self):
        self.bundle()
        proton_spike.verify_bundle(self.root)
        payload = self.root / 'app/test.exe'
        original = payload.read_bytes()
        payload.write_bytes(b'corruption')
        with self.assertRaisesRegex(ValueError, 'modified, missing or additional'):
            proton_spike.verify_bundle(self.root)
        payload.unlink()
        with self.assertRaises(ValueError):
            proton_spike.verify_bundle(self.root)
        payload.write_bytes(original)
        extra = self.root / 'app/unrequested.dll'
        extra.write_bytes(b'extra dependency')
        with self.assertRaises(ValueError):
            proton_spike.verify_bundle(self.root)
        extra.unlink()
        proton_spike.verify_bundle(self.root)

    def test_current_crate_content_and_inventory_must_match_verified_build(self):
        crate = self.root / 'crates/example'
        crate.mkdir(parents=True)
        source = crate / 'lib.rs'
        source.write_text('original')
        manifest = self.root / 'Cargo.toml'
        manifest.write_text('original manifest')
        inputs = {'files': [{'path': path.relative_to(self.root).as_posix(), 'sha256': sha(path)}
                            for path in (source, manifest)]}
        proton_spike.check_sources(self.root, inputs)
        source.write_text('modified')
        with self.assertRaisesRegex(ValueError, 'Verified build input changed'):
            proton_spike.check_sources(self.root, inputs)
        source.write_text('original')
        extra = crate / 'new.rs'
        extra.write_text('new source')
        with self.assertRaisesRegex(ValueError, 'inventory changed'):
            proton_spike.check_sources(self.root, inputs)
        extra.unlink()
        manifest.write_text('changed manifest')
        with self.assertRaisesRegex(ValueError, 'Verified build input changed'):
            proton_spike.check_sources(self.root, inputs)

    def test_windows_execution_and_missing_explicit_opt_ins_never_start_processes(self):
        with patch.object(proton_spike.sys, 'platform', 'win32'):
            with self.assertRaisesRegex(ValueError, 'requires Linux'):
                proton_spike.run_bundle(SimpleNamespace())
        with patch.object(proton_spike.sys, 'platform', 'linux'):
            for native, network in ((False, False), (True, False), (False, True)):
                with self.assertRaisesRegex(ValueError, 'allow-launcher-network'):
                    proton_spike.run_bundle(SimpleNamespace(native=native, allow_launcher_network=network))

    def test_bundle_schema_is_required(self):
        save(self.root / 'bundle.json', {'schema_version': 99, 'files': []})
        with self.assertRaisesRegex(ValueError, 'schema'):
            proton_spike.verify_bundle(self.root)

    def test_run_outputs_cannot_change_the_inspected_bundle(self):
        self.bundle()
        with patch.object(proton_spike.sys, 'platform', 'linux'):
            for output in (self.root, self.root / 'target', self.root.parent):
                options = SimpleNamespace(native=True, allow_launcher_network=True,
                                          bundle=self.root, output_root=output,
                                          prefix_root=self.root.parent / 'prefixes')
                with self.assertRaisesRegex(ValueError, 'outside the immutable bundle'):
                    proton_spike.run_bundle(options)


if __name__ == '__main__':
    unittest.main()
