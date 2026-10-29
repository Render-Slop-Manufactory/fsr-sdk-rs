# SPDX-License-Identifier: MPL-2.0
"""Materialize and check a bounded setup/composition candidate without editing production."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent


def write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding='utf-8', newline='\n')


def change(path: Path, old: str, new: str) -> None:
    content = path.read_text(encoding='utf-8')
    if content.count(old) != 1:
        raise RuntimeError(f'Baseline anchor changed: {path.relative_to(ROOT)}: {old!r}')
    write(path, content.replace(old, new))


def consumers(candidate: Path) -> list[tuple[str, str, str | None]]:
    manifest = '''# SPDX-License-Identifier: MPL-2.0
[package]
name = "setup-spike-consumer"
version = "0.0.0"
edition = "2024"
[features]
default = []
dx12 = ["fsr-sdk/dx12"]
movable = ["fsr-sdk/movable"]
[dependencies]
fsr-sdk = { path = "../crates/fsr-sdk", default-features = false }
'''
    write(candidate / 'consumer/Cargo.toml', manifest)
    prefix = '''// SPDX-License-Identifier: MPL-2.0
#![allow(dead_code)]
use fsr_sdk::runtime::setup_spike as api;
use fsr_sdk::composition_spike::{Config, FrameParameters, HistoryAction};
use fsr_sdk::upscaler::{Dimensions, JitterOffsetPixels, FrameTimeMillis, CameraParameters};
use fsr_sdk::upscaler::dx12::{ID3D12Device, ID3D12Resource, ID3D12GraphicsCommandList};
use fsr_sdk::composition_spike::dx12::{UpscaleResources, ExposureInput};
use std::{path::Path, error::Error};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
'''
    cases: list[tuple[str, str, str | None]] = []

    def add(name: str, body: str, expected: str | None = None, features: str = 'dx12,movable', portable: bool = False) -> None:
        source = body if portable else prefix + body
        write(candidate / f'consumer/src/bin/{name}.rs', source)
        cases.append((name, features, expected))

    portable = '''// SPDX-License-Identifier: MPL-2.0
use fsr_sdk::composition_spike::{FrameParameters, HistoryAction};
use fsr_sdk::upscaler::common::{CameraParameters, FrameTimeMillis, JitterOffsetPixels, NativeSharpness, Sharpening};
fn main() -> Result<(), fsr_sdk::error::Error> {
    let mut frame = FrameParameters::new(JitterOffsetPixels::new(0.0,0.0)?, FrameTimeMillis::new(16.0)?, CameraParameters::new(0.1,100.0,1.0,1.0)?, HistoryAction::Reset);
    frame.set_sharpening(Sharpening::Native(NativeSharpness::new(0.5)?));
    Ok(())
}
'''
    add('portable', portable, features='', portable=True)
    simple = ''
    planned = ''
    record = ''
    for suffix, runtime, context in [('local', 'Runtime', 'Upscaler'), ('movable', 'MovableRuntime', 'MovableUpscaler')]:
        simple += f'''
fn simple_{suffix}(path: &Path, device: &ID3D12Device) -> Result<api::{context}> {{
    // SAFETY: This compile-only consumer assumes the draft loader/setup duties.
    let mut runtime = unsafe {{ api::{runtime}::load(path)? }};
    let config = Config::new(Dimensions::new(1280,720)?, Dimensions::new(1920,1080)?);
    let context = {{
        let mut setup = unsafe {{ runtime.upscaler_dx12(device) }};
        setup.create(&config)?
    }};
    drop(runtime);
    Ok(context)
}}
'''
        planned += f'''
fn planned_{suffix}(runtime: &mut api::{runtime}, device: &ID3D12Device) -> Result<api::{context}> {{
    // SAFETY: The caller establishes all safely selectable providers' processing duties.
    let mut setup = unsafe {{ runtime.upscaler_dx12(device) }};
    let providers = setup.providers()?;
    let mut setup = setup.with_provider(api::ProviderSelection::Explicit(providers[0].id()));
    let output = Dimensions::new(1920,1080)?;
    let render = setup.render_size_for_quality(output,fsr_sdk::query::QualityMode::Quality)?;
    let config = Config::new(render,output);
    let _ = setup.ratio(fsr_sdk::query::QualityMode::Quality)?;
    let phases = setup.jitter_phase_count(render,output)?;
    let _ = setup.jitter_offset(0,phases)?;
    let _ = setup.resource_requirements()?;
    let _ = setup.estimate_memory(&config)?;
    Ok(setup.create(&config)?)
}}
'''
        record += f'''
unsafe fn abandoned_{suffix}(context: &mut api::{context}, list: &ID3D12GraphicsCommandList, textures: &[ID3D12Resource;5], frame: &FrameParameters) -> Result<()> {{
    let resources = UpscaleResources::builder().color(&textures[0]).depth(&textures[1])
        .motion_vectors(&textures[2]).exposure(ExposureInput::Texture(&textures[3])).output(&textures[4]).build().map_err(|role| format!("missing {{role:?}}"))?;
    // SAFETY: The enclosing renderer establishes the complete recording contract.
    unsafe {{ context.record(list,&resources,frame)?; }}
    context.mark_recording_abandoned();
    // The renderer retains the owner/resources and invalidates the list before cleanup.
    Ok(())
}}
'''
    add('simple', simple + 'fn main() {}\n')
    add('planning', planned + 'fn main() {}\n')
    add('recording', record + 'fn main() {}\n')
    add('movable_traits', '''fn send<T: Send>() {}
fn main() { send::<api::MovableRuntime>(); send::<api::MovableUpscaler>(); send::<api::MovableUpscalerSetup<'static,'static>>(); }
''')
    add('setup_requires_unsafe', 'fn probe(r: &mut api::Runtime,d: &ID3D12Device) { let _ = r.upscaler_dx12(d); }\nfn main() {}', 'E0133')
    add('load_requires_unsafe', 'fn main() { let _ = api::Runtime::load("fixture.dll"); }', 'E0133')
    add('loaded_runtime_has_no_query', 'fn probe(r: &mut api::Runtime) { let _ = r.ratio(fsr_sdk::query::QualityMode::Quality); }\nfn main() {}', 'E0599')
    add('legacy_constructor_cannot_take_loaded_runtime', '''fn probe(r: &api::Runtime,d: &ID3D12Device,o: fsr_sdk::upscaler::UpscalerOptions) { let _ = fsr_sdk::upscaler::dx12::Upscaler::new(r,d,o); }
fn main() {}''', 'E0308')
    add('configuration_has_no_provider', '''fn probe(c: &mut Config) { c.set_provider(None); }
fn main() {}''', 'E0599')
    add('private_state', 'fn probe(r: api::Runtime) { let _ = r.state; }\nfn main() {}', 'E0616')
    add('two_live_setups', '''fn probe(r: &mut api::Runtime,d: &ID3D12Device) {
    let first = unsafe { r.upscaler_dx12(d) }; let _second = unsafe { r.upscaler_dx12(d) }; let _ = first.selection();
} fn main() {}''', 'E0499')
    add('resource_borrow_cannot_escape', '''fn escape(r: ID3D12Resource) -> UpscaleResources<'static> {
    UpscaleResources::builder().color(&r).depth(&r).motion_vectors(&r).exposure(ExposureInput::Automatic).output(&r).build().unwrap()
} fn main() {}''', 'E0515')
    for name, ty, trait in [('local_runtime_send','api::Runtime','Send'), ('local_context_send','api::Upscaler','Send'),
                           ('local_setup_send',"api::UpscalerSetup<'static,'static>",'Send'), ('local_runtime_sync','api::Runtime','Sync'),
                           ('local_context_sync','api::Upscaler','Sync'), ('local_setup_sync',"api::UpscalerSetup<'static,'static>",'Sync'),
                           ('movable_runtime_sync','api::MovableRuntime','Sync'), ('movable_context_sync','api::MovableUpscaler','Sync'),
                           ('movable_setup_sync',"api::MovableUpscalerSetup<'static,'static>",'Sync')]:
        add(name, f'fn require<T: {trait}>() {{}}\nfn main() {{ require::<{ty}>(); }}', 'E0277')
    add('callbacks_require_unsafe', '''fn probe(s: &mut api::UpscalerSetup<'_, '_>,c: &Config,callbacks: fsr_sdk::upscaler::dx12::interop::CreationCallbacks) { let _ = s.create_with_callbacks(c,callbacks); }
fn main() {}''', 'E0133')
    add('movable_setup_requires_unsafe', 'fn probe(r: &mut api::MovableRuntime,d: &ID3D12Device) { let _ = r.upscaler_dx12(d); }\nfn main() {}', 'E0133')
    add('setup_cannot_escape_runtime', '''fn escape(d: &ID3D12Device) -> api::UpscalerSetup<'static, '_> {
    let mut runtime = unsafe { api::Runtime::load("fixture.dll") }.unwrap();
    unsafe { runtime.upscaler_dx12(d) }
} fn main() {}''', 'E0515')
    return cases


def prepare(directory: Path) -> tuple[Path, list[tuple[str, str, str | None]]]:
    candidate = directory / 'candidate'
    candidate.mkdir(parents=True)
    for name in ('Cargo.toml', 'Cargo.lock'):
        shutil.copy2(ROOT / name, candidate / name)
    for package in ('fsr-sdk', 'fsr-sdk-sys'):
        shutil.copytree(ROOT / 'crates' / package, candidate / 'crates' / package)
    inventory = []
    for source in sorted((candidate / 'crates').rglob('*')):
        if source.is_file():
            content = source.read_bytes()
            inventory.append({'path': str(source.relative_to(candidate)).replace('\\','/'), 'bytes': len(content), 'sha256': hashlib.sha256(content).hexdigest()})
    for name in ('Cargo.toml', 'Cargo.lock', 'docs/adr/D042.md', 'tools/experiments/upscaler_setup/setup.rs', 'tools/experiments/upscaler_setup/composition.rs', 'tools/experiments/upscaler_setup/controlled.rs', 'tools/experiments/upscaler_setup/run.py'):
        content = (ROOT / name).read_bytes()
        inventory.append({'path': name, 'bytes': len(content), 'sha256': hashlib.sha256(content).hexdigest()})
    write(directory / 'inputs.json', json.dumps(inventory, indent=2) + '\n')
    change(candidate/'Cargo.toml', '  "crates/fsr-sdk-sys",', '  "crates/fsr-sdk-sys",\n  "consumer",')
    with (candidate/'Cargo.toml').open('a',encoding='utf-8',newline='\n') as stream:
        stream.write('\n[profile.dev]\ndebug = 0\nincremental = false\n[profile.test]\ndebug = 0\nincremental = false\n')
    library = candidate/'crates/fsr-sdk/src'
    write(library/'composition_spike.rs', (HERE/'composition.rs').read_text(encoding='utf-8'))
    write(library/'runtime/setup_spike.rs', (HERE/'setup.rs').read_text(encoding='utf-8'))
    with (library/'lib.rs').open('a',encoding='utf-8',newline='\n') as stream:
        stream.write('\n/// Experimental portable composition; isolated candidate only.\npub mod composition_spike;\n')
    with (library/'runtime.rs').open('a',encoding='utf-8',newline='\n') as stream:
        stream.write('\n/// Experimental effect trust boundary; isolated candidate only.\npub mod setup_spike;\n')
    with (library/'runtime/movable.rs').open('a',encoding='utf-8',newline='\n') as stream:
        stream.write('\nimpl MovableRuntime { pub(super) fn spike_from_state(state: Arc<RuntimeState>) -> Self { Self { state, _not_sync: PhantomData } } }\n')
    # Reuse the exact sharpening implementation with a portable module identity.
    with (library/'upscaler/common.rs').open('a',encoding='utf-8',newline='\n') as stream:
        stream.write('\n#[path = "dx12/dispatch/sharpening.rs"]\nmod spike_sharpening;\npub use spike_sharpening::{NativeSharpness, Sharpening};\n')
    change(library/'upscaler/dx12/dispatch.rs','mod sharpening;','')
    change(library/'upscaler/dx12/dispatch.rs','pub use sharpening::{NativeSharpness, Sharpening};','pub use crate::upscaler::common::{NativeSharpness, Sharpening};')
    change(library/'upscaler/dx12/dispatch/sharpening.rs','super::UpscaleDispatch','crate::composition_spike::FrameParameters')
    tests = library/'upscaler/dx12/tests'
    write(tests/'setup_spike.rs', (HERE/'controlled.rs').read_text(encoding='utf-8'))
    with (tests/'mod.rs').open('a',encoding='utf-8',newline='\n') as stream:
        stream.write('\nmod setup_spike;\n')
    with (tests/'provider_selection.rs').open('a',encoding='utf-8',newline='\n') as stream:
        stream.write('''
pub(super) fn spike_register(observer: &libloading::Library, device: &ID3D12Device) { register(observer, device); }
pub(super) fn spike_available(value: u64) { AVAILABLE.set(value); }
pub(super) fn spike_live_id(value: u64) { LIVE_OVERRIDE.set(value); }
pub(super) fn spike_seen() -> Vec<(u64, Option<u64>)> { observed() }
''')
    return candidate, consumers(candidate)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--prepare-only', action='store_true')
    parser.add_argument('--toolchain', choices=('stable','1.88.0','both'), default='both')
    args = parser.parse_args()
    stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')
    directory = ROOT/'target/upscaler-setup-spike'/stamp
    candidate, cases = prepare(directory)
    print(f'Candidate: {candidate}', flush=True)
    if args.prepare_only:
        return 0
    results = []

    def execute(name: str, command: list[str], expected: str | None = None) -> bool:
        completed = subprocess.run(command,cwd=candidate,capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=180)
        output = completed.stdout + completed.stderr
        binary = command[command.index('--bin')+1] if '--bin' in command else None
        diagnostic = expected is None
        for line in completed.stdout.splitlines():
            try:
                item = json.loads(line)
            except json.JSONDecodeError:
                continue
            if item.get('reason') == 'compiler-message' and item.get('target',{}).get('name') == binary:
                code = (item.get('message',{}).get('code') or {}).get('code')
                if code == expected:
                    diagnostic = True
        passed = completed.returncode == 0 if expected is None else completed.returncode != 0 and diagnostic
        clean = output.replace(str(ROOT),'<repo>').replace(str(ROOT).replace('\\','/'),'<repo>')
        write(directory/f'{name}.log',clean)
        result = {'case':name,'command':[arg.replace(str(candidate),'<candidate>') for arg in command],
                  'exit':completed.returncode,'expected_diagnostic':expected,'passed':passed}
        results.append(result)
        write(directory/'results.json',json.dumps(results,indent=2)+'\n')
        print(f'{"PASS" if passed else "FAIL"} {name} (exit {completed.returncode})',flush=True)
        if not passed:
            print(clean[-7000:],flush=True)
        return passed

    if not execute('lockfile',['cargo','generate-lockfile','--offline']):
        return 1
    chains = ('stable','1.88.0') if args.toolchain=='both' else (args.toolchain,)
    for chain in chains:
        cargo = ['cargo','+'+chain]
        execute(f'{chain}-toolchain',['rustc','+'+chain,'-Vv'])
        for features in ('','dx12','dx12,movable'):
            name = features.replace(',','-') or 'portable'
            execute(f'{chain}-library-{name}',cargo+['check','-p','fsr-sdk','--no-default-features','--offline','--locked']+(['--features',features] if features else []))
        controlled = execute(f'{chain}-controlled',cargo+['test','-p','fsr-sdk','--lib','--features','movable','--offline','--locked','setup_spike','--','--nocapture'])
        execute(f'{chain}-portable-test',cargo+['test','-p','fsr-sdk','--lib','--no-default-features','--offline','--locked','setup_spike'])
        if chain == 'stable':
            execute('stable-clippy',cargo+['clippy','-p','fsr-sdk','--lib','--features','movable','--offline','--locked','--','-D','warnings'])
        if not controlled:
            break
        for name,features,expected in cases:
            execute(f'{chain}-consumer-{name}',cargo+['check','-p','setup-spike-consumer','--bin',name,'--offline','--locked','--message-format=json']+(['--features',features] if features else []),expected)
    print(f'Results: {directory / "results.json"}',flush=True)
    return 0 if all(item['passed'] for item in results) else 1


if __name__ == '__main__':
    sys.exit(main())
