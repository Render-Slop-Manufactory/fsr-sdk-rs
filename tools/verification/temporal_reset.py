# SPDX-License-Identifier: MPL-2.0
"""Paired cut/reset recovery, with separate exploration and confirmation."""
import argparse
import json
from pathlib import Path
import shutil

from execution import Blocked, Case, Rejected
from records import save, sha
from temporal_calibration import image, np_module, payload
from temporal_diagnosis import HERE, arms, file_path, frame_plan, measure, references
from temporal_quality import age_step, evaluate, frame_metrics, pooled_metrics

METRICS=('foreground','background','edge','temporal','ghost')


def protocol(confirm=False):
    value=json.loads((HERE/'temporal-reset-protocol.json').read_text())
    if confirm: value.update(confirmation=True,repeats=3,order_seeds=[419,827,1237])
    return value


def ghost_controls(measurer=measure):
    before=references(dict(scene=2,frame=63,velocity=[.375,.25],cut_frame=64))
    after=references(dict(scene=2,frame=64,velocity=[.375,.25],cut_frame=64))
    mask=before[2]['foreground']&after[2]['background']; scores=[]
    for severity in (0,.25,.5,1):
        error=severity*(before[0]-after[0]); score=measurer(error,mask)
        scores.append(dict(severity=severity,score=score))
    values=[s['score']['mae'] for s in scores]
    passed=all(s['score']['pixels']>0 for s in scores) and values[0] is not None and values[0]<=.002
    passed=passed and all(v is not None and v>.002 for v in values[1:]) and all(a<b for a,b in zip(values,values[1:]))
    return dict(passed=passed,scope='Analytic old-frame contamination; CPU only',limit=.002,controls=scores)


def pool_reset(values):
    result=pooled_metrics(values)
    from temporal_calibration import pool
    result['ghost']=pool(v['ghost'] for v in values)
    return result


def criteria_controls(criteria):
    """Calibration must not make known defective images acceptable."""
    import math
    from temporal_quality import synthetic_controls
    known=synthetic_controls()['controls']; scores={}
    for metric,kind,severity in (('foreground','missing-object',1),('background','constant-foreground',1),
                                ('edge','blur',1),('temporal','flicker',.01)):
        scores[metric]=next(c['scores'][metric]['mae'] for c in known if c['control']==kind and c['severity']==severity)
    scores['ghost']=ghost_controls()['controls'][1]['score']['mae']
    checks={metric:dict(defect_mae=value,limit=criteria['native_budgets'][metric],
        passed=math.isfinite(criteria['native_budgets'][metric]) and 0<criteria['native_budgets'][metric]<value)
        for metric,value in scores.items()}
    return dict(properties=checks,passed=all(c['passed'] for c in checks.values()),
        scope='Known analytic missing-object, constant-image, blur, flicker and 25% old-history defects; not AMD outputs')


def verdict(block,criteria):
    quality=[dict(frames=w['frames'],arms={name:evaluate(w['arms'][name],criteria['native_budgets'],pooled=True)
              for name in ('reset','fresh')}) for w in block['windows']]
    recovery=all(p['reset_fresh_maximum']<=criteria['pair_maximum'] for p in block['pairs'])
    sensitivity=block['early_no_reset_mae']>=block['early_reset_mae']+criteria['minimum_history_difference']
    return dict(quality=quality,recovery=recovery,history_sensitive=sensitivity,
        passed=recovery and sensitivity and all(v['passed'] for w in quality for v in w['arms'].values()))


