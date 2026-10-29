# SPDX-License-Identifier: MPL-2.0
"""Experimental temporal calibration protocol, independent geometry/decoder and diagnostic audit.

NumPy is required only by this explicitly selected experiment. No image-quality
threshold is evaluated. Ordinary profiles and their standard-library tools do
not import NumPy. Invoke with an existing NumPy-equipped Python interpreter.
"""
import argparse
import gzip
import hashlib
import json
import math
from pathlib import Path
import shutil
import struct
import sys
import time
import zlib

from execution import Blocked, Case, Rejected
from records import save, sha

HERE = Path(__file__).resolve().parent
NAMESPACE = 'experiment.temporal-calibration'
ARMS = ('a-correct', 'b-correct', 'b-reverse-motion', 'b-negated-jitter', 'cut-history', 'cut-fresh')


def np_module():
    import numpy as np
    return np


def f32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]


def halton(index, base):
    value, factor = 0.0, 1.0
    while index:
        factor /= base
        value += (index % base) * factor
        index //= base
    return value


def jitter_table():
    return [[f32(halton(i, 2)-0.5), f32(halton(i, 3)-0.5)] for i in range(1, 33)]


def projected_rectangle(frame, cut=False):
    # Independently reconstruct the world-space plane and project through the
    # LH camera, rather than reading native positions or logged image metrics.
    focal = 90 / math.tan(math.pi / 6)
    camera = (192 / focal, 96 / focal) if cut and frame >= 37 else (0, 0)
    world_x = ((100 + 0.375*frame)-160)*2/focal
    world_y = (90-(50 + 0.25*frame))*2/focal
    left = 160 + focal*(world_x-camera[0])/2
    top = 90 - focal*(world_y-camera[1])/2
    # Rational projected positions are exact in the protocol; remove round-off
    # from the independent inverse/project calculation (far below 1e-10 pixel).
    return round(left, 10), round(top, 10), round(left+64, 10), round(top+44, 10)


def depth(z):
    # Perspective clip-space z/w, near->0 and far->1.
    a, b = 20/(20-0.5), -20*0.5/(20-0.5)
    return (a*z+b)/z


def truncate_half(values):
    np = np_module()
    bits = np.asarray(values, dtype='<f4').view('<u4')
    exponent = ((bits >> 23) & 255).astype('i4')-127+15
    if np.any((bits & 0x7fffffff != 0) & ((exponent <= 0) | (exponent >= 31))):
        raise ValueError('Probe input is outside the documented normal-half domain')
    return np.where((bits & 0x7fffffff) == 0, (bits >> 16) & 0x8000,
                    ((bits >> 16) & 0x8000) | (exponent << 10) | ((bits & 0x7fffff) >> 13)).astype('<u2')


def expected_inputs(meta):
    np = np_module()
    x, y = np.meshgrid(np.arange(320)+0.5, np.arange(180)+0.5)
    jx, jy = jitter_table()[meta['frame'] % 32]
    x, y = x-jx, y-jy
    frame, cut = meta['frame'], meta['camera_cut']
    l, t, r, b = projected_rectangle(frame, cut)
    fg = (x >= l) & (x < r) & (y >= t) & (y < b) if meta['scene'] else np.zeros(x.shape, bool)
    color = np.ones((180, 320, 4), dtype='f8')
    if meta['scene']:
        color[:, :, :3] = np.where(fg[..., None], [0.875, 0.125, 0.0625], [0.125, 0.25, 0.375])
    else:
        color[..., 0], color[..., 1], color[..., 2] = 0.125+x/1024, 0.25+y/1024, 0.375+(x+y)/2048
    d = np.where(fg, depth(2), depth(8)).astype('<f4')
    m = np.zeros((180, 320, 2), dtype='f8')
    if frame:
        m[fg] = [-0.375/320, -0.25/180]
    if cut and frame == 37:
        m[..., 0] += np.where(fg, 96/320, 24/320)
        m[..., 1] -= np.where(fg, 48/180, 12/180)
    if meta['reverse_motion']:
        m[fg] *= -1
    return b''.join((truncate_half(color).tobytes(), d.tobytes(), truncate_half(m).tobytes(), struct.pack('<f', 1)))


