# SPDX-License-Identifier: MPL-2.0
"""Fixed native matrix and independent metric counterexamples; no quality gate."""
import argparse
from functools import lru_cache
import hashlib
import json
from pathlib import Path
import shutil
import struct
import sys

from execution import Blocked, Case, Rejected
from records import save, sha
from temporal_calibration import (depth, f32, image, jitter_table, np_module,
                                  pack, payload, pool, rows, truncate_half)

HERE = Path(__file__).resolve().parent
NAMESPACE = 'experiment.temporal-diagnosis'


def protocol():
    return json.loads((HERE/'temporal-diagnosis-protocol.json').read_text())


def arms(spec=None):
    result=[]; spec=spec or protocol()
    for scene in spec['scenes']:
        if 'jitter_phases' in scene:
            factors={'correct':(1,1),'negated-x':(-1,1),'negated-y':(1,-1),
                     'negated-jitter':(-1,-1),'zero-jitter':(0,0),'lagged-jitter':(1,1)}
            for phase in scene['jitter_phases']:
                if type(phase) is not int or not 0<=phase<32: raise ValueError('Invalid jitter start phase')
                group=f'{scene["name"]}-p{phase:02}'
                for control in scene['controls']:
                    sx,sy=factors[control]
                    result.append(dict(name=group+'-'+control,group=group,scene=scene['scene'],
                        velocity=scene['velocity'],motion=0,jitter=1,reset_all=False,
                        jitter_phase=phase,jitter_x=sx,jitter_y=sy,dispatch_lag=int(control=='lagged-jitter')))
            continue
        for control in scene['controls']:
            result.append(dict(name=scene['name']+'-'+control,group=scene['name'],scene=scene['scene'],
                velocity=scene['velocity'],motion={'reverse':1,'zero-motion':2}.get(control,0),
                jitter={'negated-jitter':-1,'zero-jitter':0}.get(control,1),reset_all=control=='reset-all'))
            if 'cut_frame' in spec:
                result[-1].update(start=spec['cut_frame'] if control=='fresh' else 0,
                    reset_frame=spec['cut_frame'] if control in ('reset','fresh') else -1)
    return result


def rectangle(meta):
    vx,vy=meta['velocity']; frame=meta['frame']
    cut=frame>=meta.get('cut_frame',999)
    return 100+vx*frame-96*cut,50+vy*frame+48*cut,164+vx*frame-96*cut,94+vy*frame+48*cut


def color(scene, foreground, x, y, left=0, top=0):
    np=np_module()
    x,y=np.broadcast_arrays(x,y)
    if scene==0:
        return np.stack((.125+x/1024,.25+y/1024,.375+(x+y)/2048),axis=-1)
    if scene==1:
        return np.broadcast_to([.875,.125,.0625] if foreground else [.125,.25,.375],x.shape+(3,)).copy()
    if foreground:
        u,v=x-left,y-top
        return np.stack((.65+.15*np.sin(.4*u),.2+.1*np.sin(.3*v),.15+.08*np.sin(.23*u+.19*v)),axis=-1)
    return np.stack((.25+.1*np.sin(.35*x),.3+.1*np.sin(.27*y),.35+.1*np.sin(.21*x+.17*y)),axis=-1)


def integral_color(scene,foreground,x0,x1,y0,y1,left=0,top=0):
    """Exact rectangle integral, with unnormalized empty-footprint handling."""
    np=np_module(); w=np.maximum(x1-x0,0); h=np.maximum(y1-y0,0)
    cx,cy=(x0+x1)/2,(y0+y1)/2
    if scene!=2:
        return color(scene,foreground,cx,cy,left,top)*(w*h)[...,None]
    if foreground:
        cx,cy=cx-left,cy-top
        parameters=((.65,.15,.4,0),(.2,.1,0,.3),(.15,.08,.23,.19))
    else:
        parameters=((.25,.1,.35,0),(.3,.1,0,.27),(.35,.1,.21,.17))
    return np.stack([(base+amplitude*np.sin(ax*cx+ay*cy)*np.sinc(ax*w/(2*np.pi))*np.sinc(ay*h/(2*np.pi)))*w*h
                     for base,amplitude,ax,ay in parameters],axis=-1)