def audit(run,output):
    run=Path(run); profile=json.loads((run/'run.json').read_text())['profile']; ns=profile.replace('proton-temporal-','experiment.temporal-',1).replace('experiment-','experiment.',1)
    spec=json.loads((run/'cases'/f'{ns}.protocol/protocol.json').read_text()); full,_=frame_plan(spec)
    output=Path(output); output.mkdir(parents=True,exist_ok=False); np=np_module()
    planned=arms(spec); blocks=[]; cut=spec['cut_frame']
    for repeat in range(1,spec['repeats']+1):
        directory=run/'readbacks'/f'{ns}.r{repeat}'
        record=json.loads((run/'cases'/f'{ns}.r{repeat}/result.json').read_text())
        if record['evaluation']!='passed' or record['observations']['input_audit']['frames']!=320:
            raise ValueError('Missing complete independent input/output audit')
        data={a['name'].split('cut-',1)[1]:[] for a in planned}; pairs=[]
        previous_ref=None; prior={}; age=np.full((360,640),999); old=None
        for frame in full:
            ref=references(dict(scene=2,frame=frame,velocity=[.375,.25],cut_frame=cut))
            if previous_ref is not None: age=age_step(age,ref[4],previous_ref[4])
            if frame==cut: old=previous_ref[2]['foreground'].copy()
            if frame>=cut:
                outputs={}
                for a in planned:
                    name=a['name'].split('cut-',1)[1]; meta=dict(arm=a['name'],frame=frame)
                    outputs[name]=image(file_path(directory,meta,'rgba16f'))
                    scores=frame_metrics(outputs[name][...,:3],prior.get(name),ref,
                        previous_ref,[.375,.25],age)
                    scores['ghost']=measure(outputs[name][...,:3]-ref[0],old&ref[2]['background'])
                    data[name].append(dict(frame=frame,scores=scores))
                    prior[name]=outputs[name][...,:3]-ref[0]
                # Fresh and history arms must differ only in temporal state/reset.
                inputs=[payload(file_path(directory,dict(arm=a['name'],frame=frame),'inputs')) for a in planned]
                if not all(raw==inputs[0] for raw in inputs[1:]): raise ValueError('Unpaired post-cut inputs')
                mask=ref[2]['interior']
                pairs.append(dict(frame=frame,reset_fresh_maximum=float(np.abs(outputs['reset']-outputs['fresh']).max()),
                    reset_fresh=measure(outputs['reset'][...,:3]-outputs['fresh'][...,:3],mask),
                    no_reset_fresh=measure(outputs['no-reset'][...,:3]-outputs['fresh'][...,:3],mask)))
            previous_ref=ref
        windows=[]
        for start,end in spec['windows']:
            windows.append(dict(frames=[cut+start,cut+end],arms={name:pool_reset([m['scores'] for m in values
                if cut+start<=m['frame']<=cut+end]) for name,values in data.items()}))
        trajectories={name:[dict(frame=m['frame'],**{metric:item['nrmse' if metric=='texture' else 'mae']
                    for metric,item in m['scores'].items()}) for m in values] for name,values in data.items()}
        block=dict(repeat=repeat,windows=windows,trajectories=trajectories,pairs=pairs,
            early_reset_mae=sum(p['reset_fresh']['mae'] for p in pairs[:8])/8,
            early_no_reset_mae=sum(p['no_reset_fresh']['mae'] for p in pairs[:8])/8)
        blocks.append(block); print(f'Evaluated reset block {repeat}',flush=True)
    result=dict(protocol=spec,protocol_sha256=sha(run/'cases'/f'{ns}.protocol/protocol.json'),
        numpy=np.__version__,ghost_controls=ghost_controls(),blocks=blocks,output_frames=320*spec['repeats'],
        scope='Synthetic camera cut; correlated frames on one equipment configuration; no universal reset/perceptual guarantee')
    if spec['confirmation']:
        criteria=json.loads((run/'cases'/f'{ns}.criteria/criteria.json').read_text())
        result['criteria']=criteria
        result['criteria_controls']=criteria_controls(criteria)
        for block in blocks: block['verdict']=verdict(block,criteria)
        result['passed']=result['ghost_controls']['passed'] and result['criteria_controls']['passed'] and all(b['verdict']['passed'] for b in blocks)
    else:
        result['passed']=result['ghost_controls']['passed']
        result['acceptance']='Exploration completion only; no quality/reset acceptance'
    save(output/'summary.json',result); return result


