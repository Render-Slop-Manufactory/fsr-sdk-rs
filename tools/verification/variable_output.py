# SPDX-License-Identifier: MPL-2.0
"""Bounded raw output-geometry experiment; padding writes are observations."""
import array
import hashlib
import json
import struct
import sys

from execution import Case, Rejected
from native import build_case, environment, native_setup

REGIONS = {
    'stable': ((640,360),)*3, 'small': ((511,287),)*3,
    'aligned': ((640,360),(512,288),(640,360)),
    'rounded': ((637,357),(511,287),(637,357)),
    'width': ((637,357),(511,357),(637,357)),
    'height': ((637,357),(637,287),(637,357)),
}
SELECTOR = 'upscaler::dx12::native_dispatch::variable_output_probe'
PUBLIC_SELECTOR = 'upscaler::dx12::native_dispatch::public_variable_output'


def public_workloads():
    base=dict(allocation='maximum',motion_allocation='maximum',motion=0,dynamic=True,sharpen=False)
    result=[dict(base,name=name,sequence=sequence) for name,sequence in
            (('stable-max','stable'),('aligned-max','aligned'),('width-only','width'),('height-only','height'))]
    for motion in range(4):
        for sharpen in (False,True):
            result.append(dict(base,name=f'rounded-motion-{motion}'+('-sharp' if sharpen else ''),
                               sequence='rounded',motion=motion,sharpen=sharpen))
    return result


def workloads():
    result = [dict(name='stable-max',sequence='stable',allocation='maximum',motion_allocation='maximum',motion=0,dynamic=True,sharpen=False),
              dict(name='stable-small-large',sequence='small',allocation='large',motion_allocation='maximum',motion=0,dynamic=True,sharpen=False),
              dict(name='aligned-max',sequence='aligned',allocation='maximum',motion_allocation='maximum',motion=0,dynamic=True,sharpen=False)]
    for name, allocation, motion, dynamic in (
            ('rounded-max','maximum',0,True), ('rounded-fixed-flag','maximum',0,False),
            ('rounded-active','active',0,True), ('rounded-large','large',0,True),
            ('display-max','maximum',1,True), ('display-active','active',1,True),
            ('display-large','large',1,True)):
        for sharpen in (False,True):
            result.append(dict(name=name+('-sharp' if sharpen else ''),sequence='rounded',allocation=allocation,
                               motion_allocation=allocation,motion=motion,dynamic=dynamic,sharpen=sharpen))
    result += [dict(result[11],name='display-max-pixels',motion=2)]
    result += [dict(result[3],name=axis+'-only',sequence=axis) for axis in ('width','height')]
    return result


def allocation(policy, active):
    return active if policy=='active' else (672,384) if policy=='large' else (640,360)


def f32(value):
    return struct.unpack('<f',struct.pack('<f',value))[0]


def half_truncated(value):
    bits=struct.unpack('<I',struct.pack('<f',value))[0]
    exponent=((bits>>23)&255)-127+15
    return (bits>>16)&0x8000 if exponent<=0 else ((bits>>16)&0x8000)|(exponent<<10)|((bits&0x7fffff)>>13)


def input_oracles(desc):
    phase=desc['phase']
    jitter=[struct.unpack('<f',struct.pack('<I',bits))[0] for bits in desc['jitter_bits']]
    # Independent binary32 arithmetic and truncating binary16 packing, matching
    # the declared generator protocol rather than copying its upload storage.
    reds=[half_truncated(f32(f32(0.2)+f32(f32(0.4)*f32(f32(f32(f32(x+0.5)+jitter[0])/320)-f32(phase/128))))) for x in range(320)]
    greens=[half_truncated(f32(f32(0.2)+f32(f32(0.4)*f32(f32(f32(f32(y+0.5)+jitter[1])/180)-f32(phase/128))))) for y in range(180)]
    color=b''.join(struct.pack('<4H',r,g,half_truncated(0.3),0x3c00) for g in greens for r in reds)
    dx=0.0 if phase==0 else -1/128
    mx,my=desc['active'] if desc['motion_mode']==2 else (320,180) if desc['motion_mode']==3 else (1,1)
    motion=struct.pack('<2H',half_truncated(dx*mx),half_truncated(dx*my))* (desc['motion_allocation'][0]*desc['motion_allocation'][1])
    return [color,struct.pack('<f',0.25)*(320*180),motion,struct.pack('<f',1.0)]


