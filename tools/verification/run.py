# SPDX-License-Identifier: MPL-2.0
"""One entry point for current verification; historical records are read-only."""
import argparse
import json
import os
from pathlib import Path
import sys

from execution import Blocked, Rejected, Runner
from profiles import PROFILES, compose
from records import export, fresh, inventory, save, verify_artifacts
from retention import CALIBRATION_PROFILES

ROOT=Path(__file__).resolve().parents[2]


def parser():
    result=argparse.ArgumentParser(description=__doc__)
    sub=result.add_subparsers(dest='command',required=True)
    sub.add_parser('list',help='List current profiles and their claims')
    runtime=sub.add_parser('runtime',help='Explicit local Proton/runtime import; no downloads')
    actions=runtime.add_subparsers(dest='runtime_command',required=True)
    prepare=actions.add_parser('prepare',help='Copy prepared local Proton/steamrt3 and freeze their identity')
    for name in ('proton','runtime','umu','launcher-root','output','provenance'):
        prepare.add_argument('--'+name,required=True)
    verify=actions.add_parser('verify',help='Verify a checksum-selected prepared runtime manifest')
    verify.add_argument('--manifest',required=True)
    verify.add_argument('--sha256',required=True)
    sdk=sub.add_parser('sdk',help='Explicit pinned maintainer SDK acquisition and verification')
    actions=sdk.add_subparsers(dest='sdk_command',required=True)
    for name in ('install','verify'):
        action=actions.add_parser(name)
        action.add_argument('--root',help='Version installation directory (parent of Kits/FidelityFX)')
        action.add_argument('--strict',action='store_true',help='Require Windows Authenticode, timestamp and online revocation checks')
        if name=='install':
            action.add_argument('--archive',help='Local pinned prebuilt ZIP; otherwise download explicitly')
            action.add_argument('--source-root',help='Local upstream source tree with the four pinned headers and two notices; otherwise download them')
    ci=sub.add_parser('export-ci',help='Export all finalized CI attempts, including failures')
    ci.add_argument('--output',required=True)
    plan=sub.add_parser('plan',help='List every required case before running')
    plan.add_argument('profile',choices=PROFILES)
    plan.add_argument('--artifacts', action='store_true', help='Plan supported native/Proton checks against Cargo archives')
    plan.add_argument('--performance-plan',help='Frozen native performance confirmation selection')
    run=sub.add_parser('run',help='Execute a fresh attempt, retaining every outcome')
    run.add_argument('profile',choices=PROFILES)
    run.add_argument('--require-passed',action='store_true',help='Return nonzero unless the finalized run status is passed; diagnostic observations remain recorded as observed')
    run.add_argument('--output-root',default=str(ROOT/'target/verification'))
    run.add_argument('--online',action='store_true',help='Permit Cargo network access and pinned ABI header acquisition')
    run.add_argument('--toolchain',help='Cargo/rustc toolchain override (MSRV cases always use 1.88.0)')
    run.add_argument('--native',action='store_true',help='Opt in to local AMD GPU execution')
    run.add_argument('--sdk-root',help='Kits/FidelityFX directory with pinned headers and signedbin DLLs')
    run.add_argument('--adapter-index',type=int,help='Explicit DXGI adapter index; default first suitable hardware adapter')
    run.add_argument('--performance-plan',help='Frozen native performance confirmation selection from a passed pilot')
    run.add_argument('--runtime-manifest',help='Explicit prepared offline-runtime manifest')
    run.add_argument('--runtime-sha256',help='Expected SHA-256 of the offline-runtime manifest')
    run.add_argument('--retain-diagnostics',action='store_true',help='Keep full local calibration images and analyses for a specific diagnosis; exports remain compact')
    run.add_argument('--artifacts', action='store_true', help='Use selected archives (or package the checkout) for supported native/Proton profiles')
    run.add_argument('--wrapper-archive',help='Select exact fsr-sdk .crate bytes instead of generating archives')
    run.add_argument('--sys-archive',help='Select exact fsr-sdk-sys .crate bytes; supply together with wrapper archive')
    from proton import arguments as proton_arguments
    proton_arguments(run)
    for name in ('export','analyze'):
        action=sub.add_parser(name,help='Explicit exclusive output; never modify the original attempt')
        action.add_argument('--run',required=True)
        action.add_argument('--output',required=True)
        if name=='analyze':
            action.add_argument('--namespace',help='Sharpening integration namespace (native or artifact); otherwise characterize all retained red readbacks')
            action.add_argument('--original-sharpening',action='store_true',help='Recompute the original D014 metrics, including failed children')
    verify=sub.add_parser('verify-run',help='Verify complete artefact inventory and required case records')
    verify.add_argument('--run',required=True)
    return result


def verify_run(path):
    path=Path(path)
    verify_artifacts(path)
    summary=json.loads((path/'run.json').read_text())
    plan=json.loads((path/'plan.json').read_text())
    required=[case['id'] for case in plan['required_cases']]
    if required!=summary['required'] or len(set(required))!=len(required): raise ValueError('Required case inventory mismatch')
    if summary['status']=='running': raise ValueError('Attempt was not finalized')
    if set(summary['cases']) != set(required) or summary['completed'] != len(required):
        raise ValueError('Missing or extra required outcomes')
    for name in required:
        record=json.loads((path/'cases'/name/'result.json').read_text())
        if record['id']!=name or record['evaluation']!=summary['cases'].get(name): raise ValueError('Case result mismatch: '+name)
    if summary['status']=='passed' and (not summary['inputs_unchanged'] or any(value!='passed' for value in summary['cases'].values())):
        raise ValueError('Pass claim contradicts required outcomes')
    return summary


