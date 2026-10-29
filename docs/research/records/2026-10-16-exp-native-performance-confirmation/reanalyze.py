# SPDX-License-Identifier: MPL-2.0
"""Check retained confirmation arithmetic and byte inventory without native work.

Uses only the Python standard library. This checks compact retained evidence;
it cannot revalidate unpublished raw frame ticks, logs or image bytes.
"""
import hashlib
import json
import math
from pathlib import Path
import statistics


ROOT = Path(__file__).resolve().parent
# Student t critical values at probabilities .95 and .975, for the retained
# eight-pair lifecycle, ten-block pilot and 112-block confirmation respectively.
CRITICAL = {
    7: (1.8945786050900044, 2.3646242515927813),
    9: (1.8331129326562343, 2.2621571627981982),
    111: (1.6586972654214476, 1.9815667570749098),
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def read(name):
    return json.loads((ROOT / name).read_text(encoding='utf-8'))


def close(actual, expected):
    require(math.isclose(actual, expected, rel_tol=1e-11, abs_tol=1e-7),
            f'Arithmetic mismatch: {actual} != {expected}')


def check_estimate(value):
    observations = value['block_differences_ns']
    count = len(observations)
    require(count == value['blocks'], 'Wrong block count')
    require(all(math.isfinite(x) for x in observations), 'Non-finite observation')
    mean = statistics.mean(observations)
    sd = statistics.stdev(observations)
    one, two = CRITICAL[count - 1]
    error = sd / math.sqrt(count)
    close(value['mean_difference_ns'], mean)
    close(value['block_sd_ns'], sd)
    close(value['lower_one_sided_95_ns'], mean - one * error)
    close(value['upper_one_sided_95_ns'], mean + one * error)
    for actual, expected in zip(value['two_sided_95_ns'],
                                (mean - two * error, mean + two * error)):
        close(actual, expected)
    budget = value['budget_ns']
    verdict = None if budget is None else (
        'within-budget' if mean + one * error < budget else
        'exceeds-budget' if mean - one * error > budget else 'inconclusive')
    require(value['verdict'] == verdict, 'Wrong budget verdict')


def walk_estimates(value):
    if isinstance(value, dict):
        if 'block_differences_ns' in value:
            check_estimate(value)
            return 1
        return sum(walk_estimates(item) for item in value.values())
    if isinstance(value, list):
        return sum(walk_estimates(item) for item in value)
    return 0


def main():
    entries = read('artifacts.json')
    names = set()
    for entry in entries:
        path = (ROOT / entry['path']).resolve()
        require(path.is_relative_to(ROOT) and not path.is_symlink(), 'Unsafe path')
        require(entry['path'] not in names, 'Duplicate inventory entry')
        names.add(entry['path'])
        content = path.read_bytes()
        require(len(content) == entry['bytes'], f'Wrong size: {path.name}')
        require(hashlib.sha256(content).hexdigest() == entry['sha256'],
                f'Wrong hash: {path.name}')
    actual = {p.relative_to(ROOT).as_posix() for p in ROOT.rglob('*')
              if p.is_file() and p.name != 'artifacts.json'}
    require(actual == names, 'Uninventoried or missing retained file')

    confirmation = read('confirmation.json')
    pilot = read('pilot.json')
    supplements = read('supplements.json')
    metrics = read('derived-metrics.json')
    require(walk_estimates(confirmation['findings']) == 80, 'Missing confirmation estimates')
    require(walk_estimates(pilot['findings']) == 80, 'Missing pilot estimates')
    require(walk_estimates(supplements['lifecycle_paired_differences']) == 12,
            'Missing lifecycle estimates')
    walk_estimates(metrics)
    require(confirmation['overall_cpu_verdict'] == 'within-budget', 'Wrong overall verdict')
    plan = read('selection.json')
    require(plan['blocks'] == 112 and len(plan['schedule']) == 112, 'Wrong frozen count')
    for workload, value in metrics['primary'].items():
        saved = read('processes-' + workload + '.json')
        rows = saved['rows']
        require(len(rows) == 224, 'Missing process rows')
        pairs = {}
        for arm in ('cpp-minimal', 'rust-wrapper'):
            arm_rows = sorted((r for r in rows if r[1] == arm), key=lambda r: r[0])
            require([r[0] for r in arm_rows] == list(range(1, 113)), 'Wrong process IDs')
            pairs[arm] = arm_rows
            for clock, column in (('cpu', 2), ('gpu', 5)):
                close(statistics.mean(r[column] for r in arm_rows),
                      value['consumers'][arm][clock]['mean_ns'])
        for clock, column in (('cpu', 2), ('gpu', 5)):
            differences = [rust[column] - cpp[column] for cpp, rust in zip(
                pairs['cpp-minimal'], pairs['rust-wrapper'])]
            require(differences == value[clock]['block_differences_ns'], 'Wrong paired values')
            require(value[clock] == confirmation['findings'][workload]['all'][clock],
                    'Conflicting primary estimates')
        require(value['cpu']['budget_ns'] == 5000 and value['cpu']['verdict'] == 'within-budget',
                'Wrong primary budget or result')
    allocations = metrics['allocations']
    require(sum(v['measured_dispatches'] for v in allocations.values()) == 1920,
            'Wrong allocation observation count')
    require(all(v['totals'] == [0] * 5 for v in allocations.values()), 'Allocation event observed')
    require([(c['status'], c['completed'], c['inputs_unchanged']) for c in read('checks.json')]
            == [('passed', 122, True), ('passed', 1220, True)], 'Wrong execution outcomes')
    print('Retained inventory, 80 confirmation + 80 pilot + 12 lifecycle estimates, '
          '1,120 process rows and allocation counts verified. No native execution.')


if __name__ == '__main__':
    main()
