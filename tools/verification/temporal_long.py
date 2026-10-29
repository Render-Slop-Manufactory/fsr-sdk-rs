# SPDX-License-Identifier: MPL-2.0
"""Continuous 256-frame histories, with independent acceptance per time window."""
import argparse
import json
from pathlib import Path

from records import save, sha
from temporal_calibration import np_module
from temporal_diagnosis import HERE, arms, frame_plan, rectangle
from temporal_quality import (collect_metrics, evaluate, pooled_metrics,
                              sensitivity, synthetic_controls)

NAMESPACE='experiment.temporal-long'


def protocol():
    return json.loads((HERE/'temporal-long-protocol.json').read_text())


def windows(spec):
    full,_=frame_plan(spec)
    expected=[[start,start+31] for start in range(32,len(full),32)]
    if spec['windows']!=expected:
        raise ValueError('Windows must cover every post-warmup frame exactly once')
    for scene in spec['scenes']:
        for frame in full:
            l,t,r,b=rectangle(dict(frame=frame,velocity=scene['velocity']))
            if not (4<l<r<316 and 4<t<b<176):
                raise ValueError('Foreground leaves the measured viewport')
    return expected


def evaluate_block(data,spec):
    """A late failure cannot be compensated by other windows or properties."""
    intervals=windows(spec); expected=list(range(32,spec['frames']))
    planned=arms(spec)
    if set(data)!={a['name'] for a in planned}:
        raise ValueError('Incomplete arm inventory')
    for measurements in data.values():
        if [m['frame'] for m in measurements]!=expected:
            raise ValueError('Incomplete, duplicated or unordered measured frames')
    result=[]
    for start,end in intervals:
        values={name:pooled_metrics([m['scores'] for m in measurements if start<=m['frame']<=end])
                for name,measurements in data.items()}
        correct=values['textured-slow-correct']; reverse=values['textured-slow-reverse']
        budget=evaluate(correct,spec['native_budgets'],pooled=True)
        checks={metric:sensitivity(correct,reverse,metric,spec['minimum_motion_ratio'])
                for metric in ('foreground','temporal')}
        result.append(dict(frames=[start,end],arms=values,budget=budget,sensitivity=checks,
            passed=budget['passed'] and all(c['passed'] for c in checks.values())))
    trajectories={name:[dict(frame=m['frame'],**{metric:item['nrmse' if metric=='texture' else 'mae']
                    for metric,item in m['scores'].items()}) for m in measurements]
                  for name,measurements in data.items()}
    return dict(windows=result,trajectories=trajectories,passed=all(w['passed'] for w in result))


def audit(run,output):
    spec=protocol(); windows(spec)
    output=Path(output); output.mkdir(parents=True,exist_ok=False)
    controls=synthetic_controls(); data,same=collect_metrics(run,spec,NAMESPACE)
    blocks=[]
    for repeat in range(1,spec['repeats']+1):
        block=evaluate_block({a['name']:data[repeat,a['name']] for a in arms(spec)},spec)
        block.update(repeat=repeat,selected_outputs_identical_repeat1=same.get(repeat))
        blocks.append(block)
    result=dict(protocol_sha256=sha(HERE/'temporal-long-protocol.json'),numpy=np_module().__version__,
        scope='Continuous textured-slow histories on one equipment configuration; 32-frame warmup then seven independent acceptance windows',
        evaluator_validation=controls,blocks=blocks,output_frames=len(arms(spec))*spec['frames']*spec['repeats'],
        passed=controls['passed'] and all(b['passed'] for b in blocks),
        statistical_claim='Frames and windows are correlated. Repeats are process blocks on one GPU/driver, not population confidence intervals.')
    save(output/'summary.json',result)
    return result


def cases(backend='windows'):
    from temporal_quality import cases as quality_cases
    windows(protocol())
    return quality_cases(dict(namespace=NAMESPACE,protocol='temporal-long-protocol.json',
        script='temporal_long.py',analysis='temporal-long',backend=backend))


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=['audit']); parser.add_argument('--run',required=True)
    parser.add_argument('--output',required=True); args=parser.parse_args()
    raise SystemExit(0 if audit(args.run,args.output)['passed'] else 1)
