# SPDX-License-Identifier: MPL-2.0
"""Falsifiable evaluator controls and prospective, scoped native confirmation."""
import argparse
import hashlib
import json
from pathlib import Path

from records import save, sha
from temporal_calibration import image, np_module, pool, rows
from temporal_diagnosis import (HERE, arms, color, file_path, measure,
                                frame_plan, rectangle, references, warp)

NAMESPACE='experiment.temporal-quality'


def protocol():
    return json.loads((HERE/'temporal-quality-protocol.json').read_text())


def age_step(age,fg,previous_fg):
    np=np_module()
    age=np.where(~fg,np.minimum(age+1,999),999)
    return np.where(~fg&previous_fg,0,age)


def frame_metrics(output,previous_error,reference,previous_reference,velocity,age):
    """Keep distinct physical properties and the size of each measured region."""
    np=np_module(); point,area,regions,_,_=reference; error=output-point
    results={name:measure(error if name!='edge' else output-area,regions[name])
             for name in ('foreground','background','edge')}
    revealed=regions['background']&(age>=2)&(age<=5)
    results['disocclusion']=measure(error,revealed)
    if previous_error is not None:
        reprojected,valid=warp(previous_error,previous_reference[2]['foreground'],velocity)
        results['temporal']=measure(error-reprojected,regions['foreground']&valid)
    else:
        results['temporal']=measure(error,np.zeros(error.shape[:2],bool))
    values=point[regions['foreground']]; delta=error[regions['foreground']]
    numerator=float(np.square(delta-delta.mean(axis=0)).sum()) if len(delta) else 0
    denominator=float(np.square(values-values.mean(axis=0)).sum()) if len(values) else 0
    results['texture']=dict(pixels=len(values),numerator=numerator,denominator=denominator,
        nrmse=(numerator/denominator)**.5 if denominator>1e-20 else None)
    return results


def evaluate(scores,budgets,pooled=False):
    """No compensation between properties; empty/nonfinite measurements fail."""
    import math
    checks={}
    for name,limit in budgets.items():
        item=scores[name]; value=item['nrmse'] if name=='texture' else item['mae']
        count=item['pixel_frames' if pooled else 'pixels']
        checks[name]=dict(value=value,limit=limit,pixel_frames=count,
            passed=count>0 and value is not None and math.isfinite(value) and value<=limit)
    return dict(passed=all(c['passed'] for c in checks.values()),properties=checks)


def synthetic_controls(measurer=frame_metrics):
    """Known degradations exercise the evaluator, independently of GPU rankings."""
    np=np_module(); spec=protocol(); velocity=[2,1]; frames={}
    age=np.full((360,640),999); previous_fg=np.zeros(age.shape,bool)
    for frame in range(41):
        meta=dict(scene=2,velocity=velocity,frame=frame); ref=references(meta)
        age=age_step(age,ref[4],previous_fg); previous_fg=ref[4]
        if frame in (39,40): frames[frame]=(meta,ref,age.copy())
    def candidate(kind,severity,frame):
        meta,ref,age=frames[frame]; point,area,regions,_,fg=ref; out=area.copy()
        x,y=np.meshgrid((np.arange(640)+.5)/2,(np.arange(360)+.5)/2)
        l,t,_,_=rectangle(meta)
        if kind=='missing-object': out=color(2,False,x,y)
        elif kind=='constant-foreground': out[:]=[.65,.2,.15]
        elif kind=='blur':
            padded=np.pad(area,((4,4),(4,4),(0,0)),mode='edge')
            out=sum(padded[y:y+360,x:x+640] for y in range(9) for x in range(9))/81
        elif kind=='texture-loss':
            mean=point[regions['foreground']].mean(axis=0)
            out[fg]=(1-severity)*area[fg]+severity*mean
        elif kind=='ghost':
            mask=regions['background']&(age>=2)&(age<=5)
            out[mask]=(1-severity)*out[mask]+severity*np.array([.65,.2,.15])
        elif kind=='flicker': out[fg]+=severity*(-1 if frame%2 else 1)
        return out
    planned=[('ideal',0,None),('missing-object',1,'foreground'),
             ('constant-foreground',1,'background'),('blur',1,'edge')]
    planned += [(kind,severity,target) for kind,target,levels in (
        ('texture-loss','texture',(.25,.5,1)),('ghost','disocclusion',(.25,.5,1)),
        ('flicker','temporal',(.01,.02,.04))) for severity in levels]
    result=[]
    for kind,severity,target in planned:
        current=candidate(kind,severity,40); previous=candidate(kind,severity,39)
        scores=measurer(current,previous-frames[39][1][0],frames[40][1],frames[39][1],velocity,frames[40][2])
        verdict=evaluate(scores,spec['synthetic_budgets'])
        expected=verdict['passed'] if target is None else not verdict['passed'] and not verdict['properties'][target]['passed']
        result.append(dict(control=kind,severity=severity,target=target,scores=scores,verdict=verdict,expectation_met=expected))
    monotonic={}
    for kind in ('texture-loss','ghost','flicker'):
        selected=[r for r in result if r['control']==kind]
        values=[r['verdict']['properties'][r['target']]['value'] for r in selected]
        monotonic[kind]=all(a<b for a,b in zip(values,values[1:]))
    return dict(scope='CPU evaluator validation, not AMD output',controls=result,severity_monotonic=monotonic,
                passed=all(r['expectation_met'] for r in result) and all(monotonic.values()))


