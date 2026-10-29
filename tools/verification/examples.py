# SPDX-License-Identifier: MPL-2.0
"""Compile complete public examples through the same downstream consumer."""
import os
import shutil

from execution import Case, Rejected


def prepare_examples(r,prefix,registry=False):
    state = r.state[prefix]
    package = state['packages']/(f'fsr-sdk-{r.state["versions"]["fsr-sdk"]}' if registry else 'fsr-sdk')
    directory = state['directory']/'examples'
    directory.mkdir()
    for path in (package/'examples').glob('*.rs'):
        shutil.copyfile(path,directory/path.name)
    if os.name != 'nt':
        r.note(complete_windows_snippets=0,platform='portable examples; Windows snippets excluded')
        return
    readme = (package/'README.md').read_text(encoding='utf-8')
    snippet = readme.split('```rust\n',1)[1].split('```',1)[0]
    (directory/'readme.rs').write_text('use fsr_sdk::upscaler::{ID3D12Device, ID3D12GraphicsCommandList, ID3D12Resource};\n'
        'unsafe fn integration(loader_path: &std::path::Path, device: ID3D12Device, command_list: ID3D12GraphicsCommandList, '
        'color: ID3D12Resource, depth: ID3D12Resource, motion_vectors: ID3D12Resource, exposure: ID3D12Resource, output: ID3D12Resource) '
        '-> Result<(), Box<dyn std::error::Error>> {\n'+snippet+'\nOk(())\n}\nfn main() {}\n')
    snippets, current = [], None
    for line in (package/'src/lib.rs').read_text(encoding='utf-8').splitlines():
        if line=='//! ```no_run': current=[]
        elif line=='//! ```' and current is not None:
            snippets.append(current)
            current=None
        elif current is not None:
            code=line[3:].removeprefix(' ')
            current.append(code.removeprefix('# '))
    if not snippets: raise Rejected('Complete public Rustdoc snippets absent')
    for index,snippet in enumerate(snippets):
        (directory/f'rustdoc-{index}.rs').write_text('\n'.join(snippet)+'\n')
    r.note(complete_windows_snippets=len(snippets),readme_included=True)


def example_cases(prefix='consumer',toolchain=None,registry=False):
    cases=[Case(prefix+'.examples.prepare','Prepare full public integration examples',lambda r:prepare_examples(r,prefix,registry=registry), (prefix+'.metadata',))]
    for features in ('dx12','dx12,movable','dx12,utils','dx12,movable,utils') if os.name=='nt' else ('','movable','utils'):
        def compile(r,features=features):
            state=r.state[prefix]
            args=['check','--manifest-path',state['directory']/'Cargo.toml','--examples']
            if features: args+=['--features',features]
            r.process(r.cargo(args,toolchain),env=state['env'])
        cases.append(Case(prefix+'.examples.'+(features.replace(',','-') or 'portable'),'Downstream complete examples',compile,(prefix+'.examples.prepare',)))
    if os.name == 'nt':
        for family in ('local', 'movable'):
            owner, setup = ('Upscaler', 'UpscalerSetup') if family == 'local' else ('MovableUpscaler', 'MovableUpscalerSetup')
            signatures = {
                'simple': ('unsafe ', "path: &std::path::Path, device: &ID3D12Device, slot: &mut Option<Owner>, list: &ID3D12GraphicsCommandList, resources: &UpscaleResources<'_>, frame: &FrameParameters, complete: Callback",
                    'first_frame(path, device, slot, list, resources, frame, complete)', '()'),
                'provider': ('', "setup: Setup<'_, '_>, options: &UpscalerOptions, id: fsr_sdk::query::ProviderId",
                    'selected_provider(setup, options, id)', 'Owner'),
                'abort': ('unsafe ', "slot: &mut Option<Owner>, list: &ID3D12GraphicsCommandList, resources: &UpscaleResources<'_>, frame: &FrameParameters, invalidate: Callback, complete: Callback",
                    'record_then_abandon(slot, list, resources, frame, invalidate, complete)', 'ContextUsability'),
                'utility': ('', "setup: Setup<'_, '_>, options: &UpscalerOptions, choice: fsr_sdk::utils::provider::FsrChoice",
                    'generation_choice(setup, options, choice)', 'Owner'),
            }
            for workflow, (safety, arguments, invocation, result) in signatures.items():
                features = 'dx12' + (',movable' if family == 'movable' else '') + (',utils' if workflow == 'utility' else '')
                binary = f'walkthrough-{family}-{workflow}'
                call = f'walkthrough::{family}::{invocation}'
                if safety:
                    call = 'unsafe { ' + call + ' }'
                source = ('#![allow(dead_code, unused_imports)]\n#![deny(unused_must_use)]\n'
                    '#[path="../../examples/upscaler_workflows.rs"] mod walkthrough;\n'
                    'use fsr_sdk::upscaler::{FrameParameters, UpscalerOptions, ContextUsability};\n'
                    f'use fsr_sdk::upscaler::dx12::{{{owner} as Owner, {setup} as Setup, ID3D12Device, ID3D12GraphicsCommandList, UpscaleResources}};\n'
                    'type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;\n'
                    'type Callback = fn() -> Result<()>;\n'
                    f'{safety}fn check({arguments}) -> Result<{result}> {{ {call} }}\nfn main() {{}}\n')
                def compile_walkthrough(r, binary=binary, source=source, features=features):
                    state = r.state[prefix]
                    path = state['directory'] / 'src/bin' / (binary + '.rs')
                    path.parent.mkdir(exist_ok=True)
                    with path.open('x', encoding='utf-8', newline='\n') as stream:
                        stream.write(source)
                    r.note(source=source, binary=binary, features=features)
                    r.process(r.cargo(['check', '--manifest-path', state['directory'] / 'Cargo.toml',
                        '--bin', binary, '--features', features], toolchain), env=state['env'])
                cases.append(Case(prefix+'.walkthrough.'+family+'.'+workflow,
                    'External integrated '+family+' '+workflow+' walkthrough', compile_walkthrough,
                    (prefix+'.examples.prepare',)))
    return cases
