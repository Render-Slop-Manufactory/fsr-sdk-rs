# SPDX-License-Identifier: MPL-2.0
"""P03 independently built C++ reference and separately launched Rust consumers."""
import json
from pathlib import Path
import re
import struct

from execution import Case, Rejected
from native import native_setup, build_case, environment
from performance import distribution
from records import save, sha

HERE = Path(__file__).resolve().parent
SELECTOR = 'upscaler::dx12::tests::cpu_comparison::native_cpu_comparison'


def expected_spec(render=(320,180), output=(640,360), *, dynamic=False, hdr=False, masks=False):
    rw,rh=render; ow,oh=output
    def resource(fmt, width, height, output=False):
        return [1,2,fmt,width,height,1,1,0,2 if output else 0,2 if output else 4]
    return {
        'creation':[65536,65547,(4<<22)|(1<<12)|1,2,(64 if dynamic else 0)|int(hdr),rw,rh,ow,oh,0],
        'shape':[65537,0,1,rw,rh,ow,oh,0,1,0],
        'float_bits':[struct.unpack('<I',struct.pack('<f',v))[0] for v in
                      (0,0,rw,rh,0,16.667,1,.1,100,1.0471975803375244140625,1)],
        'resources':[resource(4,rw,rh),resource(28,rw,rh),resource(18,rw,rh),
                     resource(28,1,1),resource(25,rw,rh) if masks else [0]*10,
                     resource(25,rw,rh) if masks else [0]*10,resource(4,ow,oh,True)],
    }


def parse(log, *, arm, debug=False, preinspect=False, render=(320,180), output=(640,360), dynamic=False, hdr=False, masks=False):
    def single(pattern):
        found = re.findall(pattern,log,re.M)
        if len(found)!=1:
            raise Rejected('Missing/duplicate CPU comparison metadata: '+pattern)
        return found[0]
    frequency=int(single(r'^CPU_CLOCK (\d+)$'))
    if frequency<=0:
        raise Rejected('Invalid QPC frequency')
    provider=single(r'^CPU_PROVIDER ([1-9]\d*) (.+)$')
    adapter_driver={'adapter':single(r'^Adapter: (.+)$'),
                    'driver':single(r'^Driver query: (.+)$')}
    spec=json.loads(single(r'^CPU_SPEC (.+)$'))
    if spec!=expected_spec(render,output,dynamic=dynamic,hdr=hdr,masks=masks):
        raise Rejected('Native descriptor differs from the selected canonical profile')
    frames=4 if debug else 160
    matrices={}
    for label,count in (('FRAME',frames),('EMPTY',256)):
        pairs=re.findall(r'^CPU_'+label+r' (\d+) (\d+)$',log,re.M)
        if len(pairs)!=count or {int(i) for i,_ in pairs}!=set(range(count)):
            raise Rejected('Incomplete/duplicate CPU '+label+' sample indices')
        matrices[label.lower()]=[int(value) for _,value in sorted(pairs,key=lambda pair:int(pair[0]))]
    pixels=output[0]*output[1]
    validity=f'CPU_VALID sentinel=0 finite={pixels} written={pixels} removed=0 debug={str(debug).lower()} preinspect={str(preinspect).lower()}'
    if single(r'^CPU_VALID .+$')!=validity or single(r'^CPU_COMPARISON PASSED (.+)$')!=arm:
        raise Rejected('Missing native validity/arm marker')
    native=matrices['frame'][32:] if not debug else matrices['frame']
    if not max(native)>0:
        raise Rejected('No measurable native recording interval')
    return {'arm':arm,'debug':debug,'preinspect':preinspect,'frequency_hz':frequency,
            'provider':{'id':int(provider[0]),'name':provider[1]},'spec':spec,
            'adapter_driver':adapter_driver,
            'ticks':matrices,'measured_ns':[value*1e9/frequency for value in native],
            'empty_ns':[value*1e9/frequency for value in matrices['empty']]}


def compatible(previous, current):
    for key in ('provider','spec','adapter_driver','frequency_hz','output_sha256'):
        if previous[key]!=current[key]:
            raise Rejected('Unmatched CPU comparison input/output: '+key)


