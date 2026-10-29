# SPDX-License-Identifier: MPL-2.0
"""Unattended release performance check reusing P06 and P07 equipment."""
import argparse
import contextlib
import json
import math
from pathlib import Path
import re
import statistics
import subprocess
import sys
import tomllib

from execution import Rejected
import performance_characterization as dispatch
from records import fresh, now, save, sha, source_identity
from release import archive_files
from run import verify_run

ROOT = Path(__file__).resolve().parents[2]
PROTOCOL = Path(__file__).with_name('release-performance-protocol.json')


def protocol():
    spec = json.loads(PROTOCOL.read_text(encoding='utf-8'))
    if (sha(dispatch.PROTOCOL) != spec['base_protocol_sha256'] or
            dispatch.protocol()['workloads'] != spec['workloads'] or
            dispatch.protocol()['cpu_budget_ns'] != spec['cpu_budget_ns']):
        raise Rejected('P08/P06 protocol mismatch; review the adaptation before measuring')
    return spec


def candidates(directory, wrapper=None, raw=None):
    """Bind native source builds to Cargo's actual package source selection."""
    if bool(wrapper) != bool(raw):
        raise Rejected('Supply both candidate archives, or neither to package the workspace')
    if not wrapper:
        build = directory / 'candidate-build'
        args = ['cargo', 'package', '--workspace', '--no-verify', '--allow-dirty',
                '--offline', '--locked', '--target-dir', str(build)]
        with (directory / 'package.log').open('x', encoding='utf-8', newline='\n') as log:
            subprocess.run(args, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, check=True)
        paths = {}
        for name in ('fsr-sdk', 'fsr-sdk-sys'):
            manifest = tomllib.loads((ROOT/'crates'/name/'Cargo.toml').read_text(encoding='utf-8'))
            paths[name] = build/'package'/f'{name}-{manifest["package"]["version"]}.crate'
    else:
        paths = {'fsr-sdk': Path(wrapper).resolve(), 'fsr-sdk-sys': Path(raw).resolve()}
    generated = {'Cargo.toml', 'Cargo.lock', '.cargo_vcs_info.json'}
    result = {}
    for name, path in paths.items():
        source = ROOT/'crates'/name
        version = tomllib.loads((source/'Cargo.toml').read_text(encoding='utf-8'))['package']['version']
        files = archive_files(path, f'{name}-{version}')
        listed = subprocess.run(['cargo', 'package', '--list', '-p', name, '--allow-dirty',
            '--offline', '--locked'], cwd=ROOT, capture_output=True, text=True, check=True)
        expected = set(listed.stdout.splitlines()) - generated
        if set(files)-generated != expected or 'Cargo.toml.orig' not in expected:
            raise Rejected('Candidate package source inventory differs: '+name)
        for inner in expected:
            original = source/('Cargo.toml' if inner == 'Cargo.toml.orig' else inner)
            if not original.is_file() or original.read_bytes() != files[inner]:
                raise Rejected('Candidate source bytes differ: '+name+'/'+inner)
        result[name] = {'path': str(path.resolve()), 'sha256': sha(path),
                        'source_files_checked': len(expected), 'version': version}
    return result


def setup_samples(log, blocks):
    rows = re.findall(r'^PROVIDER_SETUP,(\d+),(default|selected),(\d+)$', log, re.M)
    values = {(int(block), arm): int(ns) for block, arm, ns in rows}
    expected = {(block, arm) for block in range(blocks) for arm in ('default', 'selected')}
    identity = re.findall(r'^Provider setup identity: (.+)$', log, re.M)
    if len(rows) != len(expected) or set(values) != expected or len(identity) != 1:
        raise Rejected('Incomplete, duplicate or unidentified provider setup measurements')
    return {'provider': identity[0], 'samples_ns': {arm: [values[b, arm] for b in range(blocks)]
            for arm in ('default', 'selected')},
            'paired_selected_minus_default_ns': [values[b, 'selected']-values[b, 'default'] for b in range(blocks)],
            'median_ns': {arm: statistics.median(values[b, arm] for b in range(blocks))
                          for arm in ('default', 'selected')}}