def pooled_metrics(measurements):
    result={name:pool(item[name] for item in measurements) for name in
            ('foreground','background','edge','temporal','disocclusion')}
    texture=[item['texture'] for item in measurements]
    denominator=sum(t['denominator'] for t in texture); numerator=sum(t['numerator'] for t in texture)
    result['texture']=dict(pixel_frames=sum(t['pixels'] for t in texture),numerator=numerator,
        denominator=denominator,nrmse=(numerator/denominator)**.5 if denominator>1e-20 else None)
    return result


def sensitivity(correct,control,metric,minimum_ratio=1.1):
    import math
    a,b=correct[metric],control[metric]; av,bv=a['mae'],b['mae']
    populated=a['pixel_frames']>0 and b['pixel_frames']>0
    finite=av is not None and bv is not None and math.isfinite(av) and math.isfinite(bv)
    return dict(correct=av,control=bv,minimum_ratio=minimum_ratio,
                ratio=bv/av if finite and av>0 else None,
                passed=populated and finite and av>0 and bv>=minimum_ratio*av)


def collect_metrics(run,spec,namespace):
    """Collect frame-labelled scores after complete independent native audits."""
    np=np_module(); run=Path(run); planned=arms(spec); full,_=frame_plan(spec)
    repeats=range(1,spec['repeats']+1)
    directories={i:run/'readbacks'/f'{namespace}.r{i}' for i in repeats}
    # Every child has already independently checked all input bytes and decoded
    # every output. Require those successful prerequisites, without treating
    # the native helper's metrics as a quality oracle.
    for i in repeats:
        record=json.loads((run/'cases'/f'{namespace}.r{i}'/'result.json').read_text())
        if record['evaluation']!='passed' or record['observations']['input_audit']['frames']!=len(planned)*len(full):
            raise ValueError('Missing complete independent input/output audit')
    data={(i,a['name']):[] for i in repeats for a in planned}; identities={}; same={i:True for i in repeats if i!=1}
    for scene in spec['scenes']:
        group=[a for a in planned if a['group']==scene['name']]; prior={}
        age=np.full((360,640),999); previous_ref=None
        for frame in full:
            meta=dict(scene=scene['scene'],velocity=scene['velocity'],frame=frame); ref=references(meta)
            if previous_ref is not None: age=age_step(age,ref[4],previous_ref[4])
            if frame>=31:
                for i in repeats:
                    for a in group:
                        key=i,a['name']; path=file_path(directories[i],dict(arm=a['name'],frame=frame),'rgba16f')
                        u=image(path)[...,:3]
                        digest=hashlib.sha256(path.read_bytes()).hexdigest(); identity=a['name'],frame
                        if i==1: identities[identity]=digest
                        else: same[i] &= identities[identity]==digest
                        if frame>=32:
                            scores=frame_metrics(u,prior[key],ref,previous_ref,scene['velocity'],age)
                            data[key].append(dict(frame=frame,scores=scores))
                        prior[key]=u-ref[0]
            previous_ref=ref
        print('Evaluated '+scene['name'],flush=True)
    return data,same


