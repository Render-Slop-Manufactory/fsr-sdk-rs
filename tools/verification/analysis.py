# SPDX-License-Identifier: MPL-2.0
"""Independent decoders for retained upscaler protocols (no GPU execution)."""
import array
import json
import math
from pathlib import Path
import re
import struct
import sys

from records import sha


def require(condition, message):
    if not condition:
        raise ValueError(message)


def fields(line):
    return dict(re.findall(r'(\w+)=([^ ]+)', line))


def red(path, width=640, height=360):
    data = Path(path).read_bytes()
    require(len(data) == width * height * 2, f'Invalid red-half payload size: {Path(path).name}')
    bits = array.array('H')
    bits.frombytes(data)
    if sys.byteorder != 'little':
        bits.byteswap()
    values = struct.unpack(f'<{width * height}e', data)
    require(all(math.isfinite(value) for value in values), 'Non-finite readback')
    return bits, values


def compare(a, b, width=640, height=360, inset=32):
    bits_a, values_a = a
    bits_b, values_b = b
    roi = [y * width + x for y in range(inset, height-inset) for x in range(inset, width-inset)]
    require(bool(roi), 'Empty comparison region')
    mismatch = sum(bits_a[index] != bits_b[index] for index in roi)
    return {'roi_bit_equal': mismatch == 0, 'roi_bit_mismatches': mismatch,
            'roi_mae_f64': sum(abs(values_a[index]-values_b[index]) for index in roi)/len(roi)}


def case_logs(root, case):
    root = Path(root)
    result = json.loads((root / 'cases' / case / 'result.json').read_text(encoding='utf-8'))
    commands = result['commands']
    require(len(commands) == 1, f'Expected one native child for {case}')
    command = commands[0]
    return result, command, (root / command['stdout']).read_text(encoding='utf-8'), (root / command['stderr']).read_text(encoding='utf-8')


def audit_sharpening_integration(root, namespace='native'):
    root = Path(root)
    frames, comparisons = [], []
    for repeat in range(1, 4):
        case = f'{namespace}.sharpening.integration.r{repeat}'
        result, command, stdout, stderr = case_logs(root, case)
        # Historical records keep their original prefixes. Normalize only the
        # protocol labels in memory; duplicate old/new observations still fail.
        stderr = re.sub(r'(?m)^M9_', 'SHARPENING_', stderr)
        stderr = re.sub(r'(?m)^M9 integration:', 'Sharpening integration:', stderr)
        require(command['execution'] == 'completed' and command['exit_code'] == 0 and
                '1 passed; 0 failed' in stdout and 'Sharpening integration: PASSED outputs=48' in stderr,
                f'Execution contract failed: {case}')
        directory = root / 'readbacks' / case
        require(len(list(directory.glob('*.red.u16le'))) == 48, f'Missing/extra images: {case}')
        decoded = {}
        for scene in range(2):
            for arm in ('switch-a', 'switch-b', 'disabled'):
                for frame in range(8):
                    prefix = f'SHARPENING_FRAME scene={scene} arm={arm} frame={frame} '
                    lines = [line for line in stderr.splitlines() if line.startswith(prefix)]
                    require(len(lines) == 1, f'Missing/duplicate frame observation: {prefix}')
                    observed = fields(lines[0])
                    for key, expected in (('finish_hr','0x0'),('fence','1'),('width','640'),('height','360'),
                                          ('finite','230400'),('written','230400'),('debug_errors','0'),('device_removed','0x0')):
                        require(observed.get(key) == expected, f'Frame contract failed: {key} in {prefix}')
                    path = directory / f'scene-{scene}-{arm}-frame-{frame}.red.u16le'
                    decoded[scene,arm,frame] = red(path)
                    frames.append({'repeat':repeat,'scene':scene,'arm':arm,'frame':frame,'sha256':sha(path),'observation':lines[0]})
            for frame in range(8):
                for pair, arm in (('a-b','switch-b'),('a-disabled','disabled')):
                    if pair == 'a-disabled' and frame < 3:
                        continue
                    measured = compare(decoded[scene,'switch-a',frame], decoded[scene,arm,frame])
                    prefix = f'SHARPENING_COMPARE scene={scene} frame={frame} pair={pair} '
                    lines = [line for line in stderr.splitlines() if line.startswith(prefix)]
                    require(len(lines) == 1, 'Missing/duplicate comparison observation')
                    observed = fields(lines[0])
                    require(int(observed['roi_bit_mismatches']) == measured['roi_bit_mismatches'] and
                            (observed['roi_bit_equal']=='true') == measured['roi_bit_equal'] and
                            abs(float(observed['roi_mae_f64'])-measured['roi_mae_f64']) <= 1e-16,
                            'Independent image calculation disagrees with native log')
                    comparisons.append(dict(repeat=repeat,scene=scene,frame=frame,pair=pair,diagnostic_only=True,**measured))
    require(len(frames) == 144 and len(comparisons) == 78, 'Incomplete sharpening integration audit')
    return {'schema_version':1,'protocol':'Sharpening direct-control integration','outputs':144,'frames':frames,
            'comparisons':comparisons,'image_effect_thresholds':None,'all_execution_checks_passed':True}