def references(meta,shift=(0,0)):
    np=np_module(); x,y=np.meshgrid((np.arange(640)+.5)/2+shift[0],(np.arange(360)+.5)/2+shift[1])
    valid=(x>=4)&(x<316)&(y>=4)&(y<176)
    scene=meta['scene']; l,t,r,b=rectangle(meta)
    bx,by=(24,-12) if meta['frame']>=meta.get('cut_frame',999) else (0,0)
    fg=((x>=l)&(x<r)&(y>=t)&(y<b)) if scene else np.zeros(x.shape,bool)
    point=np.where(fg[...,None],color(scene,True,x,y,l,t),color(scene,False,x+bx,y+by))
    if scene:
        outside=np.hypot(np.maximum(np.maximum(l-x,x-r),0),np.maximum(np.maximum(t-y,y-b),0))
        inside=np.minimum.reduce([x-l,r-x,y-t,b-y])
        distance=2*np.where(fg,inside,outside)
        x0,x1=np.maximum(x-.25,l),np.minimum(x+.25,r)
        y0,y1=np.maximum(y-.25,t),np.minimum(y+.25,b)
        full=integral_color(scene,False,x-.25+bx,x+.25+bx,y-.25+by,y+.25+by)
        covered_bg=integral_color(scene,False,x0+bx,x1+bx,y0+by,y1+by)
        covered_fg=integral_color(scene,True,x0,x1,y0,y1,l,t)
        area=4*(full-covered_bg+covered_fg)
    else:
        distance=np.full(x.shape,np.inf); area=point.copy()
    regions=dict(interior=valid&(distance>2),foreground=valid&fg&(distance>2),
                 background=valid&~fg&(distance>2),edge=valid&(distance<=2))
    return point,area,regions,distance,fg


def expected_inputs(meta):
    np=np_module(); x,y=np.meshgrid(np.arange(320)+.5,np.arange(180)+.5)
    jx,jy=jitter_table()[(meta['frame']+meta.get('jitter_phase',0))%32]; x,y=x-jx,y-jy
    l,t,r,b=rectangle(meta); scene=meta['scene']
    bx,by=(24,-12) if meta['frame']>=meta.get('cut_frame',999) else (0,0)
    fg=((x>=l)&(x<r)&(y>=t)&(y<b)) if scene else np.zeros(x.shape,bool)
    rgba=np.ones((180,320,4)); rgba[...,:3]=np.where(fg[...,None],color(scene,True,x,y,l,t),color(scene,False,x+bx,y+by))
    z=np.where(fg,depth(2),depth(8)).astype('<f4')
    mv=np.zeros((180,320,2)); vx,vy=meta['velocity']
    factor=(1,-1,0)[meta['motion']]
    if meta['frame'] and meta['motion']!=2: mv[fg]=[-vx*factor/320,-vy*factor/180]
    if meta['frame']==meta.get('cut_frame'):
        mv[:]=[24/320,-12/180]; mv[fg]=[(96-vx)/320,(-48-vy)/180]
    mv[mv==0]=0.0  # Canonical positive zero, including stationary foreground.
    return b''.join((truncate_half(rgba).tobytes(),z.tobytes(),truncate_half(mv).tobytes(),struct.pack('<f',1)))


def input_oracle_cache():
    """Bounded reuse of analytic bytes; every actual payload is still compared."""
    @lru_cache(maxsize=192)
    def generate(scene,velocity,motion,frame,phase,cut):
        meta=dict(scene=scene,velocity=velocity,motion=motion,frame=frame,jitter_phase=(phase-frame)%32)
        if cut is not None: meta['cut_frame']=cut
        return expected_inputs(meta)
    def oracle(meta):
        velocity=tuple(meta['velocity']); frame=meta['frame']
        # At zero velocity and without cuts, canonical zero motion makes
        # frame zero and later frames physically identical at the same phase.
        physical_frame=0 if velocity==(0,0) and 'cut_frame' not in meta else frame
        return generate(meta['scene'],velocity,meta['motion'],physical_frame,
            (frame+meta.get('jitter_phase',0))%32,meta.get('cut_frame'))
    return oracle


