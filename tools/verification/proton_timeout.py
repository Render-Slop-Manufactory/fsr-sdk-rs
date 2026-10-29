# SPDX-License-Identifier: MPL-2.0
"""Readiness-synchronized Wine timeout and independent prefix-isolation oracles."""
import os
from pathlib import Path
import select
import signal
import subprocess
import time
from unittest.mock import patch
import uuid

from execution import Blocked, Case, Rejected
from records import fresh, sha


def processes(prefix, proc_root='/proc'):
    """Inspect only matching prefix ownership; never emit complete environments."""
    result = {}
    for directory in Path(proc_root).iterdir():
        if not directory.name.isdigit():
            continue
        try:
            fields = (directory / 'stat').read_text().rsplit(')', 1)[1].split()
            if fields[0] == 'Z' or directory.stat().st_uid != os.getuid():
                continue
            variables = (directory / 'environ').read_bytes().split(b'\0')
            values = [v.split(b'=', 1)[1] for v in variables if v.startswith(b'WINEPREFIX=')]
            if not values or Path(os.fsdecode(values[0])).resolve() != Path(prefix).resolve():
                continue
            pid = int(directory.name)
            result[pid] = dict(pid=pid, start_ticks=int(fields[19]), ppid=int(fields[1]),
                               process_group=int(fields[2]), session=int(fields[3]),
                               executable=(directory / 'comm').read_text().strip())
        except (OSError, ValueError):
            continue  # exited/reaped processes and inaccessible unrelated users
    return result


def growth(path, previous=0, timeout=15):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        if path.is_file() and path.stat().st_size > previous:
            return path.stat().st_size
        time.sleep(.025)
    raise Rejected('Fixture heartbeat did not advance: ' + path.name)


def after_cleanup(runner, action):
    if runner.state.get('timeout_cleanup_complete') is not True:
        raise Blocked('Post-timeout controls require completed independent fixture cleanup')
    return action(runner)


class TimeoutFixture:
    def __init__(self, runner, prefix, sentinel_prefix, executable):
        self.runner, self.prefix, self.sentinel_prefix = runner, prefix, sentinel_prefix
        self.directory = runner.directory / 'cases' / runner.current['id']
        self.target_base = self.directory / 'target'
        self.sentinel_base = self.directory / 'sentinel'
        self.popen = subprocess.Popen
        self.target = self.sentinel = None
        self.handles = {}
        self.seen = {}
        self.executable = executable

    def command(self, prefix, mode, base):
        from proton import windows_path, wine_drives
        from proton_runtime import isolated
        return isolated([self.runner.state['umu'], self.executable, mode,
                         windows_path(base, wine_drives(prefix))], self.runner.env)

    def environment(self, prefix):
        return dict(self.runner.env, WINEPREFIX=str(prefix),
                    PROTON_LOG_DIR=str(self.directory),
                    FSR_SDK_TEST_TIMEOUT_OWNER=uuid.uuid4().hex)

    def remember(self):
        for pid, item in processes(self.prefix).items():
            if pid not in self.handles:
                try:
                    self.handles[pid] = os.pidfd_open(pid)
                    self.seen[pid] = item
                except ProcessLookupError:
                    pass

    def sentinel_start(self):
        command = self.command(self.sentinel_prefix, 'hold', self.sentinel_base)
        self.stdout = (self.directory / 'sentinel.stdout.txt').open('wb')
        self.stderr = (self.directory / 'sentinel.stderr.txt').open('wb')
        self.sentinel = self.popen(command, cwd=self.runner.state['native_stage'],
                                   env=self.environment(self.sentinel_prefix),
                                   stdout=self.stdout, stderr=self.stderr, start_new_session=True)
        growth(self.sentinel_base.with_suffix('.hold'))
        self.sentinel_before = processes(self.sentinel_prefix)
        if not self.sentinel_before:
            raise Blocked('Cannot observe the independent Wine prefix')

    def spawn(self, arguments, **options):
        process = self.popen(arguments, **options)
        self.target = process
        # The real Runner.wait(timeout) starts after both Windows actors are live.
        for mode in ('parent', 'child'):
            path = self.target_base.with_suffix('.' + mode)
            first = growth(path)
            growth(path, first)
        self.remember()
        if not self.seen or not any('wine' in p['executable'].lower() for p in self.seen.values()):
            raise Blocked('Cannot observe Wine descendants before timeout')
        self.runner.note(before_timeout=list(self.seen.values()), watchdog_root=process.pid,
                         detached_groups=any(p['process_group'] != process.pid for p in self.seen.values()),
                         ready_parent_and_detached_child=True)
        return process

    def assert_stopped(self, timeout=5):
        end = time.monotonic() + timeout
        while time.monotonic() < end:
            self.remember()
            live_handles = [pid for pid, fd in self.handles.items() if not select.select([fd], [], [], 0)[0]]
            remaining = processes(self.prefix)
            if not live_handles and not remaining:
                sizes = {mode: self.target_base.with_suffix('.' + mode).stat().st_size for mode in ('parent', 'child')}
                time.sleep(.3)
                if any(self.target_base.with_suffix('.' + mode).stat().st_size != size for mode, size in sizes.items()):
                    raise Rejected('Heartbeat continued after apparent descendant exit')
                self.runner.note(descendants_stopped=True, surviving_processes=[], heartbeats_stopped=True)
                return
            time.sleep(.025)
        self.runner.note(descendants_stopped=False, surviving_processes=list(processes(self.prefix).values()))
        raise Rejected('Watchdog left live processes in its isolated Wine prefix')

    def assert_sentinel(self):
        path = self.sentinel_base.with_suffix('.hold')
        growth(path, path.stat().st_size)
        if self.sentinel.poll() is not None:
            raise Rejected('Timeout affected the unrelated launcher')
        after = processes(self.sentinel_prefix)
        actors = {pid: item for pid, item in self.sentinel_before.items()
                  if 'timeout' in item['executable'].lower()}
        if not actors:
            raise Blocked('Cannot identify the unrelated Windows actor')
        if not all(pid in after and after[pid]['start_ticks'] == item['start_ticks']
                   for pid, item in actors.items()):
            raise Rejected('Timeout affected an unrelated-prefix process')
        self.runner.note(unrelated_heartbeat_advanced=True, unrelated_processes_preserved=True)

    def close(self):
        # First request a cooperative stop, independent of the mechanism under test.
        for base in (self.target_base, self.sentinel_base):
            base.with_suffix('.stop').touch()
        end = time.monotonic() + 5
        while time.monotonic() < end and (processes(self.prefix) or processes(self.sentinel_prefix)):
            time.sleep(.05)
        emergency = []
        for prefix in (self.prefix, self.sentinel_prefix):
            for pid, item in processes(prefix).items():
                # Signal through a pidfd, never a recycled PID or a global wineserver kill.
                try:
                    fd = os.pidfd_open(pid)
                    current = processes(prefix).get(pid)
                    if current and current['start_ticks'] == item['start_ticks']:
                        signal.pidfd_send_signal(fd, signal.SIGKILL)
                        emergency.append(item)
                    os.close(fd)
                except ProcessLookupError:
                    pass
        for process in (self.target, self.sentinel):
            if process is not None:
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait(timeout=5)
        for fd in self.handles.values():
            os.close(fd)
        if hasattr(self, 'stdout'):
            self.stdout.close(); self.stderr.close()
        self.runner.note(emergency_cleanup=emergency)
        if processes(self.prefix) or processes(self.sentinel_prefix):
            raise Rejected('Independent fixture cleanup left live owned-prefix processes')