def cases():
    protocol=json.loads((HERE/'cpu-comparison-protocol.json').read_text())
    build=build_case('cpu',release=True)

    def cpp_build(r):
        stage,headers=r.state['native_stage'],r.state['headers']
        executable=stage/'cpu-reference.exe'
        r.process(['cl','/nologo','/std:c++17','/O2','/W4','/WX','/MT','/EHsc',
                   f'/I{headers/"api/include/dx12"}',f'/I{headers/"upscalers/include"}',
                   r.repository/'crates/fsr-sdk/tests/native/cpu_reference.cpp',
                   f'/Fo{stage/"cpu_reference.obj"}',f'/Fe{executable}',
                   '/link',stage/'gpu_helper.lib','d3d12.lib'])
        r.state['cpu.cpp']=executable
        r.note(executable_sha256=sha(executable),optimizer='/O2 /MT',
               independence='Standalone consumer uses pinned AMD headers and SDK symbols; shared helper only supplies device/resources/clock/submission/readback')

    result=native_setup(optimized=True)+[build,Case('cpu.build.cpp','Independent optimized native C++ executable',cpp_build,('native.helper',),scope='windows-compilation')]
    previous=(build.id,'cpu.build.cpp')
    children=[]

    def child(name,arm,requires,debug=False,preinspect=False):
        case_id='cpu.'+name
        def action(r):
            env=environment(r)
            output=r.directory/'readbacks'/case_id/'final.rgba16f'
            output.parent.mkdir()
            env.update(FSR_SDK_CPU_ARM=arm,FSR_SDK_PERFORMANCE_DEBUG=str(int(debug)),
                       FSR_SDK_CPU_PREINSPECT=str(int(preinspect)),FSR_SDK_CPU_OUTPUT=output)
            args=[r.state['cpu.cpp']] if arm=='cpp-minimal' else [
                r.state[build.id],'--ignored','--exact',SELECTOR,'--nocapture','--test-threads=1']
            _,stderr,_=r.process(args,cwd=r.state['native_stage'],env=env,
                                 markers=('CPU_COMPARISON PASSED '+arm,),timeout=180)
            measured=parse(stderr,arm=arm,debug=debug,preinspect=preinspect)
            if not output.is_file() or output.stat().st_size!=640*360*8:
                raise Rejected('Missing/invalid final output payload')
            measured['output_sha256']=sha(output)
            cohort='debug' if debug else 'timed'
            key='cpu.reference.'+cohort
            if key in r.state:
                compatible(r.state[key],measured)
            else:
                r.state[key]=measured
            r.state[case_id]=measured
            save(r.directory/'cases'/case_id/'samples.json',measured)
            r.note(provider=measured['provider'],frequency_hz=measured['frequency_hz'],
                   output_sha256=measured['output_sha256'],canonical_descriptors_match=True,
                   dispatch=distribution(measured['measured_ns']),empty_clock=distribution(measured['empty_ns']),
                   adapter_driver=[line for line in stderr.splitlines() if line.startswith(('Adapter:','Driver query:'))])
        return Case(case_id,arm+' '+('debug validity control' if debug else 'CPU recording process'),action,requires,
                    SELECTOR if arm!='cpp-minimal' else 'cpu-reference.exe','native-experiment',
                    {'arm':arm,'debug_layer':debug,'preinspect':preinspect,'frames':4 if debug else 160,
                     'measured_frames':0 if debug else 128,'watchdog_seconds':180,
                     'contract':'Matched valid execution and complete evidence, no speed gate'})

    for arm in ('cpp-minimal','rust-raw','rust-wrapper'):
        case=child('diagnostics.'+arm,arm,previous,debug=True)
        result.append(case); previous=(case.id,)
    for block,order in enumerate(protocol['process_orders'],1):
        for arm in order:
            case=child(f'block{block}.'+arm,arm,previous)
            result.append(case); children.append(case.id); previous=(case.id,)
    for block,order in enumerate(protocol['warm_state_orders'],1):
        for arm,preinspect in order:
            condition='preinspected' if preinspect else 'prepared'
            case=child(f'warm{block}.{condition}.'+arm,arm,previous,preinspect=preinspect)
            result.append(case); children.append(case.id); previous=(case.id,)

    def analyze(r):
        medians={name:distribution(r.state[name]['measured_ns'])['median_ns'] for name in children}
        blocks=[]
        for block,order in enumerate(protocol['process_orders'],1):
            values={arm:medians[f'cpu.block{block}.'+arm] for arm in order}
            blocks.append({'block':block,'order':order,'process_median_ns':values,
                           'wrapper_minus_cpp_ns':values['rust-wrapper']-values['cpp-minimal'],
                           'raw_minus_cpp_ns':values['rust-raw']-values['cpp-minimal'],
                           'wrapper_minus_raw_ns':values['rust-wrapper']-values['rust-raw']})
        sensitivity=[]
        for block in range(1,4):
            values={condition:{arm:medians[f'cpu.warm{block}.{condition}.'+arm]
                               for arm in ('rust-wrapper','rust-raw')}
                    for condition in ('prepared','preinspected')}
            sensitivity.append({'block':block,'process_median_ns':values,
                'delta_ns':{condition:arms['rust-wrapper']-arms['rust-raw'] for condition,arms in values.items()}})
        summary={'protocol':protocol,'blocks':blocks,'warm_state_controls':sensitivity,
                 'interpretation':'Differences of separate process medians, not per-frame pairs. Descriptive only; no equivalence, GPU-time or speed acceptance verdict.'}
        directory=r.directory/'analysis'; directory.mkdir(exist_ok=True)
        save(directory/'cpu-comparison.json',summary)
        r.note(blocks=blocks,warm_state_controls=sensitivity,performance_threshold=None)
    result.append(Case('cpu.analysis','Process-block differences and warm-state sensitivity',analyze,tuple(children),scope='offline-analysis'))
    return result
