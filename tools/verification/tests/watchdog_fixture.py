# SPDX-License-Identifier: MPL-2.0
"""Real nested processes with startup synchronization and independent cleanup."""
from contextlib import contextmanager
import os
from pathlib import Path
import select
import signal
import socket
import struct
import subprocess
import sys
import time
from unittest.mock import patch


CHILD = """
import os, pathlib, socket, struct, sys, time
with socket.create_connection((sys.argv[1], int(sys.argv[2])), timeout=15) as channel:
    channel.sendall(struct.pack('!Q', os.getpid()))
    time.sleep(float(sys.argv[4]))
    with pathlib.Path(sys.argv[3]).open('ab', buffering=0) as heartbeat:
        heartbeat.write(b'.')
        channel.sendall(b'R')
        channel.settimeout(.05)
        deadline = time.monotonic() + 30
        while time.monotonic() < deadline:
            heartbeat.write(b'.')
            try:
                command = channel.recv(1)
            except socket.timeout:
                continue
            if not command or command == b'Q':
                break
"""
PARENT = """
import subprocess, sys
child = subprocess.Popen([sys.executable, '-c', sys.argv[1], *sys.argv[2:]])
try:
    child.wait()
finally:
    if child.poll() is None:
        child.kill()
    child.wait(timeout=15)
"""


class TreeFixture:
    def __init__(self, heartbeat, startup_delay, on_ready):
        self.heartbeat = Path(heartbeat)
        self.on_ready = on_ready
        self.process = None
        self.channel = None
        self.child_pid = None
        self.child_handle = None
        self.stopped = False
        self.popen = subprocess.Popen
        self.listener = socket.socket()
        self.listener.bind(('127.0.0.1', 0))
        self.listener.listen(1)
        self.listener.settimeout(15)
        host, port = self.listener.getsockname()
        self.command = [sys.executable, '-c', PARENT, CHILD, host, str(port),
                        str(self.heartbeat), str(startup_delay)]

    def spawn(self, args, **kwargs):
        process = self.popen(args, **kwargs)
        if len(args) < 3 or args[1:3] != ['-c', PARENT]:
            return process  # taskkill and other commands still use the real Popen.
        self.process = process  # Retain ownership even if startup fails.
        self.channel, _ = self.listener.accept()
        self.channel.settimeout(15)
        ready = b''
        while len(ready) < 8:
            data = self.channel.recv(8-len(ready))
            if not data:
                raise AssertionError('Descendant exited before reporting readiness')
            ready += data
        self.child_pid, = struct.unpack('!Q', ready)
        if os.name == 'nt':
            import _winapi
            self.child_handle = _winapi.OpenProcess(_winapi.SYNCHRONIZE, False, self.child_pid)
        elif hasattr(os, 'pidfd_open'):
            self.child_handle = os.pidfd_open(self.child_pid)
        if self.channel.recv(1) != b'R':
            raise AssertionError('Descendant exited before reporting readiness')
        if not self.heartbeat.read_bytes():
            raise AssertionError('Descendant reported readiness without a heartbeat')
        if self.on_ready:
            self.on_ready(self)
        # Runner's real wait(timeout=...) begins only after readiness. No change
        # to the production timeout API or termination mechanism is needed.
        return process

    def assert_stopped(self, timeout=5):
        # Channel EOF alone precedes interpreter/OS handle cleanup. Wait for
        # actual process exit before asserting termination or deleting fixtures.
        if self.child_handle is not None:
            if os.name == 'nt':
                import _winapi
                self.stopped = _winapi.WaitForSingleObject(self.child_handle, int(timeout*1000)) == _winapi.WAIT_OBJECT_0
            else:
                self.stopped = bool(select.select([self.child_handle], [], [], timeout)[0])
        elif self.process is not None and self.process.returncode == 0:
            # The fixture parent returns normally only after reaping its child,
            # including a child that failed before completing the handshake.
            self.stopped = True
        elif self.child_pid is not None:
            deadline = time.monotonic() + timeout
            while time.monotonic() < deadline:
                try:
                    os.kill(self.child_pid, 0)
                except ProcessLookupError:
                    self.stopped = True
                    break
                time.sleep(.01)
        if self.stopped:
            return
        raise AssertionError('Watchdog left a live descendant')

    @staticmethod
    def force_stop(pid, tree=False):
        if os.name == 'nt':
            subprocess.run(['taskkill', '/PID', str(pid)] + (['/T'] if tree else []) + ['/F'],
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=15)
        else:
            try:
                (os.killpg if tree else os.kill)(pid, signal.SIGKILL)
            except ProcessLookupError:
                pass

    def close(self):
        try:
            # Cooperative cleanup is independent of Runner.kill, including for
            # negative controls and failures before the watchdog starts.
            if self.channel:
                try:
                    self.channel.sendall(b'Q')
                except OSError:
                    pass
            if self.process:
                try:
                    self.process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    self.force_stop(self.process.pid, tree=True)
                    self.process.wait(timeout=15)
            if self.channel:
                try:
                    self.assert_stopped()
                except AssertionError:
                    if self.child_pid:
                        self.force_stop(self.child_pid)
                    self.assert_stopped(timeout=15)
        finally:
            if self.channel:
                self.channel.close()
            if self.child_handle is not None:
                if os.name == 'nt':
                    import _winapi
                    _winapi.CloseHandle(self.child_handle)
                else:
                    os.close(self.child_handle)
            self.listener.close()


@contextmanager
def ready_tree(heartbeat, startup_delay=0, on_ready=None):
    fixture = TreeFixture(heartbeat, startup_delay, on_ready)
    try:
        with patch('execution.subprocess.Popen', side_effect=fixture.spawn):
            yield fixture
    finally:
        fixture.close()