def characterize(root):
    """Decode available payloads even when the predeclared child gate failed."""
    root = Path(root)
    images = []
    for path in sorted((root / 'readbacks').rglob('*.red.u16le')):
        try:
            _, values = red(path)
            images.append({'file':path.relative_to(root).as_posix(),'sha256':sha(path),
                           'finite':len(values),'minimum':min(values),'maximum':max(values)})
        except ValueError as error:
            images.append({'file':path.relative_to(root).as_posix(),'error':str(error)})
    return {'schema_version':1,'claim':'Retained readback characterization; does not change original acceptance', 'images':images}


def audit_sharpening(root,namespace='experiment.sharpening'):
    measurements=[]
    roi=[y*640+x for y in range(32,328) for x in range(32,608)]
    gradient_roi=[y*640+x for y in range(32,328) for x in range(32,607)]
    def gradient(values): return sum(abs(values[i+1]-values[i]) for i in gradient_roi)/len(gradient_roi)
    for repeat in range(1,4):
        directory=Path(root)/'readbacks'/f'{namespace}.r{repeat}'
        require(len(list(directory.glob('*.red.u16le')))==10,'Incomplete original sharpening payloads')
        for scene in range(2):
            baseline=red(directory/f'scene-{scene}-off_a.red.u16le')
            g_off=gradient(baseline[1])
            for setting in ('off_a','off_b','zero','0.35','1'):
                path=directory/f'scene-{scene}-{setting}.red.u16le'
                decoded=red(path)
                deltas=[decoded[1][i]-baseline[1][i] for i in roi]
                d=sum(abs(value) for value in deltas)/len(roi)
                c=sum(abs(value)>=1/1024 for value in deltas)/len(roi)
                b=abs(sum(deltas)/len(roi))
                g=gradient(decoded[1])
                measurements.append(dict(repeat=repeat,scene=scene,setting=setting,sha256=sha(path),
                    D=d,C=c,B=b,G=g,G_off=g_off,gradient_ratio=g/g_off if g_off else None,
                    bit_mismatches=sum(decoded[0][i]!=baseline[0][i] for i in roi),
                    criteria=d>=.001 and c>=.01 and g_off>0 and g>=1.01*g_off and b<=.005))
    return {'schema_version':1,'measurements':measurements,
            'criteria':{'D_min':.001,'C_min':.01,'gradient_ratio_min':1.01,'B_max':.005},
            'scope':'Original D014 stationary scenes; independent measurements do not replace child acceptance or establish M9/V01 quality'}


def audit_transfer(root,namespace,matrix):
    outcomes=[]
    # The alternatives read frozen historical records without rewriting them.
    frames_pattern=r'(?:Transfer cleanup|D013-2) fenced context=(\d+) frame=(\d+) completed=(\d+) metrics=.*finite_pixels: (\d+), changed_pixels: (\d+).*debug_errors: (\d+), device_removed: (-?\d+)'
    for label,_,_,count in matrix:
        for arm in ('control','transfer'):
            for repeat in range(1,4):
                case=f'{namespace}.{label}-{arm}-r{repeat}'
                _,command,stdout,stderr=case_logs(root,case)
                require(command['execution']=='completed' and command['exit_code']==0 and '1 passed; 0 failed' in stdout,'Unsuccessful transfer child')
                frames=re.findall(frames_pattern,stderr)
                expected={'create-after-transfer':1,'history':64,'independent-siblings':17,'unwind':1,'sibling-unwind':3}[label]
                require(len(frames)==expected,'Incomplete fenced frame observations: '+case)
                require(re.search(r'residency checkpoint=1-[^\r\n]*helper=true',stderr) is not None,
                        'Helper residency was not observed while the application held it')
                require(all(frame[2]=='1' and frame[3]=='230400' and int(frame[4])>115200 and frame[5]=='0' and frame[6]=='0' for frame in frames),'Frame oracle failed')
                destroys=re.findall(r'(?:Context|D013) native destroy: thread=ThreadId\((\d+)\) code=(0x[0-9a-f]+)',stderr)
                require(bool(destroys) and all(code=='0x00000000' for _,code in destroys),'Missing or failed native destroy')
                for checkpoint in ('5','6'):
                    require(re.search(r'residency checkpoint='+checkpoint+r'-[^\r\n]*runtime_strong=Some\(0\) loader=false provider=(true|false) helper=false',stderr) is not None,'Application cleanup observation failed')
                outcomes.append({'case':case,'fenced_frames':len(frames),'native_destroy_ok':len(destroys),
                    'unwind_drops':len(re.findall(r'automatic Drop begin context=\d+ domain_held=true unwinding=true',stderr))})
    require(len(outcomes)==30 and sum(item['fenced_frames'] for item in outcomes)==516,'Incomplete control/transfer matrix')
    return {'schema_version':1,'outcomes':outcomes,'selected_payloads':96,'provider_termination':'not established'}
