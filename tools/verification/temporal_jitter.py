# SPDX-License-Identifier: MPL-2.0
"""Validated spatial phase measurement and prospective jitter confirmation."""
import argparse
import hashlib
import json
import math
from pathlib import Path
import shutil

from execution import Blocked, Case, Rejected
from records import save, sha
from temporal_calibration import np_module, payload
from temporal_diagnosis import HERE, arms, file_path, frame_plan, rectangle, references
from temporal_quality import frame_metrics, pooled_metrics, synthetic_controls

ALIGNMENT=('x_rms','y_rms','radial_rms')
QUALITY=('foreground','background','edge','temporal','texture')
CONTROLS={'negated-x':'x_rms','negated-y':'y_rms','negated-jitter':'radial_rms',
          'lagged-jitter':'radial_rms','zero-jitter':'radial_rms'}


def protocol(confirm=False):
    filename='temporal-jitter'+('-confirm' if confirm else '')+'-protocol.json'
    return json.loads((HERE/filename).read_text())


class PositionModel:
    """Fit a known texture's phase; never align an image before quality scoring."""
    def __init__(self,meta,region):
        np=np_module(); self.region=region
        x,y=np.meshgrid((np.arange(0,640,4)+.5)/2,(np.arange(0,360,4)+.5)/2)
        l,t,r,b=rectangle(meta); fg=(x>=l)&(x<r)&(y>=t)&(y<b)
        inside=np.minimum.reduce([x-l,r-x,y-t,b-y])
        outside=np.hypot(np.maximum(np.maximum(l-x,x-r),0),np.maximum(np.maximum(t-y,y-b),0))
        valid=(x>=8)&(x<312)&(y>=8)&(y<172)
        self.mask=valid&((fg&(inside>8)) if region=='foreground' else (~fg&(outside>8)))
        if int(self.mask.sum())<100: raise ValueError('Insufficient identifiable spatial samples')
        self.parameters=((.65,.15,.4,0),(.2,.1,0,.3),(.15,.08,.23,.19)) if region=='foreground' else (
            (.25,.1,.35,0),(.3,.1,0,.27),(.35,.1,.21,.17))
        if region=='foreground': x,y=x-l,y-t
        self.design=[]; self.inverse=[]
        for _,_,ax,ay in self.parameters:
            phase=(ax*x+ay*y)[self.mask]
            design=np.stack((np.ones_like(phase),np.sin(phase),np.cos(phase)),axis=1)
            gram=np.einsum('ni,nj->ij',design,design)
            if np.linalg.cond(gram)>100: raise ValueError('Spatial phase is not identifiable')
            self.design.append(design)
            self.inverse.append(np.einsum('ij,nj->in',np.linalg.inv(gram),design))

    def measure(self,output):
        np=np_module(); values=np.asarray(output)[::4,::4,:3][self.mask]
        if not np.isfinite(values).all(): raise ValueError('Nonfinite position samples')
        fits=[]
        for channel,(base,amplitude,ax,ay) in enumerate(self.parameters):
            value=values[:,channel]; coefficients=np.einsum('ij,j->i',self.inverse[channel],value)
            dc,sine,cosine=map(float,coefficients)
            gain=math.hypot(sine,cosine)/amplitude; phase=math.atan2(cosine,sine)
            residual=value-np.einsum('ij,j->i',self.design[channel],coefficients)
            fits.append(dict(phase=phase,gain=gain,bias=dc-base,rmse=float(np.sqrt(np.square(residual).mean()))))
        dx=fits[0]['phase']/self.parameters[0][2]; dy=fits[1]['phase']/self.parameters[1][3]
        blue=fits[2]['phase']-self.parameters[2][2]*dx-self.parameters[2][3]*dy
        blue=math.atan2(math.sin(blue),math.cos(blue))
        # Half-precision readbacks perturb a shift exactly at the domain edge.
        # The .002-pixel numerical tolerance is also required by CPU controls.
        valid=abs(dx)<=1.002 and abs(dy)<=1.002 and all(.25<=f['gain']<=2 for f in fits)
        return dict(valid=valid,pixels=len(values),x=dx,y=dy,radial=math.hypot(dx,dy),
            blue_consistency=abs(blue)/math.hypot(*self.parameters[2][2:]),channels=fits)


