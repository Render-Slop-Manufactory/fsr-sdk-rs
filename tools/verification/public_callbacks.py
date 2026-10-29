# SPDX-License-Identifier: MPL-2.0
"""Production callback ABI exchange and public DX12 callback integration."""
from pathlib import Path
from collections import Counter
import json
from execution import Case, Rejected
from native import native_setup,build_case,environment
from records import sha
HERE=Path(__file__).resolve().parent

def cases(namespace='public-callbacks', registry=False, include_setup=True):
    result=native_setup() if include_setup else []
    def abi_build(r):
        directory=r.directory/(namespace+'-abi');directory.mkdir()
        selected_archive = registry or r.state.get('artifact_native', False)
        raw = (r.state['packages']/f'fsr-sdk-sys-{r.state["versions"]["fsr-sdk-sys"]}'
               if selected_archive else r.repository/'crates/fsr-sdk-sys')/'src/api.rs'
        template = (HERE/'production_callback_abi.rs').read_text(encoding='utf-8')
        marker = '#[path = "../../crates/fsr-sdk-sys/src/api.rs"]'
        if template.count(marker) != 1 or not raw.is_file():
            raise Rejected('Production callback ABI declaration source unavailable')
        source = directory/'exchange.rs'
        source.write_text(template.replace(marker, '#[path = '+json.dumps(raw.as_posix(), ensure_ascii=False)+']'),
                          encoding='utf-8', newline='\n')
        for name,toolchain in [('current',[]),('msrv',['+1.88.0'])]:
            r.process(['rustc',*toolchain,'--edition=2024','--crate-type=cdylib','-Dwarnings',
                       '-Copt-level=1',source,'-o',directory/(name+'.dll')])
        headers=r.state['headers']
        r.process(['cl','/nologo','/std:c++17','/EHsc','/W4','/WX','/MT','/DCALLBACK_ABI_ONLY',
                   '/I'+str(headers/'api/include'),'/I'+str(headers/'upscalers/include'),
                   HERE/'callback_spike.cpp','/Fe'+str(directory/'exchange.exe'),'/Fo'+str(directory/'exchange.obj')])
        r.state[namespace+'.abi']=directory
        r.note(raw_api=raw, raw_api_sha256=sha(raw), fixture_sha256=sha(source),
               registry_artifact=selected_archive, archive_hashes=r.state.get('archive_hashes') if selected_archive else None)
    result.append(Case(namespace+'.abi-build','Production raw aggregate with independent MSVC caller',abi_build,
                       ('windows.environment','abi.headers','abi.creation')+(('release.registry',) if registry else ()),scope='windows-compilation'))
    for name in ('current','msrv'):
        def exchange(r,name=name):
            directory=r.state[namespace+'.abi']
            out,_,_=r.process([directory/'exchange.exe','abi',directory/(name+'.dll')])
            rows=[json.loads(line[4:]) for line in out.splitlines() if line.startswith('ABI ')]
            expected=dict(size=56,alignment=8,handle_offset=48,exchanges_each_direction=4,negative_controls=4)
            if rows!=[expected]:raise Rejected('Incomplete production aggregate exchange')
            r.note(aggregate=expected,toolchain=name)
        result.append(Case(namespace+'.abi-'+name,'Production by-value exchange: '+name,exchange,(namespace+'.abi-build',),scope='native-abi'))
    def equipment(r):
        stage,headers=r.state['native_stage'],r.state['headers']
        r.process(['cl','/nologo','/std:c++17','/EHsc','/W4','/WX','/MT','/LD',
                   '/I'+str(headers/'api/include'),'/I'+str(headers/'upscalers/include'),
                   HERE/'public_callbacks.cpp','/Fo'+str(stage/'public_callbacks.obj'),
                   '/link','/OUT:'+str(stage/'public_callbacks.dll'),'/IMPLIB:'+str(stage/'public_callbacks.lib'),stage/'gpu_helper.lib','d3d12.lib'])
        r.note(equipment_sha256=sha(stage/'public_callbacks.dll'))
    result.append(Case(namespace+'.equipment','Externally owned process-lived C++ callback targets',equipment,('native.helper',),scope='windows-compilation'))
    build=build_case(namespace,registry=registry);result.append(build)
    def execute(r):
        env=environment(r);env['FSR_SDK_TEST_CALLBACK_DLL']=r.state['native_stage']/'public_callbacks.dll'
        out,err,_=r.process([r.state[build.id],'--ignored','--exact','--nocapture',
                          'upscaler::dx12::native_dispatch::callbacks::public_native_callbacks'],
                         env=env,cwd=r.state['native_stage'],timeout=240,markers=['Public callbacks: PASSED contexts=12 frames=20'])
        lines=(out+err).splitlines()
        contexts=[line for line in lines if line.startswith('PUBLIC_CONTEXTS ')]
        frames=[line for line in lines if line.startswith('PUBLIC_FRAME ')]
        legacy=[line for line in lines if line.startswith('PUBLIC_LEGACY ')]
        if len(contexts)!=4 or len(frames)!=20 or len(legacy)!=8:raise Rejected('Incomplete public callback inventory')
        events=[json.loads(line[6:]) for line in lines if line.startswith('EVENT ')]
        if not events or [e['seq'] for e in events]!=list(range(1,len(events)+1)):raise Rejected('Missing callback event sequence')
        counts=Counter(e['kind'] for e in events if e['phase']=='public-sdk')
        if any(counts[kind]==0 for kind in ('host_alloc','host_free','resource_alloc','resource_free','cb_alloc','message')):raise Rejected('Missing native allocation route')
        if not any(e['kind']=='heap_alloc' and e['phase']=='controlled-heap' for e in events):raise Rejected('Missing controlled heap route')
        balance=[json.loads(line[len('PUBLIC_BALANCE '):]) for line in lines if line.startswith('PUBLIC_BALANCE ')]
        if len(balance)!=1 or any(balance[0][key] for key in ('host_live','resource_live','heap_live')):raise Rejected('Unbalanced native allocation')
        r.note(contexts=contexts,frames=frames,legacy=legacy,balance=balance[0],native_callback_counts=dict(counts),
               heap_native_invoked=bool(counts['heap_alloc']),events=events,
               cross_set_frees=sum(e['kind']=='resource_free' and e['allocation_set'] not in (0,e['set']) for e in events))
    result.append(Case(namespace+'.native','Both public families and selected providers: callbacks, diagnostics, fencing and teardown',execute,
                       (build.id,namespace+'.equipment',namespace+'.abi-current',namespace+'.abi-msrv'),scope='native-gpu'))
    return result