def file_path(directory,meta,suffix):
    return Path(directory)/f'{meta["arm"]}-{meta["frame"]:03}.{suffix}.gz'


def frame_plan(spec=None):
    """Bound resource retention; sample both sides of every jitter cycle."""
    spec=spec or protocol(); count=spec['frames']
    if type(count) is not int or not 64<=count<=256 or count%32:
        raise ValueError('Frame count must be a multiple of 32 between 64 and 256')
    inputs=[f for f in range(count) if f in (0,1) or f%32 in (0,31)]
    phases={phase for scene in spec['scenes'] for phase in scene.get('jitter_phases',[])}
    if phases:
        inputs=sorted(set(inputs+[f for f in range(count) if any((f+phase)%32 in (0,31) for phase in phases)]))
    if 'cut_frame' in spec:
        if count!=128 or spec['cut_frame']!=64: raise ValueError('Reset protocol requires 128 frames and cut 64')
        inputs=sorted(set(inputs+[spec['cut_frame']+1]))
    if spec['input_only_frames']!=inputs:
        raise ValueError('Input-only frame plan must cover every cycle boundary')
    return range(count),inputs


def validate(directory,input_only=False,planned_arms=None,spec=None,proton=False):
    lookup={a['name']:a for a in (planned_arms or arms())}; seen=set(); checked=0
    full,inputs=frame_plan(spec); frames=inputs if input_only else full
    digest=hashlib.sha256()
    oracle_function=input_oracle_cache() if any('jitter_phase' in a for a in lookup.values()) else expected_inputs
    for meta in rows(directory):
        arm=lookup[meta['arm']]; frame=meta['frame']; key=meta['arm'],frame
        if key in seen or frame not in frames or frame<arm.get('start',0): raise ValueError('Duplicate/unplanned frame')
        seen.add(key); j=jitter_table()[(frame+arm.get('jitter_phase',0))%32]
        dispatch=jitter_table()[(frame+arm.get('jitter_phase',0)-arm.get('dispatch_lag',0))%32]
        expected=dict(scene=arm['scene'],velocity=arm['velocity'],motion=arm['motion'],
                      image_jitter=j,dispatch_jitter=[f32(v*arm.get('jitter_'+axis,arm['jitter'])) for v,axis in zip(dispatch,('x','y'))],reset=frame==0 or arm['reset_all'] or frame==arm.get('reset_frame',-1),
                      finish_hr=0,fence=1,debug_errors=0,device_removed=0,finite_pixels=640*360,
                      dispatch='not_run' if input_only else 'ok')
        if spec and 'cut_frame' in spec: expected['cut_frame']=spec['cut_frame']
        if 'jitter_phase' in arm: expected.update(jitter_phase=arm['jitter_phase'],dispatch_lag=arm['dispatch_lag'])
        for field,value in expected.items():
            actual=meta[field]
            if field=='debug_errors' and proton and actual==2**64-1:
                continue  # Unavailable Windows diagnostic, never zero errors.
            if field in ('image_jitter','dispatch_jitter'): actual=[f32(v) for v in actual]
            if actual!=value: raise ValueError(f'Metadata mismatch {key}: {field}')
        actual=payload(file_path(directory,meta,'inputs')); oracle=oracle_function(meta)
        if actual!=oracle:
            first=next((i for i,(a,b) in enumerate(zip(actual,oracle)) if a!=b),None)
            detail=f' actual={actual[first]} expected={oracle[first]}' if first is not None else ''
            raise ValueError(f'Input mismatch {key} bytes={len(actual)} first={first}{detail}')
        digest.update(hashlib.sha256(actual).digest()); checked+=len(actual)
        if not input_only:
            if meta['changed_pixels']!=640*360: raise ValueError('Unwritten output')
            image(file_path(directory,meta,'rgba16f'))
    planned={(name,frame) for name,arm in lookup.items() for frame in frames if frame>=arm.get('start',0)}
    if seen!=planned: raise ValueError('Incomplete input/frame inventory')
    return dict(status='passed',frames=len(seen),complete_input_bytes_checked=checked,input_digest=digest.hexdigest())


