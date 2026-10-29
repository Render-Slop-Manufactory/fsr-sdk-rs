# SPDX-License-Identifier: MPL-2.0
"""Bounded source package/content audit; no final release or GPU claim."""
from pathlib import Path
import sys
root=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(root/'tools/verification'))
from execution import Case, Runner, Rejected
from profiles import equipment
from records import fresh
from release import release_cases
from run import parser

options=parser().parse_args(['run','release-artifacts']+sys.argv[1:])
directory=fresh(root/'target/verification','native-motion-package-audit')
print('Attempt:',directory,flush=True)
runner=Runner(root,directory,options)
runner.env['CARGO_TARGET_DIR']=str(root/'target/verification-build/current')
runner.state['external_inputs']=[('package-audit-driver',Path(__file__).resolve())]
for label in ('wrapper_archive','sys_archive'):
    if getattr(options,label): runner.state['external_inputs'].append((label,Path(getattr(options,label)).resolve()))
def contents(r):
    lists={}
    for name in ('fsr-sdk','fsr-sdk-sys'):
        out,_,_=r.process(r.cargo(['package','-p',name,'--allow-dirty','--list']))
        files=set(out.splitlines());lists[name]=sorted(files)
        if not {'README.md','LICENSE'}.issubset(files): raise Rejected('Missing package notices')
        if name=='fsr-sdk' and not {'src/upscaler/dx12/tests/native_motion.rs','tests/native/public_motion.rs','tests/native/public_motion.h'}.issubset(files):
            raise Rejected('New source/test equipment absent from package')
        if name=='fsr-sdk-sys' and 'LICENSE-AMD' not in files: raise Rejected('Missing AMD licence')
    r.note(package_contents=lists)
selected=release_cases()[:2]
for case in selected: case.requires=('package.contents',)+case.requires
def extracted(r):
    count=0
    for name,version in r.state['versions'].items():
        package=r.state['packages']/f'{name}-{version}'
        for file in package.rglob('*'):
            if not file.is_file() or file.name in ('Cargo.toml','Cargo.lock','.cargo_vcs_info.json'): continue
            source=root/'crates'/name/file.relative_to(package)
            if source.is_file():
                if file.read_bytes()!=source.read_bytes(): raise Rejected('Extracted source differs: '+str(file.relative_to(package)))
                count+=1
    r.process(r.cargo(['check','--manifest-path',r.state['packages']/'Cargo.toml','--workspace','--all-targets','--features','fsr-sdk/movable']),env=r.state['registry_env'])
    r.note(source_files_matched=count,all_targets_movable_check=True,final_release_verification=False)
cases=equipment()+[Case('package.contents','Cargo file inventories include new code and existing licence notices',contents,('equipment.toolchain',))]+selected+[Case('package.extracted','Extracted archive bytes match current source and compile all targets with movable',extracted,('release.registry',))]
code=runner.execute(cases,'native-motion-package-audit',require_passed=True)
print('Result:',directory/'run.json',flush=True)
sys.exit(code)