def unchanged(source, archives):
    if source_identity(ROOT) != source:
        raise Rejected('Checkout changed during P08; do not mix candidate measurements')
    for archive in archives.values():
        if sha(Path(archive['path'])) != archive['sha256']:
            raise Rejected('Candidate archive changed during P08')


def matched_run(path, source, expected_sdk=None):
    summary = verify_run(path)
    inputs = json.loads((path/'inputs.json').read_text(encoding='utf-8'))
    expected = {row['path']: row['sha256'] for row in source['files']}
    actual = {row['path']: row['sha256'] for row in inputs['files'] if row['path'] in expected}
    if summary['status'] != 'passed' or not summary['inputs_unchanged'] or actual != expected:
        raise Rejected('P08 stage lacks passed evidence for the selected source')
    sdk = {row['path']: row['sha256'] for row in inputs['files'] if row['path'].startswith('sdk/')}
    if not sdk or (expected_sdk is not None and sdk != expected_sdk):
        raise Rejected('SDK inputs differ between P08 stages')
    return sdk


def prepare_reconfirmation(planning_batch, output):
    """Freeze new observations only; a verified earlier result supplies planning data."""
    previous = json.loads((Path(planning_batch)/'status.json').read_text(encoding='utf-8'))
    if previous['status'] != 'completed':
        raise Rejected('Reconfirmation planning requires a completed P08 batch')
    analysis_path = Path(previous['confirmation_analysis']).resolve()
    run = analysis_path.parent.parent
    checked = verify_run(run)
    if checked['status'] != 'passed' or not checked['inputs_unchanged']:
        raise Rejected('Planning confirmation lacks complete valid evidence')
    analysis = json.loads(analysis_path.read_text(encoding='utf-8'))
    if analysis['phase'] != 'confirmation' or analysis['protocol'] != dispatch.protocol():
        raise Rejected('Planning result must use the original P06/P08 protocol')
    # Tooling can change for the new protocol; packaged native/Rust sources must
    # still match. The old complete producer identity remains planning provenance.
    archives = previous['archives']
    checked_archives = candidates(Path(output).parent, archives['fsr-sdk']['path'], archives['fsr-sdk-sys']['path'])
    if any(checked_archives[name]['sha256'] != archives[name]['sha256'] for name in checked_archives):
        raise Rejected('Planning archives changed')
    inputs = json.loads((run/'inputs.json').read_text(encoding='utf-8'))
    sdk = {row['path']: row['sha256'] for row in inputs['files'] if row['path'].startswith('sdk/')}
    if not sdk or sdk != previous['sdk_inputs']:
        raise Rejected('Planning SDK identity mismatch')
    spec = dispatch.protocol(dispatch.RECONFIRM_PROTOCOL)
    n = spec['minimum_confirmation_blocks']
    if n != spec['maximum_confirmation_blocks'] or spec['cpu_budget_ns'] != protocol()['cpu_budget_ns']:
        raise Rejected('Reconfirmation requires one fixed count and the unchanged P08 criterion')
    findings = [analysis['findings'][mode]['all']['cpu'] for mode in spec['workloads']]
    counts = {finding['blocks'] for finding in findings}
    if len(counts) != 1 or min(counts) < 2:
        raise Rejected('Planning block counts differ or are incomplete')
    deviations = [finding['block_sd_ns'] for finding in findings]
    if not all(math.isfinite(value) and value >= 0 for value in deviations):
        raise Rejected('Invalid planning dispersion')
    sd = max(deviations)
    seconds = analysis['measurement_seconds']/counts.pop()
    if not math.isfinite(sd) or sd < 0 or not math.isfinite(seconds) or seconds <= 0:
        raise Rejected('Invalid planning dispersion or duration')
    margin = dispatch.t_quantile(.95, n-1)*sd/math.sqrt(n)
    predicted = n*seconds*spec['runtime_planning_multiplier']
    if margin > spec['planning_half_width_ns'] or predicted > spec['confirmation_measurement_limit_seconds']:
        raise Rejected('Planning noise/runtime no longer fits the reviewed fixed protocol')
    plan = {'schema_version': 1, 'protocol_sha256': sha(dispatch.RECONFIRM_PROTOCOL),
        'producer_identity': source_identity(ROOT)['identity'], 'blocks': n,
        'seed': spec['confirmation_seed'], 'planned_margin_ns': margin,
        'planned_precision_feasible': True, 'predicted_measurement_seconds': predicted,
        'measurement_limit_seconds': spec['confirmation_measurement_limit_seconds'],
        'pilot_equipment': analysis['equipment'], 'schedule': dispatch.schedule(n, spec['confirmation_seed'], spec),
        'analysis': spec['analysis'], 'cpu_budget_ns': spec['cpu_budget_ns'],
        'planning_only': {'batch': str(Path(planning_batch).resolve()), 'run': str(run),
            'source_identity': previous['source_identity'], 'input_identity': inputs['identity'],
            'analysis_sha256': sha(analysis_path), 'sdk_inputs': sdk,
            'blocks': findings[0]['blocks'], 'worst_block_sd_ns': sd,
            'pool_previous_samples': False}}
    with Path(output).open('x', encoding='utf-8', newline='\n') as stream:
        stream.write(json.dumps(plan, indent=2)+'\n')
    return plan


