# SPDX-License-Identifier: MPL-2.0
"""Typed role/view verification against a separate pinned-header C++ consumer."""
from functools import lru_cache
import hashlib
import json
import math
import struct

from execution import Case, Rejected
from native import build_case, environment, native_setup
from motion_scale import PROVIDERS

SELECTOR='upscaler::dx12::native_dispatch::public_typed_formats'
COLORS=(2,26,24,28)  # RGBA32F, R11G11B10F, RGB10A2 UNORM, RGBA8 UNORM.
BASE=(10,41,34,41,10,10,61)
STRIDE={2:16,10:8,26:4,24:4,28:4,54:2,41:4,34:4,61:1}
DEPTH={39:(40,41,False),40:(40,41,True),19:(20,21,False),20:(20,21,True)}
STRIDE.update({fmt:4 for fmt in DEPTH})


def pixel(fmt, values):
    if fmt==2: return struct.pack('<4f',*values)
    if fmt==10: return struct.pack('<4e',*values)
    if fmt==28: return bytes(round(v*255) for v in values)
    if fmt==24:
        q=[round(v*m) for v,m in zip(values,(1023,1023,1023,3))]
        return struct.pack('<I',q[0]|q[1]<<10|q[2]<<20|q[3]<<30)
    if fmt==26:
        # Selected inputs are nonnegative dyadic values exactly representable
        # in these components, so the half exponent can be reused without rounding.
        h=[struct.unpack('<H',struct.pack('<e',v))[0] for v in values[:3]]
        return struct.pack('<I',(h[0]>>4)|((h[1]>>4)<<11)|((h[2]>>5)<<22))
    if fmt==54: return struct.pack('<e',values[0])
    if fmt==41 or fmt in DEPTH: return struct.pack('<f',values[0])
    if fmt==34: return struct.pack('<2e',*values[:2])
    if fmt==61: return bytes([round(values[0]*255)])
    raise ValueError('unselected format')


def decode(fmt, data):
    if fmt==2: return struct.unpack('<4f',data)
    if fmt==10: return struct.unpack('<4e',data)
    if fmt==28: return tuple(v/255 for v in data)
    bits=struct.unpack('<I',data)[0]
    if fmt==24: return tuple(((bits>>shift)&mask)/mask for shift,mask in ((0,1023),(10,1023),(20,1023),(30,3)))
    if fmt==26:
        def component(value,mantissa):
            exponent=value>>mantissa;fraction=value&((1<<mantissa)-1)
            if exponent==31: return math.inf if fraction==0 else math.nan
            return math.ldexp(fraction/(1<<mantissa),-14) if exponent==0 else math.ldexp(1+fraction/(1<<mantissa),exponent-15)
        return (component(bits&2047,6),component((bits>>11)&2047,6),component(bits>>22,5))
    raise ValueError('not an output format')


def workloads(generator=False):
    base=dict(formats=list(BASE),uav=0,generate=False,dynamic=False,reversed=False,hdr=False,sharpen=False,control=False,zero=False,alter=False)
    out=[dict(base,name='control',control=True),dict(base,name='baseline'),dict(base,name='altered',alter=True)]
    for fmt in COLORS:
        for uav in (False,True):
            formats=list(BASE);formats[0]=fmt
            out.append(dict(base,name=f'color-{fmt}-uav{int(uav)}',formats=formats,uav=int(uav)))
        for sharpen in (False,True):
            formats=list(BASE);formats[4]=fmt
            out.append(dict(base,name=f'output-{fmt}-sharp{int(sharpen)}',formats=formats,sharpen=sharpen))
    for reversed_depth in (False,True):
        for uav in (False,True):
            formats=list(BASE);formats[1]=54
            out.append(dict(base,name=f'depth-reversed{int(reversed_depth)}-uav{int(uav)}',formats=formats,reversed=reversed_depth,uav=2*int(uav)))
    for fmt in (2,26):
        formats=list(BASE);formats[0]=fmt
        out.append(dict(base,name=f'hdr-{fmt}',formats=formats,hdr=True))
    out.append(dict(base,name='mixed-dynamic',formats=[28,54,34,41,24,10,61],dynamic=True,uav=3))
    if generator:
        for fmt in COLORS:
            for slot in (0,5):
                for uav in (False,True):
                    formats=list(BASE);formats[slot]=fmt
                    out.append(dict(base,name=f'generate-{slot}-{fmt}-uav{int(uav)}',formats=formats,generate=True,uav=(1<<slot)*int(uav)))
                out.append(dict(base,name=f'generate-zero-{slot}-{fmt}',formats=formats,generate=True,zero=True))
    return out