def main(argv=None):
    options=parser().parse_args(argv)
    try:
        if options.command=='runtime':
            from proton_runtime import prepare, verify
            if options.runtime_command=='prepare':
                result=prepare(options.proton,options.runtime,options.umu,options.launcher_root,
                               options.output,options.provenance)
            else:
                _,_,result=verify(options.manifest,options.sha256)
            print(json.dumps(result,indent=2))
        elif options.command=='sdk':
            from sdk import DEFAULT_ROOT, install, verify
            root=Path(options.root) if options.root else DEFAULT_ROOT
            result=install(root,options.archive,options.source_root,options.strict) if options.sdk_command=='install' else verify(root,options.strict)
            print(json.dumps(result,indent=2))
        elif options.command=='list':
            for name,claim in PROFILES.items(): print(name+': '+claim)
        elif options.command=='export-ci':
            destination=Path(options.output).resolve()
            destination.mkdir(parents=True,exist_ok=False)
            for path in sorted((ROOT/'target/verification').glob('*/run.json')):
                verify_run(path.parent)
                export(path.parent,destination/path.parent.name,ROOT)
            print(destination)
        elif options.command=='plan':
            print(json.dumps({'profile':options.profile,'required_cases':[case.plan() for case in compose(options.profile, performance_plan=options.performance_plan, artifacts=options.artifacts)],'resume':'disabled'},indent=2))
        elif options.command=='run':
            if options.adapter_index is not None and options.adapter_index<0: raise ValueError('Adapter index must be nonnegative')
            if options.retain_diagnostics and options.profile not in CALIBRATION_PROFILES:
                raise ValueError('--retain-diagnostics applies only to temporal experiments')
            directory=fresh(options.output_root,options.profile)
            print('Attempt: '+str(directory),flush=True)
            runner=Runner(ROOT,directory,options)
            # Cargo caches compiled artefacts only. No case result is ever reused.
            runner.env['CARGO_TARGET_DIR']=str(ROOT/'target/verification-build'/ (options.toolchain or 'current'))
            extra=[]
            if options.sdk_root:
                manifest=json.loads((ROOT/'tools/native-headers.json').read_text())
                for item in manifest['headers']:
                    path=Path(options.sdk_root)/item['path']
                    if path.is_file(): extra.append(('sdk/'+item['path'],path))
                for path in sorted((Path(options.sdk_root)/'signedbin').glob('*.dll')):
                    if path.name.startswith('amd_fidelityfx_'): extra.append(('sdk/signedbin/'+path.name,path))
            for name in ('wrapper_archive','sys_archive'):
                value=getattr(options,name)
                if value: extra.append((name,Path(value)))
            if options.performance_plan:
                if options.profile not in ('experiment-performance-confirm', 'release-performance-reconfirm'):
                    raise ValueError('--performance-plan applies only to a performance confirmation profile')
                extra.append(('performance-plan', Path(options.performance_plan)))
            runner.state['external_inputs']=extra
            if options.profile in ('proton','proton-offline','proton-timeout') or options.profile.startswith('proton-temporal-'):
                from proton import external_inputs
                runner.state['external_inputs']+=external_inputs(options)
            if options.runtime_manifest and Path(options.runtime_manifest).is_file():
                runner.state['external_inputs'].append(('offline-runtime-manifest',Path(options.runtime_manifest)))
            code=runner.execute(compose(options.profile, performance_plan=options.performance_plan, artifacts=(options.artifacts or options.wrapper_archive is not None or options.sys_archive is not None)),options.profile,require_passed=options.require_passed)
            if options.profile in CALIBRATION_PROFILES and not options.retain_diagnostics:
                from retention import compact_calibration
                compact_calibration(directory)
            print('Result: '+str(directory/'run.json'),flush=True)
            return code
        elif options.command=='verify-run':
            print(json.dumps(verify_run(options.run),indent=2))
        elif options.command=='export':
            verify_run(options.run)
            print(export(options.run,options.output,ROOT))
        else:
            from analysis import audit_sharpening_integration, audit_sharpening, characterize
            verify_run(options.run)
            source,destination=Path(options.run).resolve(),Path(options.output).resolve()
            if source.is_relative_to(destination) or destination.is_relative_to(source): raise ValueError('Analysis output overlaps original attempt')
            destination.mkdir(parents=True,exist_ok=False)
            try:
                result=audit_sharpening(source) if options.original_sharpening else audit_sharpening_integration(source,options.namespace) if options.namespace else characterize(source)
                save(destination/'analysis.json',result)
            except Exception as error:
                save(destination/'analysis.json',{'status':'failed','reason':str(error)})
                raise
            finally: save(destination/'artifacts.json',inventory(destination))
            print(destination/'analysis.json')
        return 0
    except (Blocked,Rejected) as error:
        print('Verification rejected: '+str(error),file=sys.stderr)
        return 1
    except (ValueError,OSError,KeyError) as error:
        print('Verification error: '+str(error),file=sys.stderr)
        return 2


if __name__=='__main__':
    sys.exit(main())