def payload(path):
    path = Path(path)
    return gzip.decompress(path.read_bytes()) if path.suffix == '.gz' else path.read_bytes()


def image(path):
    np = np_module()
    raw = payload(path)
    if len(raw) != 640*360*8:
        raise ValueError('Incomplete RGBA16F payload: '+str(path))
    # Independent IEEE binary16 decoding; no native unhalf() or native metrics.
    values = np.frombuffer(raw, dtype='<f2').reshape(360, 640, 4).astype('f8')
    if not np.isfinite(values).all() or np.any(np.frombuffer(raw, dtype='<u2') == 0x7bff):
        raise ValueError('Nonfinite or unwritten channel: '+str(path))
    return values


def validate_metadata(meta, input_only=False):
    frame, arm = meta['frame'], meta['arm']
    if arm not in ARMS+('input-a','input-b','input-cut'):
        raise ValueError('Unknown arm')
    if (meta['scene'] != (0 if arm in ('a-correct','input-a') else 1)
            or meta['camera_cut'] != (arm.startswith('cut-') or arm=='input-cut')
            or meta['reverse_motion'] != (arm=='b-reverse-motion')):
        raise ValueError('Scene/control identity mismatch')
    j = jitter_table()[frame % 32]
    expected_j = [-v for v in j] if arm == 'b-negated-jitter' else j
    if ([f32(v) for v in meta['image_jitter']] != j
            or [f32(v) for v in meta['dispatch_jitter']] != expected_j
            or meta['phase'] != frame % 32):
        raise ValueError('Jitter identity mismatch')
    expected_reset = frame == 0 or (meta['camera_cut'] and frame == 37)
    if meta['reset'] != expected_reset or meta['finish_hr'] != 0 or meta['fence'] != 1:
        raise ValueError('Reset/completion mismatch')
    if meta['debug_errors'] or meta['device_removed'] or meta['finite_pixels'] != 640*360:
        raise ValueError('Debug/device/finite failure')
    if meta['dispatch'] != ('not_run' if input_only else 'ok'):
        raise ValueError('Dispatch execution missing')
    if not input_only and meta['changed_pixels'] != 640*360:
        raise ValueError('Native unwritten-pixel observation')
    expected = dict(camera=[0.5,20,f32(math.pi/3),1], motion_scale=[320,180], exposure=1,
                    pre_exposure=1, flags=0, masks=False, sharpening=False, frame_ms=f32(1000/60))
    for name, value in expected.items():
        actual = meta[name]
        if name == 'camera': actual = [actual[0],actual[1],f32(actual[2]),actual[3]]
        if name == 'frame_ms': actual = f32(actual)
        if actual != value: raise ValueError('Dispatch parameter mismatch: '+name)


def rows(directory):
    return [json.loads(line) for line in (Path(directory)/'frames.jsonl').read_text().splitlines()]


def raw_path(directory, meta, suffix):
    return Path(directory)/f'{meta["arm"]}-{meta["frame"]:03}.{suffix}.gz'