def position_controls(estimator=None,model_factory=PositionModel):
    """Known analytic translations are independent of the fitted coefficients."""
    np=np_module(); x,y=np.meshgrid((np.arange(640)+.5)/2,(np.arange(360)+.5)/2)
    meta=dict(scene=2,frame=40,velocity=[.375,.25]); l,t,_,_=rectangle(meta); results=[]
    for region in ('foreground','background'):
        model=model_factory(meta,region)
        def candidate(dx,dy,bias=0,gain=1):
            if region=='foreground':
                out=np.stack((.65+.15*np.sin(.4*(x-l+dx)),.2+.1*np.sin(.3*(y-t+dy)),
                    .15+.08*np.sin(.23*(x-l+dx)+.19*(y-t+dy))),axis=-1)
                base=np.array([.65,.2,.15])
            else:
                out=np.stack((.25+.1*np.sin(.35*(x+dx)),.3+.1*np.sin(.27*(y+dy)),
                    .35+.1*np.sin(.21*(x+dx)+.17*(y+dy))),axis=-1)
                base=np.array([.25,.3,.35])
            return (out-base)*gain+base+bias
        measure=estimator or model.measure
        planned=[('ideal',0,0,0,1),('brightness',0,0,.03,1),('contrast',0,0,0,.5)]
        planned.extend(('axis-shift',dx,dy,0,1) for dx,dy in (
            (.125,0),(-.125,0),(0,.125),(0,-.125),(.5,0),(0,-.5),(.125,.375),(-.375,.125)))
        for size in (.125,.25,.5,1):
            planned.extend((('shift',size,-size,0,1),('shift',-size,size,-.02,1.5)))
        for name,dx,dy,bias,gain in planned:
            out=candidate(dx,dy,bias,gain)
            # Actual native readback format, without sharing the fitter's basis.
            measured=measure(out.astype('<f2').astype(float))
            passed=measured['valid'] and abs(measured['x']-dx)<.002 and abs(measured['y']-dy)<.002
            if estimator is None:
                passed &= all(abs(f['gain']-gain)<.002 and abs(f['bias']-bias)<.002 for f in measured['channels'])
                passed &= measured['blue_consistency']<.002
            results.append(dict(region=region,kind=name,expected=[dx,dy],bias=bias,gain=gain,
                measured=measured,passed=bool(passed)))
        for name,out in (('constant',np.full((360,640,3),.3)),('out-of-range',candidate(2,0))):
            measured=measure(out); results.append(dict(region=region,kind=name,measured=measured,passed=not measured['valid']))
        try:
            measure(np.full((360,640,3),np.nan)); rejected=False
        except ValueError:
            rejected=True
        results.append(dict(region=region,kind='nonfinite',passed=rejected))
    return dict(scope='CPU known translations, brightness/contrast and unidentifiable controls; render-pixel units',
        controls=results,passed=all(c['passed'] for c in results))


def pool_position(values):
    if not values or not all(v['valid'] and v['pixels']>0 for v in values):
        raise ValueError('Unidentifiable or missing spatial phase')
    result={axis+'_rms':math.sqrt(sum(v[axis]**2 for v in values)/len(values)) for axis in ('x','y','radial')}
    result.update(x_mean=sum(v['x'] for v in values)/len(values),y_mean=sum(v['y'] for v in values)/len(values),
        blue_consistency_rms=math.sqrt(sum(v['blue_consistency']**2 for v in values)/len(values)),
        minimum_gain=min(c['gain'] for v in values for c in v['channels']),frames=len(values))
    return result


def criteria_controls(criteria):
    """A generous pilot may not make a known translation or damaged image pass."""
    known=synthetic_controls()['controls']; checks={}
    for group,limits in criteria['groups'].items():
        for axis,limit in limits['alignment'].items():
            checks[group+'.'+axis]=math.isfinite(limit) and 0<limit<.125
        for metric,kind,severity in (('foreground','missing-object',1),('background','constant-foreground',1),
                ('edge','blur',1),('temporal','flicker',.01),('texture','texture-loss',.25)):
            item=next(c for c in known if c['control']==kind and c['severity']==severity)['scores'][metric]
            defect=item['nrmse' if metric=='texture' else 'mae']; limit=limits['quality'][metric]
            checks[group+'.'+metric]=math.isfinite(limit) and 0<limit<defect
    return dict(passed=all(checks.values()),checks=checks)