def run_batch(sdk_root, adapter_index=0, output_root=None, wrapper=None, raw=None, planning_batch=None):
    from run import main as run_profile

    spec = protocol()
    directory = fresh(Path(output_root or ROOT/'target/release-performance').resolve(), 'release-performance')
    source = source_identity(ROOT)
    status_path = directory/'status.json'
    index = {'status': 'running', 'started_utc': now(), 'stages': [],
             'protocol_sha256': sha(PROTOCOL), 'source_identity': source['identity'],
             'release_disposition': 'review-required', 'protocol': spec,
             'dispatch_mode': 'independent-fixed-confirmation' if planning_batch else 'adaptive-pilot-confirmation'}
    save(directory/'source.json', source)
    save(directory/'protocol.json', spec)
    save(status_path, index)
    print('P08 batch: '+str(directory), flush=True)
    try:
        print('Creating/checking candidate archives; package.log records packaging', flush=True)
        archives = candidates(directory, wrapper, raw)
        index['archives'] = archives
        save(status_path, index)
        unchanged(source, archives)
        plan_path = directory/'selection.json' if planning_batch else None
        if plan_path:
            plan = prepare_reconfirmation(planning_batch, plan_path)
            index['reconfirmation'] = {'plan': str(plan_path), 'blocks': plan['blocks'],
                'protocol_sha256': plan['protocol_sha256'], 'planning_only': plan['planning_only']}
            save(directory/'reconfirmation-protocol.json', dispatch.protocol(dispatch.RECONFIRM_PROTOCOL))
            save(status_path, index)
        setup = []
        for repeat in range(1, spec['setup_processes']+1):
            unchanged(source, archives)
            log = directory/f'setup-{repeat}.log'
            print(f'Starting setup {repeat}/{spec["setup_processes"]}; log: {log}', flush=True)
            output = directory/f'setup-{repeat}'
            with log.open('x', encoding='utf-8', newline='\n') as stream:
                with contextlib.redirect_stdout(stream), contextlib.redirect_stderr(stream):
                    code = run_profile(['run', spec['setup_profile'], '--native', '--require-passed',
                        '--sdk-root', str(Path(sdk_root).resolve()), '--adapter-index', str(adapter_index),
                        '--output-root', str(output)])
            runs = list(output.glob('*/run.json'))
            if code or len(runs) != 1:
                raise Rejected('Setup stage failed; see '+str(log))
            path = runs[0].parent
            index['sdk_inputs'] = matched_run(path, source, index.get('sdk_inputs'))
            if plan_path and index['sdk_inputs'] != plan['planning_only']['sdk_inputs']:
                raise Rejected('SDK changed since the planning measurement')
            record = json.loads((path/'cases/provider-setup.queries/result.json').read_text(encoding='utf-8'))
            telemetry = ''.join((path/command['stderr']).read_text(encoding='utf-8') for command in record['commands'])
            sample = setup_samples(telemetry, spec['setup_blocks_per_process'])
            if setup and sample['provider'] != setup[0]['provider']:
                raise Rejected('Provider changed between setup processes')
            setup.append(sample)
            index['stages'].append({'stage': f'setup-{repeat}', 'run': str(path), 'status': 'passed'})
            save(status_path, index)
        save(directory/'setup-summary.json', {'processes': setup, 'speed_threshold': None,
            'limit': 'Whole six-query paths; descriptive, no causal enumeration-cost or old/new API attribution.'})
        unchanged(source, archives)
        log = directory/'dispatch.log'
        print(('Starting fixed 256-block confirmation' if plan_path else 'Starting P06 pilot + selected confirmation')+
              '; progress log: '+str(log), flush=True)
        with log.open('x', encoding='utf-8', newline='\n') as stream:
            with contextlib.redirect_stdout(stream), contextlib.redirect_stderr(stream):
                # Keep the legacy P06 batch beside P08, rather than adding another
                # timestamped directory level that exceeds MSVC's Windows path limit.
                batch = (dispatch.run_batch(sdk_root, adapter_index, directory.parent, fixed_plan=plan_path)
                         if plan_path else dispatch.run_batch(sdk_root, adapter_index, directory.parent))
        result = json.loads((batch/'status.json').read_text(encoding='utf-8'))
        if result['status'] != 'completed':
            raise Rejected('Dispatch batch did not complete')
        for stage in result['stages'].values():
            matched_run(Path(stage['run']), source, index['sdk_inputs'])
        analysis_path = Path(result['confirmation_analysis'])
        analysis = json.loads(analysis_path.read_text(encoding='utf-8'))
        expected = dispatch.protocol(dispatch.RECONFIRM_PROTOCOL if plan_path else None)
        if analysis['protocol'] != expected or analysis['protocol']['cpu_budget_ns'] != spec['cpu_budget_ns']:
            raise Rejected('Dispatch analysis does not use the selected P08 budget/protocol')
        unchanged(source, archives)
        index.update(status='completed', cpu_verdict=analysis['overall_cpu_verdict'],
            dispatch_batch=str(batch), confirmation_analysis=str(analysis_path),
            setup_analysis=str(directory/'setup-summary.json'),
            limit='Measurements complete, release review still required. Source-matched archives; exact archive qualification remains separate.')
    except BaseException as error:
        index.update(status='interrupted' if isinstance(error, KeyboardInterrupt) else 'failed', reason=str(error))
        raise
    finally:
        index['finished_utc'] = now()
        save(status_path, index)
        print('Status file: '+str(status_path), flush=True)
    print('Completed; CPU verdict: '+str(index['cpu_verdict'])+'; review required', flush=True)
    print('Ping with this result path: '+str(status_path), flush=True)
    return directory


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    actions = parser.add_subparsers(dest='command', required=True)
    preview = actions.add_parser('plan', help='Show the prepared protocol without native work')
    preview.add_argument('--reconfirmation', action='store_true', help='Show the independent fixed 256-block protocol')
    for name, help_text in (('run', 'Run unattended P07 + fresh adaptive P06 measurement'),
                            ('reconfirm', 'Run P07 + exactly 256 fresh paired blocks; prior data for planning only')):
        command = actions.add_parser(name, help=help_text)
        command.add_argument('--sdk-root', required=True)
        command.add_argument('--adapter-index', type=int, default=0)
        command.add_argument('--output-root')
        command.add_argument('--wrapper-archive')
        command.add_argument('--sys-archive')
        if name == 'reconfirm':
            command.add_argument('--planning-batch', required=True)
    args = parser.parse_args(argv)
    try:
        if args.command == 'plan':
            print(json.dumps(dispatch.protocol(dispatch.RECONFIRM_PROTOCOL) if args.reconfirmation else protocol(), indent=2))
        else:
            if args.adapter_index < 0:
                raise ValueError('Adapter index must be nonnegative')
            if args.command == 'reconfirm':
                run_batch(args.sdk_root, args.adapter_index, args.output_root, args.wrapper_archive,
                          args.sys_archive, planning_batch=args.planning_batch)
            else:
                run_batch(args.sdk_root, args.adapter_index, args.output_root, args.wrapper_archive, args.sys_archive)
        return 0
    except (Rejected, ValueError, OSError, subprocess.CalledProcessError) as error:
        print(str(error), file=sys.stderr)
        return 1


if __name__ == '__main__':
    sys.exit(main())