def check_input_directory(directory, input_only=False):
    np = np_module()
    observations = []
    seen = set()
    for meta in rows(directory):
        key = meta['arm'],meta['frame']
        if key in seen: raise ValueError('Duplicate frame identity')
        seen.add(key)
        validate_metadata(meta,input_only)
        actual = payload(raw_path(directory,meta,'inputs'))
        expected = expected_inputs(meta)
        if actual != expected:
            mismatches = [i for i,(a,b) in enumerate(zip(actual,expected)) if a!=b]
            raise ValueError(f'Input byte mismatch {key}: bytes={len(actual)} first={mismatches[:8]}')
        # Representative interior and two pixels around the left silhouette,
        # retaining expected/actual decoded colour, depth and motion values.
        l,t,_,_ = projected_rectangle(meta['frame'],meta['camera_cut'])
        coords = [(8,8),(160,90),(min(319,max(1,int(l))),min(179,max(0,int(t+20)))),
                  (min(319,max(0,int(l)-1)),min(179,max(0,int(t+20))))]
        samples=[]
        for x,y in coords:
            i=y*320+x
            color=struct.unpack_from('<eeee',actual,i*8)
            z=struct.unpack_from('<f',actual,460800+i*4)[0]
            mv=struct.unpack_from('<ee',actual,691200+i*4)
            jx,jy=jitter_table()[meta['frame']%32]
            sx,sy=x+0.5-jx,y+0.5-jy
            l,t,r,b=projected_rectangle(meta['frame'],meta['camera_cut'])
            fg=bool(meta['scene'] and l<=sx<r and t<=sy<b)
            samples.append(dict(x=x,y=y,sample=[sx,sy],surface='foreground' if fg else 'background',
                                rgba=color,depth=z,motion_uv=mv,
                                previous_sample_from_stored_motion=[sx+320*mv[0],sy+180*mv[1]]))
        observations.append(dict(arm=meta['arm'],frame=meta['frame'],bytes=len(actual),
                                 decoded_sha256=hashlib.sha256(actual).hexdigest(),samples=samples))
    expected_count = 21 if input_only else 301
    if len(observations)!=expected_count: raise ValueError('Frame count mismatch')
    planned = ({(arm,frame) for arm in ('input-a','input-b','input-cut') for frame in (0,1,31,32,37,40,63)}
               if input_only else {(arm,frame) for arm in ARMS for frame in
                    (range(37,41) if arm=='cut-fresh' else range(41) if arm=='cut-history' else range(64))})
    if seen!=planned: raise ValueError('Frame inventory differs from frozen protocol')
    # Independently prove projection endpoint and movement identities.
    if depth(0.5)!=0 or not math.isclose(depth(20),1): raise ValueError('Depth endpoints')
    if not np.allclose(np.subtract(projected_rectangle(32),projected_rectangle(31)),[.375,.25,.375,.25],rtol=0,atol=1e-10):
        raise ValueError('Motion restarted at the phase boundary')
    return dict(status='passed',complete_input_bytes_checked=sum(o['bytes'] for o in observations),
                frames=observations,depth_fg=f32(depth(2)),depth_bg=f32(depth(8)),
                decoder='Python struct IEEE binary16; NumPy IEEE binary16 for full images')


def pack(directory,cache=None):
    records=[]
    for path in sorted(Path(directory).glob('*')):
        if path.suffix not in ('.inputs','.rgba16f'): continue
        raw=path.read_bytes()
        identity=hashlib.sha256(raw).hexdigest(); key=len(raw),identity
        target=path.with_suffix(path.suffix+'.gz')
        with target.open('xb') as output:
            output.write(cache[key].read_bytes() if cache is not None and key in cache
                         else gzip.compress(raw,compresslevel=6,mtime=0))
        if payload(target)!=raw: raise ValueError('Lossless packaging mismatch')
        if cache is not None: cache[key]=target
        records.append(dict(file=target.name,decoded_bytes=len(raw),decoded_sha256=identity,
                            compressed_bytes=target.stat().st_size,sha256=sha(target)))
        path.unlink()  # Only the round-trip-verified file in this fresh attempt.
    save(Path(directory)/'payloads.json',records)
    return records


