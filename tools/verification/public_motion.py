# SPDX-License-Identifier: MPL-2.0
"""Public native motion verification; independent encoding and readback checks."""
from functools import lru_cache
import hashlib
import json
import struct

from execution import Case, Rejected
from native import build_case, environment, native_setup
from motion_scale import PROVIDERS, bits, compare_frames
from variable_output import f32, half_truncated, output_metrics

SELECTOR = 'upscaler::dx12::native_dispatch::public_native_motion'
JITTER = ((0.25,-0.25),(-0.25,0.25),(0.125,-0.375))


def workloads():
    result = []
    for display in (False, True):
        base = dict(display=display, format=1, jitter=False, dynamic=False, uav=False)
        prefix = 'display' if display else 'render'
        for arm in ('reference','positive','mixed','wrong-x','wrong-y'):
            result.append(dict(base, arm=arm, name=prefix+'-'+arm))
        result.append(dict(base, arm='default', format=0, name=prefix+'-default'))
        for arm in ('reference','mixed','jitter-wrong'):
            result.append(dict(base, arm=arm, jitter=True, name=prefix+'-jitter-'+arm))
    for arm in ('reference','mixed','clear'):
        result.append(dict(display=True, format=1, jitter=False, dynamic=True, uav=True,
                           arm=arm, name='dynamic-uav-'+arm))
    return result


def encoding(selection, phase):
    active = (512,288) if selection['dynamic'] and phase == 1 else (640,360)
    extent = active if selection['display'] else (320,180)
    arm = selection['arm']
    scale = ((1.0,1.0) if arm in ('reference','clear') else extent if arm == 'default'
             else (2.0,4.0) if arm in ('positive','wrong-x') else (-2.0,-4.0) if arm == 'wrong-y' else (-2.0,4.0))
    encoded = (-2.0,4.0) if arm in ('wrong-x','wrong-y') else scale
    pixels = ((0.0,0.0) if phase == 0 else
              (f32(-4*f32(active[0]/320)),f32(-2*f32(active[1]/180))) if selection['display'] else (-4.0,-2.0))
    jitter = JITTER[phase] if selection['jitter'] else (0.0,0.0)
    previous = JITTER[phase-1] if selection['jitter'] and phase else jitter
    delta = tuple(f32(a-b) for a,b in zip(previous,jitter))
    stored = tuple(f32(f32(p/s)+d) if arm == 'jitter-wrong' else f32(f32(p+d)/s)
                   for p,d,s in zip(pixels,delta,encoded))
    return dict(active=active, motion_allocation=(640,360) if selection['display'] else (320,180),
                scale=scale, stored=stored, jitter=jitter)


@lru_cache(maxsize=6)
def color_input(phase, jitter):
    def wave(value):
        return abs(value % 16 - 8)/8
    # All terms are dyadic and exactly representable, including RGBA16F colour.
    reds = [0.125+0.75*wave(x+0.5+jitter[0]-phase*4) for x in range(320)]
    greens = [0.125+0.75*wave(y+0.5+jitter[1]-phase*2) for y in range(180)]
    return b''.join(struct.pack('<4e',red,green,0.25,1.0) for green in greens for red in reds)


def input_oracles(selection, phase):
    e = encoding(selection,phase)
    packed = (struct.pack('<2f',*e['stored']) if selection['format'] else
              struct.pack('<2H',*(half_truncated(v) for v in e['stored'])))
    return (color_input(phase,e['jitter']),struct.pack('<f',0.25)*(320*180),
            packed*(e['motion_allocation'][0]*e['motion_allocation'][1]),struct.pack('<f',1.0))


def descriptor(selection,phase,provider,family,control):
    e = encoding(selection,phase)
    return dict(phase=phase,provider=provider,family=family,arm=selection['arm'],format=selection['format'],
                display=selection['display'],jitter_included=selection['jitter'],dynamic=selection['dynamic'],
                uav=selection['uav'],control=control,active=list(e['active']),motion_allocation=list(e['motion_allocation']),
                scale_bits=list(map(bits,e['scale'])),stored_bits=list(map(bits,e['stored'])),jitter_bits=list(map(bits,e['jitter'])),
                override=selection['arm'] not in ('reference','default','clear'),native_format=6 if selection['format'] else 18,
                native_usage=2 if selection['uav'] else 0,reset=phase==0)