def cases():
    from proton import cases as proton_cases, invoke
    from proton_runtime import integrity_case
    base = proton_cases(offline=True)

    def build(r):
        if not hasattr(os, 'pidfd_open') or not hasattr(signal, 'pidfd_send_signal'):
            raise Blocked('Timeout oracle requires Linux pidfds')
        stage = r.state['native_stage']; cache = r.state['xwin_cache'] / 'xwin'
        command = [r.state['clang'], '--target=x86_64-pc-windows-msvc', '/nologo', '/std:c++17', '/EHsc', '/MT',
                   r.repository / 'tools/verification/fixtures/proton_timeout.cpp',
                   f'/Fe{stage}/timeout-fixture.exe', f'/Fo{stage}/timeout-fixture.obj', '-fuse-ld=lld']
        for name in ('crt/include', 'sdk/include/ucrt', 'sdk/include/um', 'sdk/include/shared'):
            command += ['/imsvc', cache / name]
        command += ['/link', '/SUBSYSTEM:CONSOLE'] + [f'/LIBPATH:{cache / name}' for name in
                  ('crt/lib/x86_64', 'sdk/lib/um/x86_64', 'sdk/lib/ucrt/x86_64')]
        r.process(command, cwd=stage, timeout=180)
        r.state['timeout_executable'] = stage / 'timeout-fixture.exe'
        r.note(fixture_sha256=sha(r.state['timeout_executable']), fixture_lifetime_seconds=120)

    def timeout(r):
        r.state['timeout_cleanup_complete'] = False
        prefix = r.state['prefix']
        sentinel = fresh(r.repository / 'target/proton-prefixes', 'timeout-sentinel')
        executable = r.state['timeout_executable']
        invoke(r, [executable, 'once', 'unused'], environment=dict(WINEPREFIX=sentinel, PROTON_VERB='waitforexitandrun'))
        fixture = TimeoutFixture(r, prefix, sentinel, executable)
        r.note(target_prefix=prefix, unrelated_prefix=sentinel)
        try:
            fixture.sentinel_start()
            with patch('execution.subprocess.Popen', side_effect=fixture.spawn):
                _, _, command = r.process(fixture.command(prefix, 'parent', fixture.target_base),
                                          env=fixture.environment(prefix), cwd=r.state['native_stage'],
                                          timeout=1, observe=True, name='forced-timeout')
            if command['execution'] != 'timed_out':
                raise Rejected('Watchdog expiry did not occur after readiness')
            stopped_error = None
            try:
                fixture.assert_stopped()
            except Rejected as error:
                stopped_error = error
            fixture.assert_sentinel()
            if stopped_error:
                raise stopped_error
            r.current['evaluation'] = 'passed'  # Timeout is required only in this diagnostic case.
        finally:
            fixture.close()
            r.state['timeout_cleanup_complete'] = True

    result = base + [Case('proton.timeout.fixture', 'Cross-build bounded Windows parent/detached-child fixture', build,
                          ('proton.dispatch',)),
                     Case('proton.timeout.expiry', 'Forced watchdog expiry stops all owned descendants and preserves unrelated Wine prefix',
                          timeout, ('proton.timeout.fixture',))]
    for kind in ('queries', 'dispatch'):
        original = next(c for c in base if c.id == 'proton.' + kind)
        result.append(Case('proton.timeout.post-' + kind, 'Normal ' + kind + ' succeeds after forced timeout and fixture cleanup',
                           lambda r, action=original.action: after_cleanup(r, action),
                           ('proton.timeout.fixture',), original.selector, original.scope, original.expectation))
    integrity = integrity_case()
    integrity.requires = ('proton.timeout.post-dispatch',)
    result.append(integrity)
    return result