def stats(values, pixels=None):
    np=np_module()
    a=np.asarray(values,dtype='f8').ravel()
    if not a.size: return dict(pixels=0,components=0,mae=None,rmse=None,p95=None,maximum=None,signed_mean=None)
    return dict(pixels=int(pixels if pixels is not None else a.size//3),components=int(a.size),
                mae=float(np.abs(a).mean()),rmse=float(np.sqrt(np.square(a).mean())),
                p95=float(np.quantile(np.abs(a),.95)),maximum=float(np.abs(a).max()),signed_mean=float(a.mean()))


def references(meta):
    np=np_module()
    x,y=np.meshgrid((np.arange(640)+.5)/2,(np.arange(360)+.5)/2)
    valid=(x>=4)&(x<316)&(y>=4)&(y<176)
    if not meta['scene']:
        ref=np.stack((.125+x/1024,.25+y/1024,.375+(x+y)/2048),axis=2)
        zero=np.zeros(valid.shape,bool)
        return ref,ref,dict(interior=valid,foreground=zero,background=valid,edge=zero),np.full(valid.shape,np.inf),zero
    l,t,r,b=projected_rectangle(meta['frame'],meta['camera_cut'])
    fg=(x>=l)&(x<r)&(y>=t)&(y<b)
    outside=np.hypot(np.maximum(np.maximum(l-x,x-r),0),np.maximum(np.maximum(t-y,y-b),0))
    inside=np.minimum.reduce([x-l,r-x,y-t,b-y])
    distance=2*np.where(fg,inside,outside)
    # Exact overlap between half-render-pixel footprint and the rectangle.
    coverage=np.maximum(0,np.minimum(x+.25,r)-np.maximum(x-.25,l))*np.maximum(0,np.minimum(y+.25,b)-np.maximum(y-.25,t))*4
    bg=np.array([.125,.25,.375]); fc=np.array([.875,.125,.0625])
    point=np.where(fg[...,None],fc,bg)
    area=bg+coverage[...,None]*(fc-bg)
    regions=dict(interior=valid&(distance>2),foreground=valid&fg&(distance>2),
                 background=valid&~fg&(distance>2),edge=valid&(distance<=2))
    return point,area,regions,distance,fg


def reproject(previous, previous_mask):
    np=np_module()
    # True FG flow is (-.75,-.5) display pixels, excluding jitter and the cut.
    x,y=np.meshgrid(np.arange(640)-.75,np.arange(360)-.5)
    ix,iy=np.floor(x).astype(int),np.floor(y).astype(int)
    valid=(ix>=0)&(ix<639)&(iy>=0)&(iy<359)
    ix=np.clip(ix,0,638); iy=np.clip(iy,0,358)
    fx=(x-ix)[...,None]; fy=(y-iy)[...,None]
    result=(previous[iy,ix]*(1-fx)+previous[iy,ix+1]*fx)*(1-fy)+(previous[iy+1,ix]*(1-fx)+previous[iy+1,ix+1]*fx)*fy
    valid &= previous_mask[iy,ix]&previous_mask[iy,ix+1]&previous_mask[iy+1,ix]&previous_mask[iy+1,ix+1]
    return result,valid


def summarize(measurements):
    np=np_module()
    result={}
    for region in sorted({region for item in measurements for region in item['point']}):
        samples=[item['point'][region] for item in measurements if item['frame']>=32 and item['point'][region]['pixels']]
        if not samples:
            result[region]=dict(pixel_frames=0,mae=None,rmse=None)
        else:
            weights=np.array([s['pixels'] for s in samples],dtype='f8')
            result[region]=dict(pixel_frames=int(weights.sum()),
                mae=float(np.average([s['mae'] for s in samples],weights=weights)),
                rmse=float(np.sqrt(np.average([s['rmse']**2 for s in samples],weights=weights))))
    return result


def analyze_arm(directory, arm):
    np=np_module()
    metadata=[m for m in rows(directory) if m['arm']==arm]
    valid=(np.indices((360,640))[1]>=8)&(np.indices((360,640))[1]<632)&(np.indices((360,640))[0]>=8)&(np.indices((360,640))[0]<352)
    permanent=valid.copy()
    for meta in metadata:
        permanent &= references(meta)[2]['background']
    age=np.full((360,640),999,dtype=int)
    prev=None
    measurements=[]
    phase_differences=[]
    for meta in metadata:
        frame=meta['frame']; u=image(raw_path(directory,meta,'rgba16f'))
        point,area,regions,distance,fg=references(meta)
        error=u[:,:,:3]-point
        regions['permanent_background']=permanent
        if prev and not (meta['camera_cut'] and frame==37):
            age=np.where(~fg,np.minimum(age+1,999),999)
            age=np.where(~fg&prev['fg'],0,age)
        else: age[:]=999
        for name,lo,hi in (('age0',0,0),('age1',1,1),('age2to5',2,5)):
            age_mask=valid&~fg&(age>=lo)&(age<=hi)
            regions[name]=age_mask
            for band,mask in (('edge',distance<=2),('2to8',(distance>2)&(distance<=8)),('over8',distance>8)):
                regions[name+'_'+band]=age_mask&mask
        item=dict(frame=frame,phase=meta['phase'],point={name:stats(error[mask]) for name,mask in regions.items()},
                  area={name:stats((u[:,:,:3]-area)[mask]) for name,mask in regions.items()},
                  reference_difference=stats((point-area)[regions['edge']]),temporal={},leakage={})
        if prev and not (meta['camera_cut'] and frame==37):
            bg_mask=regions['background']&prev['background']
            item['temporal']['background']=stats((error-prev['error'])[bg_mask])
            warped,mask=reproject(prev['error'],prev['foreground'])
            item['temporal']['foreground']=stats((error-warped)[regions['foreground']&mask])
        if meta['scene']:
            contrast=np.array([.75,-.125,-.3125])
            g=np.sum(error*contrast,axis=2)/float(np.dot(contrast,contrast))
            for name,mask in regions.items():
                if name.startswith('age') or name=='background':
                    selected=mask&~fg&(distance>2)
                    item['leakage'][name]=stats(g[selected],int(selected.sum()))
        if not meta['scene'] and frame>=32:
            earlier=dict(meta,frame=frame-32)
            phase_differences.append(dict(frame=frame,previous=frame-32,rgb=stats((u[:,:,:3]-image(raw_path(directory,earlier,'rgba16f'))[:,:,:3])[valid])))
        # Fixed traces around the original and trailing silhouette, preserving
        # absolute coordinates and every frame rather than selecting extrema.
        item['traces']=[dict(x=x,y=y,rgb=u[y,x,:3].tolist(),reference=point[y,x].tolist(),age=int(age[y,x]),
                             distance=float(distance[y,x]) if math.isfinite(distance[y,x]) else None)
                        for x,y in ((200,140),(226,160),(240,160),(300,160))]
        measurements.append(item)
        prev=dict(error=error,fg=fg,background=regions['background'],foreground=regions['foreground'])
    return dict(arm=arm,frames=len(metadata),late_window=[max(32,metadata[0]['frame']),metadata[-1]['frame']],
                units='Linear RGB; distances in display pixels; leakage dimensionless; temporal residual per frame',
                measurements=measurements,late_summary=summarize(measurements),phase32=phase_differences)


def paired(directory, first, second, frames):
    result=[]
    all_meta={(m['arm'],m['frame']):m for m in rows(directory)}
    for frame in frames:
        a,b=all_meta[first,frame],all_meta[second,frame]
        ua,ub=image(raw_path(directory,a,'rgba16f')),image(raw_path(directory,b,'rgba16f'))
        regions=references(a)[2]
        item=dict(frame=frame,rgba=stats(ua-ub,640*360),
                  rgb={name:stats((ua[:,:,:3]-ub[:,:,:3])[mask]) for name,mask in regions.items()},
                  inputs_identical=payload(raw_path(directory,a,'inputs'))==payload(raw_path(directory,b,'inputs')))
        result.append(item)
    return result


def pool(measurements):
    samples = [sample for sample in measurements if sample['pixels']]
    count = sum(sample['pixels'] for sample in samples)
    if not count:
        return dict(pixel_frames=0, mae=None, rmse=None, signed_mean=None)
    return dict(pixel_frames=count,
                mae=sum(s['pixels']*s['mae'] for s in samples)/count,
                rmse=(sum(s['pixels']*s['rmse']**2 for s in samples)/count)**.5,
                signed_mean=sum(s['pixels']*s['signed_mean'] for s in samples)/count)


def compact_diagnostics(arms, comparisons):
    """Pool the predeclared windows before discarding verbose frame analyses."""
    result = {}
    for data in arms:
        late = [m for m in data['measurements'] if m['frame'] >= 32]
        groups = {}
        for group in ('point', 'area', 'temporal', 'leakage'):
            regions = sorted({region for m in late for region in m[group]})
            groups[group] = {region: pool(m[group][region] for m in late if region in m[group])
                             for region in regions}
        if data['phase32']:
            groups['phase32'] = dict(pooled=pool(m['rgb'] for m in data['phase32']),
                                    first=data['phase32'][0], last=data['phase32'][-1])
        result[data['arm']] = dict(window=data['late_window'], groups=groups)
    controls = {}
    for name in ('motion', 'jitter'):
        late = [m for m in comparisons[name] if m['frame'] >= 32]
        controls[name] = {region: pool(m['rgb'][region] for m in late) for region in late[0]['rgb']}
    return dict(scope='Diagnostic calibration only; pooled frame window 32..63 except cut 37..40',
                units='Linear RGB; pixel_frames are summed ROI pixels, not independent statistical samples; leakage dimensionless',
                arms=result, paired_control_differences=controls)


def audit(run, output, namespace=NAMESPACE):
    np=np_module(); start=time.monotonic(); run=Path(run); output=Path(output)
    output.mkdir(parents=True,exist_ok=False)
    results=[]
    for repeat in range(1,4):
        directory=run/'readbacks'/f'{namespace}.r{repeat}'
        check=check_input_directory(directory)
        save(output/f'r{repeat}-input-audit.json',check)
        arms=[]; detailed=[]
        for arm in ARMS:
            analysis=analyze_arm(directory,arm)
            save(output/f'r{repeat}-{arm}.json',analysis)
            arms.append(dict(arm=arm,frames=analysis['frames'],late_summary=analysis['late_summary']))
            detailed.append(analysis)
            print(f'Audited r{repeat} {arm}',flush=True)
        comparisons=dict(motion=paired(directory,'b-correct','b-reverse-motion',range(64)),
                         jitter=paired(directory,'b-correct','b-negated-jitter',range(64)),
                         reset=paired(directory,'cut-history','cut-fresh',range(37,41)))
        if not all(item['inputs_identical'] for item in comparisons['reset']):
            raise ValueError('Reset pair has different current input bytes')
        save(output/f'r{repeat}-comparisons.json',comparisons)
        if repeat == 1:
            diagnostics = compact_diagnostics(detailed, comparisons)
        results.append(dict(repeat=repeat,arms=arms,reset=comparisons['reset']))
    spread=[]
    for arm in ARMS:
        first=run/'readbacks'/f'{namespace}.r1'
        for repeat in (2,3):
            other=run/'readbacks'/f'{namespace}.r{repeat}'
            entries=[]
            for meta in rows(first):
                if meta['arm']!=arm: continue
                a,b=payload(raw_path(first,meta,'rgba16f')),payload(raw_path(other,meta,'rgba16f'))
                entry=dict(frame=meta['frame'],bit_identical=a==b)
                entry['rgb']=stats(image(raw_path(first,meta,'rgba16f'))[:,:,:3]-image(raw_path(other,meta,'rgba16f'))[:,:,:3])
                entries.append(entry)
            spread.append(dict(arm=arm,repeat=repeat,frames=entries))
    save(output/'repeat-spread.json',spread)
    sheet(run,output/'contact-sheet.png',namespace)
    result=dict(status='structurally_complete',diagnostic_only=True,image_quality_thresholds=None,
                numpy=np.__version__,python=sys.version,output_frames=903,input_only_frames=21,
                elapsed_seconds=round(time.monotonic()-start,3),repeats=results,
                repeat_all_bit_identical=all(f['bit_identical'] for arm in spread for f in arm['frames']))
    save(output/'summary.json',result)
    diagnostics.update(repeat_all_bit_identical=result['repeat_all_bit_identical'],
                       output_frames=903, input_only_frames=21,
                       reset_pairs=12,
                       reset_current_inputs_identical=all(item['inputs_identical'] for repeat in results for item in repeat['reset']),
                       reset_all_rgba_equal=all(item['rgba']['maximum']==0 for repeat in results for item in repeat['reset']))
    save(output/'diagnostic-summary.json',diagnostics)
    return result


def sheet(run,path,namespace=NAMESPACE):
    np=np_module(); directory=Path(run)/'readbacks'/f'{namespace}.r1'
    lookup={(m['arm'],m['frame']):m for m in rows(directory)}
    images=[image(raw_path(directory,lookup[arm,63],'rgba16f'))[:,:,:3] for arm in ARMS[:4]]
    history=image(raw_path(directory,lookup['cut-history',37],'rgba16f'))[:,:,:3]
    fresh=image(raw_path(directory,lookup['cut-fresh',37],'rgba16f'))[:,:,:3]
    images += [history,fresh,np.abs(history-fresh)*16,
               np.abs(images[1]-references(lookup['b-correct',63])[1])*4]
    pixels=np.concatenate([np.concatenate(images[:4],axis=1),np.concatenate(images[4:],axis=1)],axis=0)
    pixels=(np.clip(pixels,0,1)*255+.5).astype('u1')
    def chunk(tag,data):
        return struct.pack('>I',len(data))+tag+data+struct.pack('>I',zlib.crc32(tag+data)&0xffffffff)
    data=b''.join(b'\0'+row.tobytes() for row in pixels)
    Path(path).write_bytes(b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',2560,720,8,2,0,0,0))+
                           chunk(b'IDAT',zlib.compress(data))+chunk(b'IEND',b''))


def cases():
    from native import DLL_HASHES, native_setup, regression_cases, environment
    ns=NAMESPACE
    def protocol(r):
        try: np=np_module()
        except ImportError as error: raise Blocked('Existing NumPy-equipped Python required for this experiment only') from error
        directory=r.directory/'cases'/r.current['id']
        shutil.copyfile(HERE/'temporal-calibration-protocol.json',directory/'protocol.json')
        save(directory/'jitter.json',jitter_table())
        table=directory/'jitter.txt'
        table.write_text(''.join(f'{x:.17g} {y:.17g}\n' for x,y in jitter_table()))
        r.state['temporal_calibration_jitter']=table
        # The shared input inventory pins source hashes. Keep the small frozen
        # protocol and generator in the repository, without repeated snapshots.
        r.note(protocol_sha256=sha(directory/'protocol.json'),numpy=np.__version__,jitter=jitter_table())
    result=[Case(ns+'.protocol','Freeze bounded protocol, jitter and source identities',protocol)]
    regressions=regression_cases(ns,sharpening=False,movable=False)
    result+=native_setup()+[regressions[0]]
    def child(name,input_only=False):
        selector='upscaler::dx12::native_dispatch::native_temporal_'+('input_only' if input_only else 'calibration')
        def action(r):
            directory=r.directory/'readbacks'/f'{ns}.{name}'; directory.mkdir()
            env=environment(r); env.update(FSR_SDK_TEST_TEMPORAL_CALIBRATION_OUTPUT=directory,FSR_SDK_TEST_TEMPORAL_CALIBRATION_JITTER=r.state['temporal_calibration_jitter'])
            try:
                _,stderr,command=r.process([r.state[ns+'.build.fsr_sdk'],'--ignored','--exact',selector,
                    '--nocapture','--test-threads=1'],cwd=r.state['native_stage'],env=env,
                    markers=(f'Temporal calibration {"input-only" if input_only else "calibration"}: COMPLETED frames={21 if input_only else 301}',
                             '1 passed; 0 failed'),timeout=180 if input_only else 240)
                r.note(elapsed_seconds=command['elapsed_seconds'],native_observations=[line for line in stderr.splitlines()
                    if line.startswith(('Adapter:','Driver query:','Upscaler provider:','Temporal calibration memory:','Temporal calibration GPU memory:'))])
                if not input_only:
                    modules={}
                    for line in stderr.splitlines():
                        if line.startswith('Temporal calibration module: '):
                            module_name,path=line[len('Temporal calibration module: '):].split(' path=',1)
                            actual=Path(path).resolve()
                            # Windows may return the same module with a \\?\
                            # extended path prefix; compare filesystem identity.
                            if not actual.samefile(r.state['native_stage']/module_name) or sha(actual)!=DLL_HASHES[module_name]:
                                raise Rejected('Actual loaded module differs from pinned staged deployment')
                            modules[module_name]=dict(path=actual,sha256=sha(actual))
                    if set(modules)!=set(DLL_HASHES): raise Rejected('Actual loaded module observations missing')
                    r.note(actual_loaded_modules=modules)
            finally:
                retained=pack(directory)
                r.note(payloads=retained,decoded_bytes=sum(p['decoded_bytes'] for p in retained),
                       compressed_bytes=sum(p['compressed_bytes'] for p in retained))
            if len(retained)!=(21 if input_only else 602): raise Rejected('Missing raw payloads')
            check=check_input_directory(directory,input_only)
            save(r.directory/'cases'/r.current['id']/'input-audit.json',check)
            if not input_only:
                for meta in rows(directory): image(raw_path(directory,meta,'rgba16f'))
            r.note(structural_status='complete',image_quality_evaluation='diagnostic only')
        requires=(ns+'.protocol',ns+'.build.fsr_sdk') if input_only else (ns+'.input-only',ns+'.dispatch.single',ns+'.dispatch.temporal',ns+'.dispatch.semantics')
        return Case(ns+'.'+name,'Input-only validation' if input_only else 'Independent 301-frame calibration child',action,
                    requires,selector=selector,scope='native-gpu',expectation={'evaluation':'passed','structural_only':True,
                    'frames':21 if input_only else 301,'watchdog_seconds':180 if input_only else 240})
    result.append(child('input-only',True))
    for regression in regressions[1:]:
        regression.requires += (ns+'.input-only',)
    result += regressions[1:]
    result += [child(f'r{i}') for i in range(1,4)]
    def analysis(r):
        output=r.directory/'analysis'/'temporal-calibration'
        r.process([sys.executable,HERE/'temporal_calibration.py','audit','--run',r.directory,'--output',output],timeout=1200)
        summary=json.loads((output/'summary.json').read_text())
        r.note(status=summary['status'],output_frames=summary['output_frames'],image_quality_thresholds=None)
    # Independent failures do not block later repeats. The audit remains explicit
    # about partial evidence; it cannot count absent payloads as completed work.
    result.append(Case(ns+'.audit','Independent RGB, geometry, age, control, reset and repeat audit',analysis,
                      (ns+'.r1',ns+'.r2',ns+'.r3'),scope='offline-analysis'))
    return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=['audit'])
    parser.add_argument('--run',required=True); parser.add_argument('--output',required=True)
    parser.add_argument('--namespace',default=NAMESPACE,
                        help='Readback namespace; use experiment.v01-calibration for historical attempts')
    args=parser.parse_args()
    audit(args.run,args.output,args.namespace)
