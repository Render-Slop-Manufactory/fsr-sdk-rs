# SPDX-License-Identifier: MPL-2.0
"""Remaining external format roles: native calls, metadata, writes and cleanup."""
import argparse
from functools import lru_cache
import hashlib
import json
import math
from pathlib import Path
import shutil
import struct
from bgra_srgb_fsr import mask_check
from execution import Case, Rejected, Runner
from native import build_case, environment, native_setup
from records import fresh
from typed_formats import BASE, SELECTOR, pixel, decode
from motion_scale import PROVIDERS

ROOT = Path(__file__).resolve().parents[2]
# Fixed SDK views. RGB32F has no public-helper entry; its enum is explicit.
VIEWS = {1:2, 9:10, 23:24, 27:28, 90:87, 5:6, 6:6, 31:31, 67:67,
         15:16, 33:34, 53:54, 39:41, 60:61, 55:56, 44:46, 45:46, 56:56}
DEPTH = (44,45,55)
STRIDES = {2:16,10:8,24:4,28:4,87:4,6:12,31:4,67:4,16:8,34:4,54:2,41:4,61:1,56:2}


def encode(fmt, values):
    fmt = VIEWS.get(fmt,fmt)
    if fmt == 6: return struct.pack('<3f', *values[:3])
    if fmt == 16: return struct.pack('<2f', *values[:2])
    if fmt == 31: return struct.pack('<4b', *(round(v*127) for v in values))
    if fmt == 67:
        # Independent shared-exponent packing; these dyadic test values are exact.
        exponent = max(-16, math.floor(math.log2(max(values[:3])))) + 16 if max(values[:3]) else 0
        scale = math.ldexp(1.0, exponent-24)
        mantissas = [round(v/scale) for v in values[:3]]
        if max(mantissas) == 512:
            exponent += 1; mantissas = [round(v/math.ldexp(1.0,exponent-24)) for v in values[:3]]
        return struct.pack('<I', mantissas[0] | mantissas[1]<<9 | mantissas[2]<<18 | exponent<<27)
    if fmt == 87: return bytes(round(v*255) for v in (values[2],values[1],values[0],values[3]))
    if fmt == 56: return struct.pack('<H',round(values[0]*65535))
    return pixel(fmt,values)