def verdict(block,spec,criteria):
    checks=[]
    for scene in spec['scenes']:
        limits=criteria['groups'][scene['name']]
        for phase in scene['jitter_phases']:
            group=f'{scene["name"]}-p{phase:02}'; region='background' if scene['name']=='stationary' else 'foreground'
            for start,end in spec['windows']:
                values=block['windows'][f'{group}:{start}-{end}']; correct=values['correct']
                position=correct['position'][region]
                alignment=all(position[key]<=limit for key,limit in limits['alignment'].items())
                quality=all(correct['quality'][key]['pixel_frames']>0 and math.isfinite(correct['quality'][key]['nrmse' if key=='texture' else 'mae'])
                    and correct['quality'][key]['nrmse' if key=='texture' else 'mae']<=limit for key,limit in limits['quality'].items())
                sensitivity={control:values[control]['position'][region][metric]>=1.1*position[metric]+.001
                    for control,metric in CONTROLS.items()}
                checks.append(dict(group=group,frames=[start,end],alignment=alignment,quality=quality,
                    sensitivity=sensitivity,passed=alignment and quality and all(sensitivity.values())))
    return dict(checks=checks,passed=all(c['passed'] for c in checks))


def audit(run,output):
    np=np_module(); run=Path(run); output=Path(output); output.mkdir(parents=True,exist_ok=False)
    ns=json.loads((run/'run.json').read_text())['profile'].replace('proton-temporal-','experiment.temporal-',1).replace('experiment-','experiment.',1)
    spec=json.loads((run/'cases'/f'{ns}.protocol/protocol.json').read_text()); frame_plan(spec)
    planned=arms(spec); blocks=[]; hashes={}; trajectories={}; age=np.full((360,640),999)
    for repeat in range(1,spec['repeats']+1):
        directory=run/'readbacks'/f'{ns}.r{repeat}'
        state=json.loads((run/'cases'/f'{ns}.r{repeat}/result.json').read_text())
        if state['evaluation']!='passed' or state['observations']['input_audit']['frames']!=len(planned)*128:
            raise ValueError('Missing independent complete input/output audit')
        windows={}; paired=0; same=True; trajectories={}
        for scene in spec['scenes']:
            for phase in scene['jitter_phases']:
                group=f'{scene["name"]}-p{phase:02}'; selected=[a for a in planned if a['group']==group]
                values={a['name']:[] for a in selected}; previous={}; previous_ref=None; models=None
                static_ref=references(dict(scene=2,frame=31,velocity=[0,0])) if scene['velocity']==[0,0] else None
                for frame in range(31,128):
                    meta=dict(scene=2,frame=frame,velocity=scene['velocity'])
                    ref=static_ref if static_ref is not None else references(meta)
                    if models is None or scene['velocity']!=[0,0]:
                        models={region:PositionModel(meta,region) for region in ('foreground','background')}
                    common=None
                    for arm in selected:
                        key=dict(arm=arm['name'],frame=frame)
                        inputs=payload(file_path(directory,key,'inputs'))
                        if common is None: common=inputs
                        elif common!=inputs: raise ValueError('Jitter controls changed rendered input bytes')
                        paired+=1
                        raw=payload(file_path(directory,key,'rgba16f')); identity=hashlib.sha256(raw).hexdigest()
                        if repeat==1: hashes[arm['name'],frame]=identity
                        else: same &= hashes[arm['name'],frame]==identity
                        u=np.frombuffer(raw,dtype='<f2').reshape(360,640,4).astype(float)[...,:3]
                        if frame>=32:
                            scores=frame_metrics(u,previous[arm['name']],ref,previous_ref,scene['velocity'],age)
                            position={region:model.measure(u) for region,model in models.items()}
                            values[arm['name']].append(dict(frame=frame,position=position,scores=scores))
                        previous[arm['name']]=u-ref[0]
                    previous_ref=ref
                for start,end in spec['windows']:
                    window={}
                    for arm in selected:
                        items=[v for v in values[arm['name']] if start<=v['frame']<=end]
                        if len(items)!=32: raise ValueError('Incomplete alignment window')
                        window[arm['name'][len(group)+1:]]=dict(position={region:pool_position([v['position'][region] for v in items])
                            for region in ('foreground','background')},quality=pooled_metrics([v['scores'] for v in items]))
                    windows[f'{group}:{start}-{end}']=window
                trajectories.update({name:[dict(frame=v['frame'],position=v['position']) for v in items] for name,items in values.items()})
                print(f'Evaluated jitter r{repeat} {group}',flush=True)
        save(output/f'r{repeat}-trajectories.json',trajectories)
        blocks.append(dict(repeat=repeat,windows=windows,paired_input_checks=paired,selected_outputs_equal_repeat1=same if repeat>1 else None))
    result=dict(protocol=spec,protocol_sha256=sha(run/'cases'/f'{ns}.protocol/protocol.json'),numpy=np.__version__,
        position_controls=position_controls(),blocks=blocks,output_frames=len(planned)*128*spec['repeats'])
    if spec['confirmation']:
        criteria=json.loads((run/'cases'/f'{ns}.criteria/criteria.json').read_text()); result['criteria']=criteria
        result['criteria_controls']=criteria_controls(criteria)
        for block in blocks: block['verdict']=verdict(block,spec,criteria)
        result['passed']=result['position_controls']['passed'] and result['criteria_controls']['passed'] and all(b['verdict']['passed'] for b in blocks)
    else:
        result.update(passed=result['position_controls']['passed'],acceptance='Exploration only; no native jitter acceptance')
    save(output/'summary.json',result); return result


