# SPDX-License-Identifier: MPL-2.0
"""Bounded same-queue timestamp pilot; never infer GPU time from CPU clocks."""
import json
import math
from pathlib import Path
import re
import struct

from cpu_comparison import compatible as cpu_compatible, parse as parse_cpu
from execution import Case, Rejected
from native import native_setup, build_case, environment
from performance import distribution
from records import save, sha

HERE=Path(__file__).resolve().parent
SELECTOR='upscaler::dx12::tests::gpu_timing::native_gpu_timing'


def parse(log, *, arm, debug=False, dynamic=False, jitter_phases=None, hdr=False, masks=False):
    queries=re.findall(r'^GPU_QUERY (\d+) (\d+) (\d+)$',log,re.M)
    if len(queries)!=1:
        raise Rejected('Missing/duplicate GPU profile query')
    rw,rh,phase=map(int,queries[0])
    if not (0<rw<=1920 and 0<rh<=1080 and 0<phase<=2147483647):
        raise Rejected('Invalid queried render size/jitter phase')
    result=parse_cpu(log,arm=arm,debug=debug,render=(rw,rh),output=(1920,1080),dynamic=dynamic,hdr=hdr,masks=masks)
    frames=4 if debug else 160
    samples=re.findall(r'^GPU_FRAME (\d+) (\d+) (\d+) (\d+) (\d+) (\d+)$',log,re.M)
    jitters=re.findall(r'^GPU_JITTER (\d+) (\d+) (\d+)$',log,re.M)
    for values in (samples,jitters):
        if len(values)!=frames or {int(row[0]) for row in values}!=set(range(frames)):
            raise Rejected('Missing/duplicate GPU sample or jitter')
    ticks=[list(map(int,row[1:])) for row in sorted(samples,key=lambda row:int(row[0]))]
    frequency=ticks[0][4]
    if not 0<frequency<=2**64-1 or any(row[4]!=frequency for row in ticks):
        raise Rejected('Invalid/changing GPU queue frequency')
    for a,b,c,d,_ in ticks:
        if not (0<a<=b<=c<d<=2**64-1):
            raise Rejected('Non-monotonic/empty FSR GPU timestamps')
    if any(current[0]<previous[3] for previous,current in zip(ticks,ticks[1:])):
        raise Rejected('GPU timestamps moved backwards between fenced frames')
    jitter_bits=[list(map(int,row[1:])) for row in sorted(jitters,key=lambda row:int(row[0]))]
    seen={}
    for frame,bits in enumerate(jitter_bits):
        if any(value>2**32-1 or not math.isfinite(struct.unpack('<f',struct.pack('<I',value))[0]) for value in bits):
            raise Rejected('Invalid jitter float bits')
        count=jitter_phases[frame] if jitter_phases is not None else phase
        key=(count,frame%count)
        if key in seen and bits!=seen[key]:
            raise Rejected('Jitter does not repeat at the queried phase count')
        seen[key]=bits
    # The parser retains all raw timestamps, including warm-up and empty brackets.
    skip=0 if debug else 32
    result.update(query={'render':[rw,rh],'phase':phase},gpu_frequency_hz=frequency,
                  gpu_ticks=ticks,jitter_bits=jitter_bits,
                  gpu_ns=[(row[3]-row[2])*1e9/frequency for row in ticks[skip:]],
                  empty_gpu_ns=[(row[1]-row[0])*1e9/frequency for row in ticks])
    return result


def compatible(previous,current):
    cpu_compatible(previous,current)
    for key in ('query','gpu_frequency_hz','jitter_bits'):
        if previous[key]!=current[key]:
            raise Rejected('Unmatched GPU comparison: '+key)