def evaluate(text,command,directory,selection,provider,family,control=False):
    if command['execution'] != 'completed' or command['exit_code'] != 0 or text.count('1 passed; 0 failed') != 1:
        raise Rejected('Public motion child did not complete its exact test')
    parse = lambda prefix: [json.loads(line[len(prefix):]) for line in text.splitlines() if line.startswith(prefix)]
    identities = [line for line in text.splitlines() if line.startswith('Public motion provider:')]
    if len(identities) != 1 or f'ProviderId({PROVIDERS[provider]})' not in identities[0] or f'"{provider}"' not in identities[0]:
        raise Rejected('Missing or wrong public motion provider identity')
    if parse('PUBLIC_MOTION_RESULT ') != [dict(outcome='completed',frames=3,cleanup_ok=True)]:
        raise Rejected('Public motion native rejection, incomplete execution or cleanup failure')
    descs, frames, capabilities = parse('PUBLIC_MOTION_DESCRIPTOR '),parse('PUBLIC_MOTION_FRAME '),parse('PUBLIC_MOTION_CAPS ')
    if len(descs) != 3 or len(frames) != 3 or len(capabilities) != 3:
        raise Rejected('Incomplete or duplicate public motion evidence')
    evidence = []
    for phase,(desc,frame,cap) in enumerate(zip(descs,frames,capabilities)):
        if desc != descriptor(selection,phase,provider,family,control):
            raise Rejected('Public motion family, descriptor bits or role metadata mismatch')
        required = 0x20|0x100|0x200|(0x2000000 if selection['uav'] else 0)
        if selection['format'] and cap['support1'] & required != required:
            raise Rejected('Independent device support does not cover public RG32F views')
        if frame['phase'] != phase or frame['completed'] != 1 or any(frame[k] for k in ('debug_errors','input_mismatches','removed')):
            raise Rejected('Public motion diagnostics, input mutation or incomplete fence')
        inputs = input_oracles(selection,phase)
        for role,data in enumerate(inputs):
            if (directory/f'phase-{phase}-role-{role}.bin').read_bytes() != data:
                raise Rejected(f'Independent public motion input oracle failed: phase={phase} role={role}')
        output = (directory/f'phase-{phase}-role-4.bin').read_bytes()
        if len(output) != 640*360*8:
            raise Rejected('Public motion output size mismatch')
        metrics = output_metrics(output,tuple(desc['active']),(640,360))
        if any(frame[k] != v for k,v in metrics.items()):
            raise Rejected('Independent public motion output audit disagrees with helper')
        expected = desc['active'][0]*desc['active'][1]
        if metrics['finite'] != expected or metrics['written'] != (0 if control else expected) or (control and metrics['outside']):
            raise Rejected('Public motion finite/write/control criterion failed')
        evidence.append(dict(descriptor=desc,capabilities=cap,metrics=frame,
                             input_sha256=[hashlib.sha256(data).hexdigest() for data in inputs],
                             output_sha256=hashlib.sha256(output).hexdigest()))
    if {p.name for p in directory.iterdir()} != {f'phase-{p}-role-{r}.bin' for p in range(3) for r in range(5)}:
        raise Rejected('Missing or extra public motion payloads')
    return dict(provider_name=provider,family=family,selection=selection,control=control,frames=evidence)


def comparison_pairs():
    pairs = []
    for prefix in ('render','display'):
        pairs += [(prefix+'-reference',prefix+'-'+arm,True) for arm in ('positive','mixed')]
        pairs += [(prefix+'-reference',prefix+'-'+arm,False) for arm in ('wrong-x','wrong-y')]
        pairs += [(prefix+'-jitter-reference',prefix+'-jitter-mixed',True),
                  (prefix+'-jitter-reference',prefix+'-jitter-jitter-wrong',False)]
    pairs += [('dynamic-uav-reference','dynamic-uav-'+arm,True) for arm in ('mixed','clear')]
    return pairs


def assess(read):
    measurements, criteria = [], {}
    for reference,candidate,equivalent in comparison_pairs():
        frames = [compare_frames(read(reference,p),read(candidate,p)) for p in range(3)]
        criteria[candidate] = all(f['equal'] for f in frames) if equivalent else frames[0]['equal'] and any(not f['equal'] for f in frames[1:])
        measurements += [dict(reference=reference,candidate=candidate,phase=p,equivalent=equivalent,**f) for p,f in enumerate(frames)]
    return dict(criteria=criteria,comparisons=measurements,passed=all(criteria.values()))


def cases():
    namespace = 'public-motion'
    result = native_setup(motion_probe=True)
    build = build_case(namespace)
    result.append(build)
    for tag,provider in (('fsr4','4.1.1'),('fsr3','3.1.5')):
        for family in ('local','movable'):
            group = namespace+'.'+tag+'.'+family
            selections = [(dict(workloads()[0],name='control'),True)]+[(s,False) for s in workloads()]
            for selection,control in selections:
                name = group+'.'+selection['name']
                def run(r,name=name,selection=selection,control=control,provider=provider,family=family):
                    directory = r.directory/'readbacks'/name
                    directory.mkdir(parents=True)
                    env = environment(r)
                    settings = dict(PROVIDER=provider,FAMILY=family,ARM=selection['arm'],FORMAT=selection['format'],
                                    DISPLAY=int(selection['display']),JITTER=int(selection['jitter']),DYNAMIC=int(selection['dynamic']),
                                    UAV=int(selection['uav']),CONTROL=int(control),DIR=directory)
                    env.update({'FSR_SDK_TEST_PUBLIC_MOTION_'+k:str(v) for k,v in settings.items()})
                    out,err,command = r.process([r.state[build.id],'--ignored','--exact',SELECTOR,'--nocapture','--test-threads=1'],
                                               cwd=r.state['native_stage'],env=env,observe=True,timeout=180)
                    r.note(raw_result_lines=[line for line in (out+err).splitlines() if line.startswith(('PUBLIC_MOTION_','Public motion provider:','Public motion rejection:','Adapter:','Driver query:'))])
                    r.note(public_motion=evaluate(out+err,command,directory,selection,provider,family,control))
                    r.current['evaluation'] = 'passed'
                requires = (build.id,) if control else (build.id,group+'.control')
                result.append(Case(name,'Public native motion '+tag+' '+family+' '+selection['name'],run,requires,
                                   SELECTOR,'native-gpu',dict(provider_name=provider,family=family,selection=selection,control=control,watchdog_seconds=180)))
            def analyze(r,group=group,provider=provider,family=family):
                def read(name,phase):
                    return (r.directory/'readbacks'/(group+'.'+name)/f'phase-{phase}-role-4.bin').read_bytes()
                assessment = assess(read)
                r.note(provider_name=provider,family=family,public_motion_comparisons=assessment)
                if not assessment['passed']:
                    raise Rejected('Public motion encoding equivalence or sensitive-control criterion failed')
            result.append(Case(group+'.compare','Public motion encoding/scale/jitter consistency '+tag+' '+family,analyze,
                               tuple(group+'.'+s['name'] for s in workloads()),scope='native-analysis'))
    return result
