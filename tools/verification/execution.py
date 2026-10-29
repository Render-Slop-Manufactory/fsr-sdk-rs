# SPDX-License-Identifier: MPL-2.0
"""Case DAG and bounded subprocess execution; all outcomes are recorded."""
from dataclasses import dataclass
import json
import hashlib
import os
from pathlib import Path
import signal
import shutil
import subprocess
import time

from records import inventory, now, save, Sanitizer, source_identity


class Blocked(Exception):
    pass


class Rejected(Exception):
    pass


def rust_lint_error(output, lint):
    """Require an actual Cargo compiler error, not an echoed lint name."""
    for line in output.splitlines():
        try:
            record = json.loads(line)
        except ValueError:
            continue
        if not isinstance(record, dict) or record.get('reason') != 'compiler-message':
            continue
        message = record.get('message')
        if isinstance(message, dict) and message.get('level') == 'error':
            code = message.get('code')
            if isinstance(code, dict) and code.get('code') == lint:
                return True
    return False


@dataclass
class Case:
    id: str
    claim: str
    action: object
    requires: tuple = ()
    selector: str = ''
    scope: str = 'host'
    expectation: object = None

    def plan(self):
        return {'id': self.id, 'claim': self.claim, 'requires': list(self.requires),
                'selector': self.selector, 'scope': self.scope,
                'expectation': self.expectation or {'evaluation':'passed','contract':self.claim}}


