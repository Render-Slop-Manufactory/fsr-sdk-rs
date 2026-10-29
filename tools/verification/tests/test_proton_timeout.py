# SPDX-License-Identifier: MPL-2.0
"""Timeout ownership, live-descendant rejection and mandatory containment evidence."""
import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from execution import Blocked, Case, Rejected, Runner
from profiles import compose
import proton
import proton_runtime
from proton_timeout import after_cleanup, processes, TimeoutFixture


class TimeoutControls(unittest.TestCase):
    def test_same_pid_namespace_is_rejected(self):
        with patch.object(proton_runtime.os, 'readlink', return_value='same'):
            with self.assertRaisesRegex(Rejected, 'PID namespace'):
                proton_runtime.process_proof('same')

    def test_network_proof_alone_cannot_pass_a_proton_launch(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name); output = root / 'run'; output.mkdir()
            runner = Runner(root, output, SimpleNamespace(toolchain=None, online=False))
            runner.state.update(native_stage=root, umu=sys.executable, offline_runtime=True)
            code = 'print(\'OFFLINE NETWORK: {"distinct_namespace":true,"interfaces":["lo"],"unreachable_errno":[101,101]}\')'
            action = lambda r: proton.invoke(r, ['-c', code])
            with patch.object(proton_runtime, 'isolated', side_effect=lambda args, env: args):
                with contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(runner.execute([Case('proof', 'missing PID evidence', action)], 'fixture'), 1)
            self.assertEqual(runner.results['proof']['evaluation'], 'failed')

    def test_profile_preserves_before_and_after_gpu_checks(self):
        cases = compose('proton-timeout'); known = set()
        for case in cases:
            self.assertTrue(set(case.requires).issubset(known))
            known.add(case.id)
        self.assertTrue({'proton.dispatch', 'proton.timeout.expiry', 'proton.timeout.post-queries',
                         'proton.timeout.post-dispatch', 'proton.offline.integrity'} <= known)

    def test_post_controls_cannot_start_after_incomplete_fixture_cleanup(self):
        runner = SimpleNamespace(state={})
        action = lambda r: self.fail('native control started')
        with self.assertRaises(Blocked): after_cleanup(runner, action)
        runner.state['timeout_cleanup_complete'] = True
        self.assertEqual(after_cleanup(runner, lambda r: 'completed'), 'completed')

    @unittest.skipIf(sys.platform != 'linux', 'Linux prefix ownership inventory')
    def test_owner_inventory_filters_other_prefixes_zombies_and_private_environment(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name); proc = root / 'proc'; proc.mkdir()
            for pid, prefix, state in ((321, root / 'owned', 'S'), (322, root / 'other', 'S'), (323, root / 'owned', 'Z')):
                directory = proc / str(pid); directory.mkdir()
                fields = ['0'] * 20; fields[0] = state; fields[1] = '1'; fields[2] = fields[3] = '7'; fields[19] = '123'
                (directory / 'stat').write_text(str(pid) + ' (actor with spaces) ' + ' '.join(fields))
                (directory / 'comm').write_text('fixture\n')
                (directory / 'environ').write_bytes(b'PRIVATE=never-emit\0WINEPREFIX=' + os.fsencode(prefix) + b'\0')
            observed = processes(root / 'owned', proc)
            self.assertEqual(set(observed), {321})
            self.assertEqual(observed[321]['start_ticks'], 123)
            self.assertNotIn('never-emit', json.dumps(observed))

    @unittest.skipIf(sys.platform != 'linux', 'Real Linux pidfd descendant oracle')
    def test_live_descendant_is_rejected_independently_of_launcher_exit(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name); (root / 'cases/control').mkdir(parents=True)
            prefix = root / 'owned'; prefix.mkdir()
            notes = {}
            runner = SimpleNamespace(directory=root, current=dict(id='control'), note=lambda **values: notes.update(values))
            fixture = TimeoutFixture(runner, prefix, root / 'unrelated', root / 'fixture.exe')
            process = subprocess.Popen([sys.executable, '-c', 'import time;time.sleep(30)'],
                                       env=dict(os.environ, WINEPREFIX=str(prefix)),
                                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            try:
                fixture.handles[process.pid] = os.pidfd_open(process.pid)
                with self.assertRaisesRegex(Rejected, 'live processes'):
                    fixture.assert_stopped(timeout=.1)
                self.assertFalse(notes['descendants_stopped'])
            finally:
                process.terminate(); process.wait(timeout=5)
                for fd in fixture.handles.values(): os.close(fd)


if __name__ == '__main__':
    unittest.main()