def measure(values,mask):
    np=np_module(); a=values[mask]; n=int(mask.sum())
    if not n: return dict(pixels=0,mae=None,rmse=None,signed_mean=None)
    return dict(pixels=n,mae=float(np.abs(a).mean()),rmse=float(np.sqrt(np.square(a).mean())),signed_mean=float(a.mean()))


def warp(values,mask,velocity):
    np=np_module(); vx,vy=velocity
    x,y=np.meshgrid(np.arange(640)-2*vx,np.arange(360)-2*vy)
    ix,iy=np.floor(x).astype(int),np.floor(y).astype(int)
    valid=(ix>=0)&(ix<639)&(iy>=0)&(iy<359)
    ix=np.clip(ix,0,638); iy=np.clip(iy,0,358)
    fx,fy=(x-ix)[...,None],(y-iy)[...,None]
    u=(values[iy,ix]*(1-fx)+values[iy,ix+1]*fx)*(1-fy)+(values[iy+1,ix]*(1-fx)+values[iy+1,ix+1]*fx)*fy
    valid &= mask[iy,ix]&mask[iy,ix+1]&mask[iy+1,ix]&mask[iy+1,ix+1]
    return u,valid


def analyze_arm(directory,arm):
    np=np_module(); metadata=[m for m in rows(directory) if m['arm']==arm['name']]
    permanent=references(metadata[0])[2]['background'].copy()
    for m in metadata: permanent &= references(m)[2]['background']
    age=np.full((360,640),999,dtype=int); previous=None; measurements=[]
    for meta in metadata:
        point,area,regions,distance,fg=references(meta)
        u=image(file_path(directory,meta,'rgba16f'))[...,:3]; error=u-point
        regions['permanent_background']=permanent
        valid=regions['interior']|regions['edge']
        if previous:
            age=np.where(~fg,np.minimum(age+1,999),999)
            age=np.where(~fg&previous['fg'],0,age)
        for name,lo,hi in (('age0',0,0),('age1',1,1),('age2to5',2,5)):
            base=valid&~fg&(age>=lo)&(age<=hi)
            regions[name]=base; regions[name+'_pure']=base&(distance>2)
        item=dict(frame=meta['frame'],point={},area={},temporal={},leakage={},shift={})
        for name,mask in regions.items():
            item['point'][name]=measure(error,mask); item['area'][name]=measure(u-area,mask)
        if previous:
            item['temporal']['background']=measure(error-previous['error'],regions['background']&previous['background'])
            reprojected,valid_fg=warp(previous['error'],previous['foreground'],arm['velocity'])
            item['temporal']['foreground']=measure(error-reprojected,regions['foreground']&valid_fg)
        if meta['scene']==1:
            contrast=np.array([.75,-.125,-.3125]); g=np.sum(error*contrast,axis=2)/np.dot(contrast,contrast)
            for name in ('background','age0_pure','age1_pure','age2to5_pure'):
                item['leakage'][name]=measure(g,regions[name])
        if meta['scene']==0:
            # Fixed displacements test alignment rather than optimizing it.
            for name,shift in (('left',(-.5,0)),('right',(.5,0)),('up',(0,-.5)),('down',(0,.5))):
                item['shift'][name]=measure(u-references(meta,shift)[0],regions['interior'])
        measurements.append(item)
        previous=dict(error=error,fg=fg,foreground=regions['foreground'],background=regions['background'])
    late=[m for m in measurements if m['frame']>=32]
    groups={group:{region:pool(m[group][region] for m in late if region in m[group])
                   for region in sorted({k for m in late for k in m[group]})}
            for group in ('point','area','temporal','leakage','shift')}
    return dict(arm=arm['name'],window=[32,63],groups=groups,measurements=measurements)