class Runner:
    def __init__(self, repository, directory, options, env=None):
        self.repository = Path(repository).resolve()
        self.directory = Path(directory).resolve()
        self.options = options
        self.env = dict(env if env is not None else os.environ)
        # Inherited diagnostic controls cannot alter an ordinary profile.
        self.env = {key: value for key, value in self.env.items()
                    if not key.startswith('FSR_SDK_')}
        self.env.update(CARGO_INCREMENTAL='0', CARGO_TERM_COLOR='never', RUST_BACKTRACE='1',
                        RUSTDOCFLAGS='-D warnings')
        self.state = {}
        self.results = {}
        self.current = None
        self.sanitizer = Sanitizer(self.repository, self.directory)
        (self.directory / 'cases').mkdir()
        (self.directory / 'readbacks').mkdir()

    def cargo(self, arguments, toolchain=None):
        toolchain = toolchain or self.options.toolchain
        prefix = ['cargo'] + ([f'+{toolchain}'] if toolchain else [])
        arguments = list(arguments)
        if arguments[0] != 'fmt':
            insertion = arguments.index('--') if '--' in arguments else len(arguments)
            arguments[insertion:insertion] = ([] if arguments[0] == 'generate-lockfile' else ['--locked']) + ([] if self.options.online else ['--offline'])
        return prefix + arguments

    def process(self, args, *, cwd=None, env=None, timeout=1200, diagnostic=None, lint=None,
                markers=(), observe=False, name='command'):
        record = self.current
        if not record:
            raise RuntimeError('Processes must belong to a planned case')
        directory = self.directory / 'cases' / record['id']
        sequence = len(record['commands'])
        prefix = f'{sequence:02d}-{name}'
        out, err = directory / f'{prefix}.stdout.txt', directory / f'{prefix}.stderr.txt'
        environment = dict(self.env)
        if env:
            environment.update({key: str(value) for key, value in env.items() if value is not None})
            for key, value in env.items():
                if value is None:
                    environment.pop(key, None)
        args = [str(arg) for arg in args]
        # Windows CreateProcess searches the parent's PATH, not the supplied
        # child environment. Resolve against the acquired developer-shell PATH.
        if not Path(args[0]).is_absolute():
            resolved = shutil.which(args[0], path=environment.get('PATH', environment.get('Path')))
            if resolved:
                args[0] = resolved
        command = {'args': self.sanitizer.value(args), 'cwd': self.sanitizer.text(cwd or self.repository),
                   'started_utc': now(), 'timeout_seconds': timeout,
                   'expectation': {'diagnostic': diagnostic, 'markers': list(markers), 'observation_only': observe},
                   'stdout': out.relative_to(self.directory).as_posix(),
                   'stderr': err.relative_to(self.directory).as_posix(), 'execution': 'running'}
        command['environment_overrides'] = self.sanitizer.value(env or {})
        if lint:
            command['expectation']['lint'] = lint
        record['commands'].append(command)
        save(directory / 'result.json', record)
        start = time.monotonic()
        process = None
        outcome = 'completed'
        exit_code = None
        with out.open('wb') as stdout, err.open('wb') as stderr:
            try:
                process = subprocess.Popen(args, cwd=cwd or self.repository, env=environment,
                    stdout=stdout, stderr=stderr, start_new_session=os.name != 'nt',
                    creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
                exit_code = process.wait(timeout=timeout)
                if (os.name != 'nt' and exit_code < 0) or (os.name == 'nt' and exit_code & 0x80000000):
                    outcome = 'aborted'
            except subprocess.TimeoutExpired:
                outcome = 'timed_out'
                self.kill(process)
                exit_code = process.wait(timeout=15)
            except OSError as error:
                outcome = 'blocked'
                stderr.write(str(error).encode('utf-8', errors='replace'))
            except KeyboardInterrupt:
                outcome = 'interrupted'
                if process:
                    self.kill(process)
                    exit_code = process.wait(timeout=15)
                raise
            finally:
                command.update(execution=outcome, exit_code=exit_code, finished_utc=now(),
                               elapsed_seconds=round(time.monotonic() - start, 3))
                save(directory / 'result.json', record)
        stdout = out.read_text(encoding='utf-8', errors='replace')
        stderr = err.read_text(encoding='utf-8', errors='replace')
        text = stdout + stderr
        if outcome == 'blocked':
            raise Blocked(self.sanitizer.text(stderr))
        if observe:
            record['evaluation'] = 'observed'
        else:
            accepted = outcome == 'completed' and all(marker in text for marker in markers)
            if lint:
                accepted = accepted and exit_code != 0 and rust_lint_error(stdout, lint)
            else:
                accepted = accepted and ((exit_code != 0 and f'error[{diagnostic}]' in text)
                                         if diagnostic else exit_code == 0)
            if not accepted:
                raise Rejected(f'{name}: {outcome}, exit={exit_code}; expectation not met')
        return stdout, stderr, command

    @staticmethod
    def kill(process):
        if os.name == 'nt':
            subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'],
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=15)
            if process.poll() is None:
                process.kill()
        else:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass

    def note(self, **observations):
        self.current['observations'].update(self.sanitizer.value(observations))

    def execute(self, cases, profile, *, require_passed=False):
        ids = [case.id for case in cases]
        if len(set(ids)) != len(ids):
            raise ValueError('Case IDs must be unique in a profile')
        preceding = set()
        for case in cases:
            if not set(case.requires).issubset(preceding):
                raise ValueError(f'Invalid prerequisites for {case.id}')
            preceding.add(case.id)
        inputs = source_identity(self.repository, self.state.get('external_inputs', ()))
        source_digest = inputs['identity']
        inputs['source_identity'] = source_digest
        inputs['configuration'] = self.sanitizer.value(vars(self.options))
        inputs['environment'] = self.sanitizer.value({key:self.env.get(key) for key in
            ('RUSTFLAGS','RUSTDOCFLAGS','CARGO_INCREMENTAL','CARGO_BUILD_TARGET','CARGO_HOME','RUSTUP_TOOLCHAIN')})
        inputs['identity'] = hashlib.sha256(json.dumps(inputs,sort_keys=True).encode()).hexdigest()
        save(self.directory / 'inputs.json', inputs)
        save(self.directory / 'plan.json', {'schema_version': 1, 'profile': profile,
             'required_cases': [case.plan() for case in cases], 'resume': 'disabled'})
        summary = {'schema_version': 1, 'profile': profile, 'started_utc': now(),
                   'status': 'running', 'input_identity': inputs['identity'], 'required': ids}
        save(self.directory / 'run.json', summary)
        interrupted = False
        try:
            for case in cases:
                directory = self.directory / 'cases' / case.id
                directory.mkdir()
                record = dict(case.plan(), schema_version=1, started_utc=now(),
                              evaluation='passed', execution='not_started', commands=[], observations={})
                self.current = record
                missing = [name for name in case.requires if self.results[name]['evaluation'] != 'passed']
                try:
                    if missing or interrupted:
                        raise Blocked('Unmet prerequisites: ' + ', '.join(missing) if missing else 'Run interrupted')
                    record['execution'] = 'running'
                    save(directory / 'result.json', record)
                    case.action(self)
                    record['execution'] = record['commands'][-1]['execution'] if record['evaluation']=='observed' and record['commands'] else 'completed'
                except Blocked as error:
                    record.update(execution='blocked', evaluation='blocked', reason=self.sanitizer.text(error))
                except KeyboardInterrupt:
                    interrupted = True
                    record.update(execution='interrupted', evaluation='failed', reason='Interrupted by operator')
                except Exception as error:
                    outcome = record['commands'][-1]['execution'] if record['commands'] else 'failed'
                    record.update(execution=outcome,
                                  evaluation='failed', reason=self.sanitizer.text(error))
                record['finished_utc'] = now()
                self.results[case.id] = record
                save(directory / 'result.json', record)
                print(f'{case.id}: {record["evaluation"]}' + (f' ({record["reason"]})' if 'reason' in record else ''), flush=True)
                summary['cases'] = {name: item['evaluation'] for name, item in self.results.items()}
                save(self.directory / 'run.json', summary)
        finally:
            self.current = None
            after = source_identity(self.repository, self.state.get('external_inputs', ()))
            unchanged = after['identity'] == source_digest
            evaluations = [self.results.get(name, {}).get('evaluation', 'missing') for name in ids]
            status = 'passed' if unchanged and all(item == 'passed' for item in evaluations) else 'failed'
            if unchanged and evaluations and all(item in ('passed', 'observed') for item in evaluations) and 'observed' in evaluations:
                status = 'observed'
            summary.update(status=status, finished_utc=now(), inputs_unchanged=unchanged,
                           completed=len(self.results), cases={name: item['evaluation'] for name, item in self.results.items()})
            save(self.directory / 'run.json', summary)
            # Builds and distributions stay local; inventories include all retained
            # files so export can detect corruption before copying any evidence.
            save(self.directory / 'artifacts.json', inventory(self.directory))
        if require_passed and summary['status'] != 'passed':
            print('Acceptance requires passed status; recorded status: '+summary['status'], flush=True)
        accepted = ('passed',) if require_passed else ('passed', 'observed')
        return 0 if summary['status'] in accepted else 1