def output_metrics(content, active, extent):
    width,height=extent
    if len(content)!=width*height*8:
        raise Rejected('Output payload length mismatch')
    values=array.array('H'); values.frombytes(content)
    if sys.byteorder!='little': values.byteswap()
    result=dict(finite=0,written=0,outside=0,right=0,bottom=0)
    for y in range(height):
        for x in range(width):
            index=(y*width+x)*4
            pixel=values[index:index+4]
            written=any(v!=0x7bff for v in pixel)
            if written:
                result['right']=max(result['right'],x+1); result['bottom']=max(result['bottom'],y+1)
            if x<active[0] and y<active[1]:
                result['finite']+=all(v&0x7c00!=0x7c00 for v in pixel)
                result['written']+=written
            else: result['outside']+=written
    return result


def evaluate(text, command, directory, selection, *, control=False, public_family=None):
    if command['execution']!='completed' or command['exit_code']!=0 or text.count('1 passed; 0 failed')!=1:
        raise Rejected('Output probe did not complete its exact child')
    parse=lambda prefix:[json.loads(line[len(prefix):]) for line in text.splitlines() if line.startswith(prefix)]
    if public_family is not None and parse('OUTPUT_ROUTE ')!=[dict(public=True,family=public_family)]:
        raise Rejected('Missing or wrong public output route/family')
    summaries=parse('OUTPUT_RESULT '); descs=parse('OUTPUT_DESCRIPTOR '); frames=parse('OUTPUT_FRAME ')
    if len(summaries)!=1:
        raise Rejected('Missing or duplicate output completion')
    summary=summaries[0]
    if not summary['cleanup_ok'] or type(summary['code']) is not int or not 0<=summary['code']<=0xffffffff:
        raise Rejected('Output terminal cleanup/code failure')
    count=summary['frames']
    rejection=summary['outcome']=='rejected'
    if public_family is not None and rejection:
        raise Rejected('Public active-output recording was rejected')
    if (type(count) is not int or len(frames)!=count or not 0<=count<=3 or
            len(descs)!=count+int(rejection) or (rejection and (control or summary['code']==0 or count>=3))):
        raise Rejected('Incomplete output frame/rejection evidence')
    if not rejection and (summary['outcome']!='completed' or count!=3 or summary['code']!=0):
        raise Rejected('Diagnostic or incomplete output result')
    evidence=[]
    for phase,desc in enumerate(descs):
        active=REGIONS[selection['sequence']][phase]
        extent=allocation(selection['allocation'],active)
        motion=allocation(selection['motion_allocation'],active) if selection['motion'] in (1,2) else (320,180)
        scale=(1,1) if selection['motion'] in (2,3) else active if selection['motion']==1 else (320,180)
        if (desc['phase']!=phase or tuple(desc['active'])!=active or tuple(desc['allocation'])!=extent or
                tuple(desc['motion_allocation'])!=motion or tuple(desc['motion_scale'])!=scale or
                desc['dynamic'] is not selection['dynamic'] or desc['sharpen'] is not selection['sharpen'] or
                desc['motion_mode']!=selection['motion'] or desc['control'] is not control or desc['reset'] is not (phase==0)):
            raise Rejected('Output descriptor/selection mismatch')
        if phase==count: continue  # Rejected list is never submitted/read back.
        frame=frames[phase]
        if frame['phase']!=phase or frame['completed']!=1 or any(frame[k]!=0 for k in ('debug_errors','removed','input_mismatches')):
            raise Rejected('Output GPU diagnostics/completion/input mismatch')
        inputs=input_oracles(desc)
        for role,expected in enumerate(inputs):
            if (directory/f'phase-{phase}-role-{role}.bin').read_bytes()!=expected:
                raise Rejected(f'Independent input oracle failed: phase={phase} role={role}')
        content=(directory/f'phase-{phase}-role-4.bin').read_bytes()
        independent=output_metrics(content,active,extent)
        if any(frame[key]!=value for key,value in independent.items()):
            raise Rejected('Independent output readback audit disagrees with helper')
        if control:
            if independent['written'] or independent['outside']:
                raise Rejected('No-provider control lost output seed')
        elif frame['finite']!=active[0]*active[1] or frame['written']!=active[0]*active[1]:
            raise Rejected('Requested active output is non-finite or incompletely written')
        evidence.append(dict(descriptor=desc,metrics=frame,output_sha256=hashlib.sha256(content).hexdigest(),
                             input_sha256=[hashlib.sha256(v).hexdigest() for v in inputs]))
    expected_files={f'phase-{phase}-role-{role}.bin' for phase in range(count) for role in range(5)}
    if {p.name for p in directory.iterdir()}!=expected_files:
        raise Rejected('Missing or extra output probe payload')
    return dict(summary=summary,frames=evidence,evaluation='observed' if rejection else 'passed')