def cases():
    protocol=json.loads((HERE/'gpu-timing-protocol.json').read_text())
    build=build_case('gpu-timing',release=True)

    def cpp_build(r):
        stage,headers=r.state['native_stage'],r.state['headers']
        executable=stage/'gpu-reference.exe'
        r.process(['cl','/nologo','/std:c++17','/O2','/W4','/WX','/MT','/EHsc',
                   f'/I{headers/"api/include/dx12"}',f'/I{headers/"upscalers/include"}',
                   r.repository/'crates/fsr-sdk/tests/native/gpu_reference.cpp',
                   f'/Fo{stage/"gpu_reference.obj"}',f'/Fe{executable}',
                   '/link',stage/'gpu_helper.lib','d3d12.lib'])
        r.state['gpu_reference_executable']=executable
        r.note(executable_sha256=sha(executable),optimizer='/O2 /MT',
               independence='Pinned AMD headers and SDK calls in standalone C++; shared equipment contains no AMD calls')

    result=native_setup(optimized=True,gpu_timing=True)+[build,
        Case('gpu.build.cpp','Independent optimized GPU reference',cpp_build,('native.helper',),scope='windows-compilation')]
    previous=(build.id,'gpu.build.cpp')
    children=[]

    def child(name,arm,requires,debug=False):
        case_id='gpu.'+name
        def action(r):
            output=r.directory/'readbacks'/case_id/'final.rgba16f'
            output.parent.mkdir()
            env=environment(r)
            env.update(FSR_SDK_CPU_ARM=arm,FSR_SDK_PERFORMANCE_DEBUG=str(int(debug)),
                       FSR_SDK_CPU_PREINSPECT='0',FSR_SDK_CPU_OUTPUT=output)
            args=[r.state['gpu_reference_executable']] if arm=='cpp-minimal' else [r.state[build.id],
                '--ignored','--exact',SELECTOR,'--nocapture','--test-threads=1']
            _,stderr,_=r.process(args,cwd=r.state['native_stage'],env=env,
                markers=('CPU_COMPARISON PASSED '+arm,),timeout=180)
            measured=parse(stderr,arm=arm,debug=debug)
            if not output.is_file() or output.stat().st_size!=1920*1080*8:
                raise Rejected('Missing/invalid GPU output payload')
            measured['output_sha256']=sha(output)
            key='gpu.reference.'+('debug' if debug else 'timed')
            if key in r.state: compatible(r.state[key],measured)
            else: r.state[key]=measured
            r.state[case_id]=measured
            save(r.directory/'cases'/case_id/'samples.json',measured)
            r.note(provider=measured['provider'],query=measured['query'],
                   gpu_frequency_hz=measured['gpu_frequency_hz'],output_sha256=measured['output_sha256'],
                   cpu=distribution(measured['measured_ns']),gpu=distribution(measured['gpu_ns']),
                   empty_gpu=distribution(measured['empty_gpu_ns']))
        return Case(case_id,arm+' '+('debug control' if debug else 'GPU timestamp process'),action,requires,
                    SELECTOR if arm!='cpp-minimal' else 'gpu-reference.exe','native-experiment',
                    {'debug_layer':debug,'frames':4 if debug else 160,'warmup':0 if debug else 32,
                     'watchdog_seconds':180,'contract':'Matched valid execution and complete timestamps; no speed gate'})

    for arm in ('cpp-minimal','rust-wrapper'):
        case=child('diagnostics.'+arm,arm,previous,True)
        result.append(case); previous=(case.id,)
    for block,order in enumerate(protocol['process_orders'],1):
        for arm in order:
            case=child(f'block{block}.'+arm,arm,previous)
            result.append(case); children.append(case.id); previous=(case.id,)

    def analyze(r):
        blocks=[]
        for block,order in enumerate(protocol['process_orders'],1):
            values={arm:{clock:distribution(r.state[f'gpu.block{block}.'+arm][key])
                        for clock,key in (('cpu','measured_ns'),('gpu','gpu_ns'),('empty_gpu','empty_gpu_ns'))}
                    for arm in order}
            blocks.append({'block':block,'order':order,'distributions':values,
                'wrapper_minus_cpp_median_ns':{clock:values['rust-wrapper'][clock]['median_ns']-values['cpp-minimal'][clock]['median_ns']
                                               for clock in ('cpu','gpu')}})
        summary={'protocol':protocol,'blocks':blocks,'performance_threshold':None}
        directory=r.directory/'analysis'; directory.mkdir(exist_ok=True)
        save(directory/'gpu-timing.json',summary)
        r.note(**summary)
    result.append(Case('gpu.analysis','Separate CPU/GPU distributions and process differences',analyze,tuple(children),scope='offline-analysis'))
    return result