@lru_cache(maxsize=128)
def colour(fmt,phase,opaque):
    return b''.join(encode(fmt,(0.125+((x-phase*4)//16%4)/8,
                               0.125+(y//16%4)/8,
                               0.25 if opaque or not (80<=x<240 and 45<=y<135) else 0.75,1))
                    for y in range(180) for x in range(320))


def inputs(s,phase,role):
    fmt=s['formats'][role]
    if role in (0,5): return colour(fmt,phase,role==5)
    if role==1:
        denominator=65535 if fmt in (55,56) else 16777215 if fmt in (44,45) else None
        values=(0.25,0.75) if not s['reversed'] else (0.75,0.25)
        if denominator:
            values=tuple(struct.unpack('<f',struct.pack('<f',round(v*denominator)/denominator))[0] for v in values)
        def depth(v): return struct.pack('<f',v) if fmt in DEPTH else encode(fmt,(v,))
        return (depth(values[0])*160+depth(values[1])*160)*180
    if role==2: return encode(fmt,(0 if phase==0 else -4,0))*320*180
    if role==3: return encode(fmt,(1,))
    if role==4: return encode(fmt,(1,0,1,0.25))*640*360
    if role==6: return b'\x55'*320*180
    raise ValueError(role)


def workloads():
    base=dict(formats=list(BASE),uav=0,generate=False,dynamic=False,reversed=False,sharpen=False,control=False,mask=0,depth_buffer=True)
    out=[dict(base,name='control',control=True)]
    for fmt in (1,9,23,27,90,5,6,31,67):
        for uav in (False,True):
            if fmt in (5,6) and uav: continue
            fs=list(BASE); fs[0]=fmt
            out.append(dict(base,name=f'color-{fmt}-uav{int(uav)}',formats=fs,uav=int(uav)))
        if fmt not in (5,6):
            for sharpen in (False,True):
                fs=list(BASE);fs[4]=fmt
                out.append(dict(base,name=f'output-{fmt}-sharp{int(sharpen)}',formats=fs,sharpen=sharpen))
        for role in (0,5):
            fs=list(BASE);fs[role]=fmt
            out.append(dict(base,name=f'generate-{role}-{fmt}',formats=fs,generate=True))
            if fmt not in (5,6):
                out.append(dict(base,name=f'generate-{role}-{fmt}-uav',formats=fs,generate=True,uav=1<<role))
    for fmt in (15,33,53,39,56,44,45,55):
        role=2 if fmt in (15,33) else 1
        for uav in (False,True):
            if fmt in DEPTH and uav: continue
            fs=list(BASE);fs[role]=fmt
            out.append(dict(base,name=f'role-{role}-{fmt}-uav{int(uav)}',formats=fs,uav=(1<<role)*int(uav),depth_buffer=fmt!=39))
        if fmt in DEPTH:
            out.append(dict(base,name=f'depth-{fmt}-reversed-active',formats=fs,reversed=True,dynamic=True))
    fs=list(BASE);fs[3]=39
    for uav in (False,True):
        out.append(dict(base,name=f'exposure-typeless-uav{int(uav)}',formats=fs,uav=8*int(uav)))
    for mask in (1,2):
        fs=list(BASE);fs[6]=60
        out.append(dict(base,name=f'mask-typeless-{mask}',formats=fs,mask=mask))
    fs=list(BASE);fs[6]=60
    out.append(dict(base,name='generate-typeless-output',formats=fs,generate=True))
    return out


def evaluate(text,command,directory,s,provider,mode):
    if command['execution']!='completed' or command['exit_code']!=0 or text.count('1 passed; 0 failed')!=1:
        raise Rejected('Native format child did not complete')
    parse=lambda prefix:[json.loads(line[len(prefix):]) for line in text.splitlines() if line.startswith(prefix)]
    identities=[line for line in text.splitlines() if line.startswith('Format provider:')]
    if len(identities)!=1 or f'ProviderId({PROVIDERS[provider]})' not in identities[0] or f'"{provider}"' not in identities[0]:
        raise Rejected('Wrong provider identity')
    if parse('FORMAT_RESULT ')!=[dict(frames=3,cleanup_ok=True)]: raise Rejected('Missing cleanup')
    if parse('FORMAT_FRAME ')!=[dict(phase=p,debug_errors=0,removed=0) for p in range(3)]: raise Rejected('Fence/debug/device failure')
    caps=parse('FORMAT_CAP ')
    if len(caps)!=21: raise Rejected('Incomplete capability inventory')
    frames=[]
    for phase in range(3):
        hashes=[]
        for role in range(7):
            if (caps[phase*7+role]['role'],caps[phase*7+role]['format'])!=(role,s['formats'][role]):
                raise Rejected('Wrong role/allocation inventory')
            raw=(directory/f'{mode}-{phase}-{role}.bin').read_bytes();expected=inputs(s,phase,role)
            if len(raw)!=len(expected): raise Rejected('Incomplete resource readback')
            if role!=4 and not (role==6 and s['generate']) and raw!=expected:
                raise Rejected(f'Input mutation or copy/normalisation mismatch in role {role}')
            hashes.append(hashlib.sha256(raw).hexdigest())
        raw=(directory/f'{mode}-{phase}-4.bin').read_bytes();seed=inputs(s,phase,4)
        fmt=VIEWS.get(s['formats'][4],s['formats'][4]);stride=STRIDES[fmt]
        active=(512,288) if s['dynamic'] and phase==1 else (640,360)
        written=0
        for y in range(active[1]):
            for x in range(active[0]):
                offset=(y*640+x)*stride;texel=raw[offset:offset+stride]
                written+=texel!=seed[offset:offset+stride]
                if fmt in (2,10) and not all(map(math.isfinite,decode(fmt,texel))): raise Rejected('Non-finite output')
        if written!=(0 if s['control'] else active[0]*active[1]): raise Rejected('Incomplete output writes/control')
        row=dict(phase=phase,resource_sha256=hashes,written=written)
        if s['formats'][1] in DEPTH:
            sampled=(directory/f'{mode}-{phase}-depth-sampled.bin').read_bytes()
            if sampled!=inputs(s,phase,1): raise Rejected('Wrong depth SRV samples')
            row['depth_sample_sha256']=hashlib.sha256(sampled).hexdigest()
            if s['formats'][1] in (44,45):
                stencil=(directory/f'{mode}-{phase}-stencil.bin').read_bytes()
                if stencil!=b'\x5a'*320*180: raise Rejected('Changed stencil plane')
                row['stencil_sha256']=hashlib.sha256(stencil).hexdigest()
        if s['generate']: row['mask']=mask_check((directory/f'{mode}-{phase}-6.bin').read_bytes(),s,phase)
        frames.append(row)
    return dict(frames=frames,capabilities=caps,depth_views=parse('FORMAT_DEPTH '))


def cases(direct=False,smoke=False):
    result=native_setup(format_probe=True);build=build_case('resource-formats');result.append(build)
    selections=workloads()
    if smoke: selections=[s for s in selections if s['name'] in ('control','color-6-uav0','output-67-sharp0','role-1-55-uav0','role-1-45-uav0','role-1-44-uav0','role-1-53-uav0','role-1-39-uav0','role-1-39-uav1','generate-5-6','generate-5-67-uav')]
    for provider in ('3.1.5','4.1.1'):
        for family in (('local',) if direct else ('local','movable')):
            group='resource-formats.'+provider+'.'+family
            for s in selections:
                if s['generate'] and provider!='3.1.5': continue
                name=group+'.'+s['name']
                def run(r,s=s,provider=provider,family=family,name=name):
                    directory=r.directory/'readbacks'/name;directory.mkdir(parents=True)
                    for phase in range(3):
                        for role in range(7): (directory/f'input-{phase}-{role}.bin').write_bytes(inputs(s,phase,role))
                    mode='reference' if direct else 'public'
                    settings=dict(PROVIDER=provider,FAMILY=family,MODE=mode,DIR=directory,FORMATS=','.join(map(str,s['formats'])),
                                  UAV=s['uav'],GENERATE=int(s['generate']),DYNAMIC=int(s['dynamic']),REVERSED=int(s['reversed']),
                                  HDR=0,SHARPEN=int(s['sharpen']),CONTROL=int(s['control']),MASK=s['mask'],DEPTH_BUFFER=int(s['depth_buffer']))
                    env=environment(r);env.update({'FSR_SDK_TEST_FORMAT_'+k:str(v) for k,v in settings.items()})
                    out,err,command=r.process([r.state[build.id],'--ignored','--exact',SELECTOR,'--nocapture','--test-threads=1'],
                                             env=env,cwd=r.state['native_stage'],observe=True,timeout=180)
                    observations=evaluate(out+err,command,directory,s,provider,mode)
                    r.note(selection=s,provider=provider,family=family,mode=mode,integration=observations,
                           raw_payload='discarded_after_successful_evaluation')
                    # This exclusive child directory contains only the inputs
                    # and completed readbacks just evaluated above. Keep failed
                    # payloads; successful evidence is the compact measurements.
                    parent=(r.directory/'readbacks').resolve()
                    if directory.is_symlink() or directory.resolve().parent!=parent:
                        raise Rejected('Unexpected disposable payload location')
                    shutil.rmtree(directory)
                    r.current['evaluation']='passed'
                result.append(Case(name,'Native external format '+s['name'],run,(build.id,),SELECTOR,'native-gpu'))
    return result


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--sdk-root',required=True);p.add_argument('--adapter-index',required=True,type=int)
    p.add_argument('--native',required=True,action='store_true');p.add_argument('--direct',action='store_true');p.add_argument('--smoke',action='store_true')
    options=p.parse_args();options.online=False;options.toolchain=None
    directory=fresh(ROOT/'target/verification','resource-formats-direct' if options.direct else 'resource-formats-public')
    print(directory,flush=True)
    raise SystemExit(Runner(ROOT,directory,options).execute(cases(options.direct,options.smoke),'resource-formats',require_passed=True))