def metric_controls():
    np=np_module(); arm=next(a for a in arms() if a['name']=='flat-fast-correct')
    meta=dict(arm='synthetic',scene=1,frame=40,velocity=arm['velocity'])
    point,area,regions,_,_=references(meta)
    padded=np.pad(area,((1,1),(1,1),(0,0)),mode='edge')
    blur=sum(padded[y:y+360,x:x+640] for y in range(3) for x in range(3))/9
    candidates=dict(ideal_area=area,constant_background=np.broadcast_to([.125,.25,.375],area.shape),
                    constant_foreground=np.broadcast_to([.875,.125,.0625],area.shape),blurred_area=blur)
    result={}
    previous_point,_,previous_regions,_,_=references(dict(meta,frame=39))
    for name,u in candidates.items():
        contrast=np.array([.75,-.125,-.3125]); g=np.sum((u-point)*contrast,axis=2)/np.dot(contrast,contrast)
        temporal=None
        if name.startswith('constant'):
            warped,valid=warp(u-previous_point,previous_regions['foreground'],arm['velocity'])
            temporal=measure((u-point)-warped,regions['foreground']&valid)
        result[name]=dict(point={k:measure(u-point,m) for k,m in regions.items()},
                          area={k:measure(u-area,m) for k,m in regions.items()},
                          background_leakage=measure(g,regions['background']),
                          foreground_temporal_residual=temporal)
    return dict(scope='CPU synthetic counterexamples, not AMD output',frame=40,results=result)


def audit(run,output):
    np=np_module(); run=Path(run); output=Path(output); output.mkdir(parents=True,exist_ok=False)
    result=dict(scope='Diagnostic ranking investigation; no image-quality acceptance',protocol_sha256=sha(HERE/'temporal-diagnosis-protocol.json'),
                numpy=np.__version__,repeats=[],metric_controls=metric_controls())
    first=run/'readbacks'/f'{NAMESPACE}.r1'
    identities={}
    for repeat in (1,2):
        directory=run/'readbacks'/f'{NAMESPACE}.r{repeat}'; check=validate(directory)
        pooled=[]; all_identical=True; paired_checks=0
        for arm in arms():
            data=analyze_arm(directory,arm); save(output/f'r{repeat}-{arm["name"]}.json',data)
            pooled.append({k:v for k,v in data.items() if k!='measurements'})
            for frame in range(64):
                meta=dict(arm=arm['name'],frame=frame); raw=payload(file_path(directory,meta,'rgba16f'))
                key=arm['name'],frame; identity=hashlib.sha256(raw).hexdigest()
                if repeat==1: identities[key]=identity
                else: all_identical &= identities[key]==identity
                reference=dict(arm=arm['group']+'-correct',frame=frame)
                a=payload(file_path(directory,meta,'inputs')); b=payload(file_path(directory,reference,'inputs'))
                if a[:691200]!=b[:691200] or a[-4:]!=b[-4:]: raise ValueError('Colour/depth/exposure changed across controls')
                if arm['motion']==0 and a!=b: raise ValueError('Jitter/reset control changed input bytes')
                paired_checks+=1
            print(f'Audited r{repeat} {arm["name"]}',flush=True)
        result['repeats'].append(dict(repeat=repeat,input_audit=check,arms=pooled,paired_input_checks=paired_checks,
                                      outputs_equal_repeat1=all_identical if repeat==2 else None))
    sheet(first,output/'contact-sheet.png')
    result.update(output_frames=2432,status='structurally_complete',quality_gate=None)
    save(output/'summary.json',result)
    return result