def cases(public=False):
    namespace='public-variable-output' if public else 'variable-output'
    selector=PUBLIC_SELECTOR if public else SELECTOR
    result=native_setup(output_probe=True)
    build=build_case(namespace); result.append(build)
    control=dict(workloads()[3],name='control')
    if public:
        selections=[(tag+'.'+family,provider,selection,is_control,family)
            for tag,provider in (('fsr4','4.1.1'),('fsr3','3.1.5')) for family in ('local','movable')
            for selection,is_control in [(dict(public_workloads()[4],name='control'),True)]+[(s,False) for s in public_workloads()]]
    else:
        selections=[('control','4.1.1',control,True,None)]+[
            (tag,provider,selection,False,None) for tag,provider in (('fsr4','4.1.1'),('fsr3','3.1.5'),('fsr2','2.3.4')) for selection in workloads()]
    for tag,provider,selection,is_control,family in selections:
        name=namespace+'.'+tag+('.'+selection['name'] if public or not is_control else '')
        def run(r,selection=selection,provider=provider,is_control=is_control,name=name,family=family):
            directory=r.directory/'readbacks'/name; directory.mkdir(parents=True)
            variables=environment(r)
            settings=dict(PROVIDER=provider,SEQUENCE=selection['sequence'],ALLOCATION=selection['allocation'],
                          MOTION_ALLOCATION=selection['motion_allocation'],MOTION=selection['motion'],
                          DYNAMIC=int(selection['dynamic']),SHARPEN=int(selection['sharpen']),CONTROL=int(is_control),DIR=directory)
            if public: settings['FAMILY']=family
            variables.update({'FSR_SDK_TEST_OUTPUT_'+key:str(value) for key,value in settings.items()})
            out,err,command=r.process([r.state[build.id],'--ignored','--exact',selector,'--nocapture','--test-threads=1'],
                cwd=r.state['native_stage'],env=variables,observe=True,timeout=180)
            r.note(raw_result_lines=[line for line in (out+err).splitlines() if line.startswith(('OUTPUT_','Output probe provider:','Adapter:','Driver query:'))])
            evidence=evaluate(out+err,command,directory,selection,control=is_control,public_family=family)
            r.note(variable_output=evidence,public_variable_output=public,provider_name=provider,family=family)
            r.current['evaluation']=evidence['evaluation']
        control_id=namespace+'.'+tag+'.control' if public else namespace+'.control'
        requires=(build.id,) if is_control else (build.id,control_id)
        if not is_control and selection['name']!='stable-max': requires+=(namespace+'.'+tag+'.stable-max',)
        result.append(Case(name,('Public' if public else 'Raw')+' variable output '+tag+' '+selection['name'],run,requires,selector,'native-gpu' if public else 'native-experiment',
            {'provider_name':provider,'selection':selection,'control':is_control,'watchdog_seconds':180,
             'contract':'Require completed GPU work, full active output and clean diagnostics; padding writes are measured only'}))
    return result