def select(run,output):
    from run import verify_run
    run=Path(run); state=verify_run(run)
    if state['status']!='passed' or state['profile']!='experiment-temporal-reset-explore':
        raise ValueError('Selection requires a completed exploration')
    source=run/'analysis/temporal-reset/summary.json'; values=json.loads(source.read_text())
    if values['protocol']['confirmation'] or len(values['blocks'])!=1: raise ValueError('Not a single exploration block')
    block=values['blocks'][0]
    budgets={metric:1.25*max(w['arms']['fresh'][metric]['mae'] for w in block['windows'])+1e-6 for metric in METRICS}
    result=dict(schema_version=1,exploration_input_identity=state['input_identity'],exploration_summary_sha256=sha(source),
        native_budgets=budgets,pair_maximum=1e-6,minimum_history_difference=1e-4,
        rule='1.25 times largest fresh-context window MAE plus .000001; fixed RGBA reset/fresh tolerance .000001 from the cut onwards; first-eight-frame no-reset/fresh interior MAE must exceed reset/fresh by .0001',
        claim='Reset clears pre-cut history to the fresh-context result. No prediction that no-reset loses every analytic quality metric.')
    result['known_defect_validation']=criteria_controls(result)
    if not result['known_defect_validation']['passed']: raise ValueError('Calibrated limits accept a known defective image')
    if not verdict(block,result)['passed']: raise ValueError('Exploration does not support the predeclared reset/sensitivity prediction')
    output=Path(output)
    if output.exists(): raise FileExistsError(output)
    save(output,result); return result


def cases(confirm=False,backend='windows'):
    from temporal_quality import cases as quality_cases
    # Matrix generation needs the effective repetition count frozen as a file.
    ns='experiment.temporal-reset'+('' if confirm else '-explore')
    filename='temporal-reset-confirm-protocol.json' if confirm else 'temporal-reset-protocol.json'
    result=quality_cases(dict(namespace=ns,protocol=filename,script='temporal_reset.py',analysis='temporal-reset',backend=backend))
    def controls(r):
        value=ghost_controls(); save(r.directory/'cases'/r.current['id']/'ghost-controls.json',value)
        r.note(controls=value)
        if not value['passed']: raise Rejected('Old-history contamination evaluator is insensitive')
    check=Case(ns+'.ghost-evaluator','Known old-history contamination and severity controls',controls,(ns+'.protocol',),scope='offline-analysis')
    result.insert(1,check)
    for case in result[2:]: case.requires+=(check.id,)
    if confirm:
        def freeze(r):
            source=HERE/'temporal-reset-criteria.json'
            if not source.is_file(): raise Blocked('Select frozen criteria from an independent exploration first')
            value=json.loads(source.read_text())
            if set(value['native_budgets'])!=set(METRICS) or value['pair_maximum']!=1e-6 or value['minimum_history_difference']!=1e-4:
                raise Rejected('Invalid frozen reset criteria')
            if not criteria_controls(value)['passed']: raise Rejected('Frozen criteria accept a known defective image')
            shutil.copyfile(source,r.directory/'cases'/r.current['id']/'criteria.json'); r.note(criteria_sha256=sha(source))
        check=Case(ns+'.criteria','Freeze independently selected quality limits and fixed reset predictions',freeze,(ns+'.protocol',),scope='offline-analysis')
        result.insert(1,check)
        for case in result[2:]: case.requires+=(check.id,)
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__); parser.add_argument('command',choices=['audit','select'])
    parser.add_argument('--run',required=True); parser.add_argument('--output',required=True); args=parser.parse_args()
    if args.command=='select': select(args.run,args.output)
    else: raise SystemExit(0 if audit(args.run,args.output)['passed'] else 1)