@lru_cache(maxsize=64)
def color_bytes(fmt,phase,hdr,opaque,zero,alter):
    rows=[]
    for y in range(180):
        row=[]
        for x in range(320):
            values=(0,0,0,1) if zero else (
                0.125+((x-phase*4)//8%5)/8,
                0.125+(y//8%5)/8,
                0.25 if (x//16+y//16)%2 else 0.5,1)
            if not opaque and 80<=x<240 and 45<=y<135:
                values=(values[0]+(0 if zero else 0.25),*values[1:])
            if hdr: values=(values[0]*4,values[1]*4,values[2]*4,1)
            if alter: values=(values[2],values[0],values[1],1)
            row.append(pixel(fmt,values))
        rows.append(b''.join(row))
    return b''.join(rows)


def input_bytes(selection,phase,role):
    fmt=selection['formats'][role]
    if role in (0,5):
        return color_bytes(fmt,phase,selection['hdr'],role==5,selection['zero'],selection['alter'])
    if role==1:
        row=b''.join(pixel(fmt,((0.25 if x<160 else 0.75) if not selection['reversed'] else (0.75 if x<160 else 0.25),)) for x in range(320))
        return row*180
    if role==2: return pixel(fmt,(0 if phase==0 else -4,0))*320*180
    if role==3: return pixel(fmt,(1,))
    if role==4: return pixel(fmt,(1,0,1,0.25))*640*360
    if role==6: return b'\x55'*320*180
    raise ValueError('unselected resource role')


def output_check(data,seed,fmt,active,control):
    if len(data)!=640*360*STRIDE[fmt] or len(seed)!=len(data): raise Rejected('typed output byte count')
    stride=STRIDE[fmt];written=0;minimum=math.inf;maximum=-math.inf
    for y in range(active[1]):
        for x in range(active[0]):
            offset=(y*640+x)*stride;encoded=data[offset:offset+stride]
            values=decode(fmt,encoded)
            if not all(map(math.isfinite,values)): raise Rejected('non-finite typed output')
            minimum=min(minimum,*values[:3]);maximum=max(maximum,*values[:3])
            written+=encoded!=seed[offset:offset+stride]
    expected=0 if control else active[0]*active[1]
    if written!=expected: raise Rejected('incomplete typed active output writes')
    if control and data!=seed: raise Rejected('no-dispatch control changed output')
    return dict(written=written,finite=active[0]*active[1],min_rgb=minimum,max_rgb=maximum,sha256=hashlib.sha256(data).hexdigest())


def evaluate(text,command,directory,selection,provider,mode):
    if command['execution']!='completed' or command['exit_code']!=0 or text.count('1 passed; 0 failed')!=1:
        raise Rejected('typed format child did not complete')
    parse=lambda prefix:[json.loads(line[len(prefix):]) for line in text.splitlines() if line.startswith(prefix)]
    identities=[line for line in text.splitlines() if line.startswith('Format provider:')]
    if len(identities)!=1 or f'ProviderId({PROVIDERS[provider]})' not in identities[0] or f'"{provider}"' not in identities[0]:
        raise Rejected('wrong typed format provider')
    if parse('FORMAT_RESULT ')!=[dict(frames=3,cleanup_ok=True)]: raise Rejected('incomplete typed cleanup')
    if parse('FORMAT_FRAME ')!=[dict(phase=p,debug_errors=0,removed=0) for p in range(3)]: raise Rejected('typed diagnostics/fence failure')
    capabilities=parse('FORMAT_CAP ')
    if len(capabilities)!=21: raise Rejected('missing format capability evidence')
    frames=[]
    for phase in range(3):
        hashes=[]
        for role in range(7):
            cap=capabilities[phase*7+role];fmt=selection['formats'][role]
            if cap['role']!=role or cap['format']!=fmt: raise Rejected('format capability identity mismatch')
            if fmt in COLORS or fmt==54:
                required=0x20|0x100|(0 if role==4 else 0x200)
                if role==4 or selection['uav']&(1<<role): required|=0x2000000
                if cap['support1']&required!=required or role==4 and not cap['support2']&0x80:
                    raise Rejected('adapter lacks selected typed view/access capability')
            expected=input_bytes(selection,phase,role)
            data=(directory/f'{mode}-{phase}-{role}.bin').read_bytes()
            if len(data)!=len(expected): raise Rejected('typed resource row/byte count mismatch')
            if role!=4 and not (role==6 and selection['generate']):
                if data!=expected: raise Rejected('typed input byte/readback mismatch')
            if role==6 and selection['generate']:
                width,height=(160,90) if phase==1 else (320,180)
                active=b''.join(data[y*320:y*320+width] for y in range(height))
                if selection['zero']:
                    if any(active): raise Rejected('equal decoded colours did not generate zero mask')
                elif not any(v not in (0,0x55) for v in active): raise Rejected('generation did not write a reactive response')
                if any(v==0x55 for v in active): raise Rejected('incomplete generated active mask')
            hashes.append(hashlib.sha256(data).hexdigest())
        active=(512,288) if selection['dynamic'] and phase==1 else (640,360)
        metrics=output_check((directory/f'{mode}-{phase}-4.bin').read_bytes(),input_bytes(selection,phase,4),selection['formats'][4],active,selection['control'])
        frames.append(dict(phase=phase,resource_sha256=hashes,output=metrics))
    return dict(frames=frames,capabilities=capabilities)


def depth_workloads(generator=False):
    base=workloads()[0]
    result=[base,dict(base,name='baseline',control=False),dict(base,name='baseline-reversed',control=False,reversed=True),
            dict(base,name='altered',control=False,alter=True)]
    for fmt in DEPTH:
        for reversed_depth in (False,True):
            formats=list(BASE);formats[1]=fmt
            result.append(dict(base,name=f'depth-{fmt}-reversed{int(reversed_depth)}',formats=formats,control=False,reversed=reversed_depth))
        formats=list(BASE);formats[1]=fmt
        result.append(dict(base,name=f'depth-{fmt}-dynamic',formats=formats,control=False,dynamic=True))
    return result


def depth_audit(text,directory,selection,mode):
    fmt=selection['formats'][1]
    rows=[json.loads(line[len('FORMAT_DEPTH '):]) for line in text.splitlines() if line.startswith('FORMAT_DEPTH ')]
    if fmt not in DEPTH:
        if rows: raise Rejected('unexpected depth view evidence')
        return
    dsv,srv,typed=DEPTH[fmt]
    if len(rows)!=3 or any(row['allocation']!=fmt or row['dsv']!=dsv or row['srv']!=srv or
                          row['casting'] not in (0,1) or typed and not row['casting'] for row in rows):
        raise Rejected('missing or incorrect depth view/casting evidence')
    for phase in range(3):
        if (directory/f'{mode}-{phase}-depth-sampled.bin').read_bytes()!=input_bytes(selection,phase,1):
            raise Rejected('depth SRV sampled the wrong values/plane')
        if fmt in (19,20) and (directory/f'{mode}-{phase}-stencil.bin').read_bytes()!=b'\x5a'*(320*180):
            raise Rejected('stencil plane changed')


def cases(depth=False):
    result=native_setup(format_probe=True)
    build=build_case('formats');result.append(build)
    for tag,provider in (('fsr4','4.1.1'),('fsr3','3.1.5')):
        for family in ('local','movable'):
            group=('depth-formats.' if depth else 'formats.')+tag+'.'+family
            selections=depth_workloads() if depth else workloads(generator=provider=='3.1.5')
            for selection in selections:
                name=group+'.'+selection['name']
                def run(r,name=name,selection=selection,provider=provider,family=family):
                    directory=r.directory/'readbacks'/name;directory.mkdir(parents=True)
                    for phase in range(3):
                        for role in range(7): (directory/f'input-{phase}-{role}.bin').write_bytes(input_bytes(selection,phase,role))
                    observations={}
                    for mode in ('public','reference'):
                        env=environment(r)
                        settings=dict(PROVIDER=provider,FAMILY=family,MODE=mode,DIR=directory,FORMATS=','.join(map(str,selection['formats'])),
                                      UAV=selection['uav'],GENERATE=int(selection['generate']),DYNAMIC=int(selection['dynamic']),
                                      REVERSED=int(selection['reversed']),HDR=int(selection['hdr']),SHARPEN=int(selection['sharpen']),CONTROL=int(selection['control']))
                        env.update({'FSR_SDK_TEST_FORMAT_'+k:str(v) for k,v in settings.items()})
                        out,err,command=r.process([r.state[build.id],'--ignored','--exact',SELECTOR,'--nocapture','--test-threads=1'],
                            env=env,cwd=r.state['native_stage'],observe=True,timeout=180)
                        observations[mode]=evaluate(out+err,command,directory,selection,provider,mode)
                        depth_audit(out+err,directory,selection,mode)
                    for phase in range(3):
                        for role in (4,6) if selection['generate'] else (4,):
                            if (directory/f'public-{phase}-{role}.bin').read_bytes()!=(directory/f'reference-{phase}-{role}.bin').read_bytes():
                                raise Rejected('public/C++ same-format result mismatch')
                    r.note(selection=selection,provider_name=provider,family=family,typed_formats=observations,reference_byte_equal=True)
                    r.current['evaluation']='passed'
                result.append(Case(name,'Typed format public/C++ '+tag+' '+family+' '+selection['name'],run,
                    (build.id,) if selection['control'] else (build.id,group+'.control'),SELECTOR,'native-gpu',
                    dict(provider_name=provider,family=family,selection=selection,watchdog_seconds=180)))
            def sensitive(r,group=group):
                root=r.directory/'readbacks'
                if not any((root/(group+'.baseline')/f'public-{p}-4.bin').read_bytes()!=(root/(group+'.altered')/f'public-{p}-4.bin').read_bytes() for p in range(3)):
                    raise Rejected('typed colour input-sensitive control had no effect')
                r.note(input_sensitive=True)
            result.append(Case(group+'.sensitive','Independent changed-input sensitivity '+tag+' '+family,sensitive,
                               (group+'.baseline',group+'.altered'),scope='native-analysis'))
            if depth:
                def equivalent(r,group=group,selections=selections):
                    root=r.directory/'readbacks'
                    for selection in selections:
                        if selection['formats'][1] not in DEPTH or selection['dynamic']: continue
                        baseline='baseline-reversed' if selection['reversed'] else 'baseline'
                        for phase in range(3):
                            if (root/(group+'.'+selection['name'])/f'public-{phase}-4.bin').read_bytes()!=(root/(group+'.'+baseline)/f'public-{phase}-4.bin').read_bytes():
                                raise Rejected('depth-family output differs from matched R32F baseline')
                    r.note(depth_baseline_byte_equal=True)
                result.append(Case(group+'.equivalent','Depth family/R32F exact output comparison '+tag+' '+family,equivalent,
                    tuple(group+'.'+s['name'] for s in selections if not s['dynamic']),scope='native-analysis'))
    return result