def select(run,output):
    from run import verify_run
    run=Path(run); state=verify_run(run)
    if state['status']!='passed' or state['profile']!='experiment-temporal-jitter-explore': raise ValueError('Completed exploration required')
    source=run/'analysis/temporal-jitter/summary.json'; values=json.loads(source.read_text()); spec=values['protocol']
    if spec['confirmation'] or len(values['blocks'])!=1: raise ValueError('Independent single exploration required')
    block=values['blocks'][0]; groups={}
    for scene in spec['scenes']:
        selected=[v['correct'] for name,v in block['windows'].items() if name.startswith(scene['name']+'-')]
        region='background' if scene['name']=='stationary' else 'foreground'
        groups[scene['name']]=dict(alignment={key:1.25*max(v['position'][region][key] for v in selected)+1e-6 for key in ALIGNMENT},
            quality={key:1.25*max(v['quality'][key]['nrmse' if key=='texture' else 'mae'] for v in selected)+1e-6 for key in QUALITY})
    result=dict(schema_version=1,exploration_input_identity=state['input_identity'],exploration_summary_sha256=sha(source),groups=groups,
        rule=spec['selection_rule'],scope='Bounded spatial-phase and local quality/sensitivity confirmation; no universal quality or jitter policy')
    result['known_defect_validation']=criteria_controls(result)
    if not result['known_defect_validation']['passed']: raise ValueError('Calibration accepts a known defect')
    if not verdict(block,spec,result)['passed']: raise ValueError('Pilot does not support predeclared per-window alignment/sensitivity predictions')
    output=Path(output)
    if output.exists(): raise FileExistsError(output)
    save(output,result); return result


def cases(confirm=False,backend='windows'):
    from temporal_quality import cases as quality_cases
    ns='experiment.temporal-jitter'+('' if confirm else '-explore')
    filename='temporal-jitter'+('-confirm' if confirm else '')+'-protocol.json'
    if backend=='proton':
        filename='proton-temporal-jitter-protocol.json'
    result=quality_cases(dict(namespace=ns,protocol=filename,script='temporal_jitter.py',analysis='temporal-jitter',
        pack_cache=True,analysis_timeout=1800,backend=backend))
    def evaluator(r):
        value=position_controls(); save(r.directory/'cases'/r.current['id']/'position-controls.json',value)
        r.note(passed=value['passed'],controls=len(value['controls']))
        if not value['passed']: raise Rejected('Positions cannot be distinguished from brightness/contrast')
    check=Case(ns+'.position-evaluator','Known spatial phase, bias, contrast and unidentifiable controls',evaluator,(ns+'.protocol',),scope='offline-analysis')
    result.insert(1,check)
    for case in result[2:]: case.requires+=(check.id,)
    if confirm:
        def freeze(r):
            source=HERE/'temporal-jitter-criteria.json'
            if not source.is_file(): raise Blocked('Freeze independently selected jitter criteria first')
            value=json.loads(source.read_text())
            if set(value['groups'])!={'stationary','moving'} or any(set(v['alignment'])!=set(ALIGNMENT) or set(v['quality'])!=set(QUALITY) for v in value['groups'].values()):
                raise Rejected('Invalid frozen jitter criteria')
            if not criteria_controls(value)['passed']: raise Rejected('Criteria accept known defects')
            shutil.copyfile(source,r.directory/'cases'/r.current['id']/'criteria.json'); r.note(criteria_sha256=sha(source))
        check=Case(ns+'.criteria','Freeze independent alignment and quality limits',freeze,(ns+'.protocol',),scope='offline-analysis')
        result.insert(1,check)
        for case in result[2:]: case.requires+=(check.id,)
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__); parser.add_argument('command',choices=['controls','audit','select'])
    parser.add_argument('--run'); parser.add_argument('--output',required=True); args=parser.parse_args()
    if args.command=='controls': value=position_controls(); save(Path(args.output),value)
    else:
        if not args.run: parser.error('--run required')
        value=select(args.run,args.output) if args.command=='select' else audit(args.run,args.output)
    raise SystemExit(0 if value.get('passed',True) else 1)