def sheet(directory,path):
    # Fixed frame/crop and controls, chosen in advance; use Pillow already
    # installed with the analysis runtime solely for annotated diagnostic PNGs.
    from PIL import Image, ImageDraw
    np=np_module(); panels=[]
    for group in ('flat-slow','flat-fast','textured-slow'):
        for control in ('correct','reverse','zero-motion','negated-jitter','reset-all'):
            meta=dict(arm=group+'-'+control,frame=40)
            u=image(file_path(directory,meta,'rgba16f'))[...,:3]
            arm=next(a for a in arms() if a['name']==meta['arm']); l,t,_,_=rectangle(dict(velocity=arm['velocity'],frame=40))
            x,y=int(2*l)-16,int(2*t)+20
            crop=u[y:y+72,x:x+128]
            panel=Image.new('RGB',(256,168),'white')
            panel.paste(Image.fromarray((np.clip(crop,0,1)*255+.5).astype('u1')).resize((256,144),Image.Resampling.NEAREST),(0,24))
            ImageDraw.Draw(panel).text((4,5),group+' / '+control,fill='black'); panels.append(panel)
    canvas=Image.new('RGB',(1280,504),'white')
    for i,panel in enumerate(panels): canvas.paste(panel,((i%5)*256,(i//5)*168))
    canvas.save(path)


def cases(configuration=None):
    from native import DLL_HASHES, build_case, environment, native_setup, regression_cases
    configuration=configuration or dict(namespace=NAMESPACE,protocol='temporal-diagnosis-protocol.json',
        script='temporal_diagnosis.py',analysis='temporal-diagnosis')
    ns=configuration['namespace']; source=HERE/configuration['protocol']
    proton=configuration.get('backend')=='proton'
    spec=json.loads(source.read_text()); planned=arms(spec)
    full,inputs=frame_plan(spec)
    repeats=spec['repeats']; frame_count=sum(len(full)-a.get('start',0) for a in planned)
    input_count=sum(f>=a.get('start',0) for a in planned for f in inputs)
    def table_text(values):
        return ''.join(f'{a["name"]} {a["scene"]} {a["velocity"][0]} {a["velocity"][1]} {a["motion"]} {a["jitter"]} {int(a["reset_all"])}'
                       +(f' {a["start"]} {a["reset_frame"]}' if 'cut_frame' in spec else '')
                       +(f' {a["jitter_phase"]} {a["jitter_x"]} {a["jitter_y"]} {a["dispatch_lag"]}' if 'jitter_phase' in a else '')+'\n' for a in values)
    def freeze(r):
        try: np=np_module()
        except ImportError as error: raise Blocked('Existing NumPy-equipped Python required') from error
        directory=r.directory/'cases'/r.current['id']
        shutil.copyfile(source,directory/'protocol.json')
        table=directory/'arms.txt'
        table.write_text(table_text(planned))
        jitter=directory/'jitter.txt'; jitter.write_text(''.join(f'{x:.17g} {y:.17g}\n' for x,y in jitter_table()))
        r.state['diagnosis_arms']=table; r.state['diagnosis_jitter']=jitter
        if ns==NAMESPACE: save(directory/'metric-controls.json',metric_controls())
        r.note(protocol_sha256=sha(source),arms=planned,numpy=np.__version__)
    regressions=regression_cases(ns,sharpening=False,movable=False)
    result=[Case(ns+'.protocol','Freeze the image matrix, jitter and evaluation protocol',freeze)]
    if proton:
        from proton import cases as proton_cases
        result+=proton_cases(regression=True)
        def executable(r):
            r.state[ns+'.build.fsr_sdk']=r.state['executable']
        result.append(Case(ns+'.build.fsr_sdk','Use the cross-built and Proton-launched test executable',executable,('proton.launch',)))
    else:
        result+=native_setup()+[regressions[0]]
    def child(name,input_only=False):
        selector='upscaler::dx12::native_dispatch::'+('proton' if proton else 'native')+'_temporal_diagnosis'+('_inputs' if input_only else '')
        def action(r):
            directory=r.directory/'readbacks'/f'{ns}.{name}'; directory.mkdir()
            table=r.state['diagnosis_arms']
            if not input_only and 'order_seeds' in spec:
                import random
                ordered=planned.copy(); seed=spec['order_seeds'][int(name[1:])-1]
                random.Random(seed).shuffle(ordered)
                table=r.directory/'cases'/r.current['id']/'arms.txt'
                table.write_text(table_text(ordered))
                r.note(order_seed=seed,arm_order=[a['name'] for a in ordered])
            env=environment(r); env.update(FSR_SDK_TEST_TEMPORAL_DIAGNOSIS_OUTPUT=directory,
                FSR_SDK_TEST_TEMPORAL_DIAGNOSIS_ARMS=table,FSR_SDK_TEST_TEMPORAL_CALIBRATION_JITTER=r.state['diagnosis_jitter'],
                FSR_SDK_TEST_TEMPORAL_DIAGNOSIS_FRAMES=str(len(full)))
            if 'cut_frame' in spec: env['FSR_SDK_TEST_TEMPORAL_DIAGNOSIS_CUT_FRAME']=str(spec['cut_frame'])
            if any('jitter_phase' in a for a in planned): env['FSR_SDK_TEST_TEMPORAL_DIAGNOSIS_JITTER_STUDY']='1'
            try:
                markers=(f'Temporal diagnosis {"input-only" if input_only else "matrix"}: COMPLETED frames={input_count if input_only else frame_count}','1 passed; 0 failed')
                arguments=[r.state[ns+'.build.fsr_sdk'],'--ignored','--exact',selector,'--nocapture','--test-threads=1']
                if proton:
                    from proton import invoke, loaded_modules, windows_path, wine_drives
                    drives=wine_drives(r.state['prefix'])
                    env={key:windows_path(value,drives) if isinstance(value,Path) else value for key,value in env.items()}
                    stderr=invoke(r,arguments,environment=env,markers=markers,validation=True,timeout=180 if input_only else 360)
                    command=r.current['commands'][-1]
                else:
                    _,stderr,command=r.process(arguments,cwd=r.state['native_stage'],env=env,markers=markers,timeout=180 if input_only else 360)
                observations=[line for line in stderr.splitlines() if line.startswith(('Adapter:','Driver query:','Upscaler provider:'))]
                r.note(elapsed_seconds=command['elapsed_seconds'],native_observations=observations)
                if not input_only:
                    modules={}
                    if proton:
                        modules=loaded_modules(stderr,r.state['native_stage'],drives)
                    for line in (() if proton else stderr.splitlines()):
                        if line.startswith('Temporal calibration module: '):
                            module_name,path=line[len('Temporal calibration module: '):].split(' path=',1); actual=Path(path).resolve()
                            if not actual.samefile(r.state['native_stage']/module_name) or sha(actual)!=DLL_HASHES[module_name]: raise Rejected('Loaded module mismatch')
                            modules[module_name]=dict(path=actual,sha256=sha(actual))
                    if set(modules)!=set(DLL_HASHES): raise Rejected('Missing loaded module observations')
                    r.note(actual_loaded_modules=modules)
            finally:
                cache=r.state.setdefault('diagnosis_pack_cache',{}) if configuration.get('pack_cache') else None
                retained=pack(directory,cache=cache); r.note(payloads=retained,decoded_bytes=sum(p['decoded_bytes'] for p in retained),compressed_bytes=sum(p['compressed_bytes'] for p in retained))
            check=validate(directory,input_only,planned,spec,proton=proton); save(r.directory/'cases'/r.current['id']/'input-audit.json',check)
            r.note(input_audit=check,image_quality='Evaluated separately by the frozen offline protocol',
                   directx_diagnostics='Unavailable counts permitted as UINT64_MAX; nonzero errors rejected' if proton else 'Required zero errors')
        requires=(ns+'.protocol',ns+'.build.fsr_sdk') if input_only else ((ns+'.input-only','proton.dispatch') if proton else (ns+'.input-only',ns+'.dispatch.single',ns+'.dispatch.temporal',ns+'.dispatch.semantics'))
        return Case(ns+'.'+name,'Independent input oracle' if input_only else f'Fixed {frame_count}-frame matrix',action,requires,selector=selector,scope='proton-gpu' if proton else 'native-gpu',
                    expectation={'evaluation':'passed','diagnostics':'proton' if proton else 'windows','watchdog_seconds':180 if input_only else 360})
    result.append(child('input-only',True))
    for case in regressions[1:]: case.requires+=(ns+'.input-only',)
    result+=([] if proton else regressions[1:])+[child('r'+str(i)) for i in range(1,repeats+1)]
    def analysis(r):
        r.process([sys.executable,HERE/configuration['script'],'audit','--run',r.directory,'--output',r.directory/'analysis'/configuration['analysis']],timeout=configuration.get('analysis_timeout',900))
    result.append(Case(ns+'.audit','Independent evaluation under the frozen protocol',analysis,tuple(ns+'.r'+str(i) for i in range(1,repeats+1)),scope='offline-analysis'))
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__); parser.add_argument('command',choices=['audit'])
    parser.add_argument('--run',required=True); parser.add_argument('--output',required=True)
    args=parser.parse_args(); audit(args.run,args.output)