def audit(run,output):
    np=np_module(); output=Path(output); output.mkdir(parents=True,exist_ok=False)
    spec=protocol(); planned=arms(spec); repeats=range(1,spec['repeats']+1)
    result=dict(protocol_sha256=sha(HERE/'temporal-quality-protocol.json'),numpy=np.__version__,
                evaluator_validation=synthetic_controls(),blocks=[],interactions=[],
                scope='Synthetic evaluator validation and prospective textured-slow local regression/sensitivity checks')
    data,same=collect_metrics(run,spec,NAMESPACE)
    for i in repeats:
        values={a['name']:pooled_metrics([f['scores'] for f in data[i,a['name']]]) for a in planned}
        correct=values['textured-slow-correct']
        budget=evaluate(correct,spec['native_budgets'],pooled=True)
        checks={name:sensitivity(correct,values['textured-slow-'+control],metric)
                for name,control,metric in (('motion_detail','reverse','foreground'),
                    ('motion_temporal','reverse','temporal'),('history_temporal','reset-all','temporal'))}
        block=dict(repeat=i,arms=values,budget=budget,sensitivity=checks,
            selected_outputs_identical_repeat1=same.get(i),passed=budget['passed'] and all(c['passed'] for c in checks.values()))
        result['blocks'].append(block)
        effects={}
        for texture in ('flat','textured'):
            for speed in ('slow','fast'):
                group=texture+'-'+speed
                effects[group]={metric:values[group+'-reverse'][metric]['mae']-values[group+'-correct'][metric]['mae']
                               for metric in spec['native_budgets']}
        interactions={metric:effects['textured-fast'][metric]-effects['textured-slow'][metric]
                      -effects['flat-fast'][metric]+effects['flat-slow'][metric] for metric in spec['native_budgets']}
        result['interactions'].append(dict(repeat=i,reverse_minus_correct=effects,
            texture_speed_motion_interaction=interactions,acceptance='diagnostic only'))
    result.update(output_frames=len(planned)*spec['frames']*spec['repeats'],
        passed=result['evaluator_validation']['passed'] and all(b['passed'] for b in result['blocks']),
        statistical_claim='No p-values or population confidence intervals; frames are correlated and blocks use one equipment configuration.')
    save(output/'summary.json',result)
    return result


def cases(configuration=None):
    from execution import Case, Rejected
    from temporal_diagnosis import cases as matrix_cases
    def evaluator(r):
        value=synthetic_controls(); save(r.directory/'cases'/r.current['id']/'evaluator.json',value)
        r.note(controls=len(value['controls']),passed=value['passed'])
        if not value['passed']: raise Rejected('Evaluator failed a known-defect or severity control')
    configuration=configuration or dict(namespace=NAMESPACE,protocol='temporal-quality-protocol.json',
        script='temporal_quality.py',analysis='temporal-quality')
    ns=configuration['namespace']; result=matrix_cases(configuration)
    check=Case(ns+'.evaluator','Known-defect and severity validation before GPU execution',evaluator,
               (ns+'.protocol',),scope='offline-analysis')
    result.insert(1,check)
    for case in result[2:]: case.requires+=(check.id,)
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=['audit','controls']); parser.add_argument('--run'); parser.add_argument('--output',required=True)
    args=parser.parse_args()
    if args.command=='audit':
        if not args.run: parser.error('--run is required for audit')
        result=audit(args.run,args.output)
    else:
        result=synthetic_controls(); save(Path(args.output),result)
    raise SystemExit(0 if result['passed'] else 1)
