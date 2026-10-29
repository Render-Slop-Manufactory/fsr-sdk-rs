# SPDX-License-Identifier: MPL-2.0
"""Acceptance tests for failure handling and immutable evidence boundaries."""
import argparse
import contextlib
import io
import json
from pathlib import Path
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from analysis import compare, red
from consumers import consumer_profile, registry_provenance, require_native_dependencies
from execution import Case, Runner, Rejected
from profiles import PROFILES, compose, infrastructure
from records import export, fresh, Sanitizer, verify_artifacts
from release import archive_files
from run import main, verify_run
from watchdog_fixture import ready_tree


class InfrastructureTests(unittest.TestCase):
    def setUp(self):
        self.temporary=tempfile.TemporaryDirectory()
        self.root=Path(self.temporary.name)
        self.repository=self.root/'repo'
        self.repository.mkdir()
        (self.repository/'Cargo.toml').write_text('[workspace]\n')
        options=argparse.Namespace(toolchain=None,online=False)
        self.runner=Runner(self.repository,fresh(self.root/'runs','test'),options)

    def tearDown(self):
        self.temporary.cleanup()

    def execute(self,cases):
        with contextlib.redirect_stdout(io.StringIO()):
            return self.runner.execute(cases,'test')

    def child(self,code,**kwargs):
        return lambda r:r.process([sys.executable,'-c',code],**kwargs)

    def test_provider_setup_timing_requires_native_staging_and_exact_completion(self):
        cases={case.id:case for case in compose('experiment-provider-setup')}
        sample=cases['provider-setup.queries']
        self.assertEqual(sample.scope,'native-query')
        self.assertEqual(sample.selector,'upscaler::dx12::tests::provider_setup::native_provider_setup_cost')
        self.assertEqual(sample.requires,('equipment.toolchain','provider-setup.build.fsr_sdk'))
        self.assertEqual(cases['provider-setup.build.fsr_sdk'].requires,('equipment.toolchain','native.helper'))
        self.assertIn('Provider setup timing: PASSED blocks=32 samples=64',sample.expectation['markers'])

    def test_infrastructure_failure_prints_bounded_sanitized_stderr_and_keeps_full_log(self):
        process = self.runner.process
        repository = str(self.repository.resolve())
        code = ('import sys; sys.stderr.write("x"*20000 + "\\nFAIL: controlled assertion " + '
                +repr(repository)+' + "\\n"); sys.exit(1)')
        output = io.StringIO()
        with patch.object(self.runner, 'process', side_effect=lambda args: process([sys.executable, '-c', code])), \
                contextlib.redirect_stderr(output):
            self.assertEqual(self.execute([infrastructure()[0]]), 1)
        self.assertIn('FAIL: controlled assertion <repo>', output.getvalue())
        self.assertNotIn(repository, output.getvalue())
        self.assertLess(len(output.getvalue()), 16500)
        record = self.runner.results['infrastructure.tests']
        self.assertEqual(record['evaluation'], 'failed')
        retained = self.runner.directory/record['commands'][0]['stderr']
        self.assertGreater(retained.stat().st_size, 20000)
        self.assertIn('FAIL: controlled assertion', retained.read_text(encoding='utf-8'))
        verify_run(self.runner.directory)

    def test_followup_source_failure_prevents_every_gpu_child(self):
        cases=compose('windows-upscaler-followup')
        executed=[]
        for case in cases:
            case.action=lambda r,name=case.id:executed.append(name)
        def fail(r):
            raise Rejected('Deliberate public-consumer failure')
        next(case for case in cases if case.id=='consumer.tuning-no-checked-constructor').action=fail
        self.assertEqual(self.execute(cases),1)
        self.assertIn('msrv.default.test',executed)
        self.assertEqual(self.runner.results['followup.source']['execution'],'blocked')
        for case in cases:
            if case.scope=='native-gpu':
                self.assertNotIn(case.id,executed)
                self.assertEqual(self.runner.results[case.id]['execution'],'blocked')

    def test_native_dependency_boundaries_reject_leaks_and_missing_dependencies(self):
        common = 'fsr-sdk v0.3.0\nfsr-sdk-sys v0.1.2\n'
        loader = 'libloading v0.9.0\n'
        windows = 'windows v0.62.2\n'
        require_native_dependencies(common, loader=False, windows=False)
        require_native_dependencies(common + loader, loader=True, windows=False)
        require_native_dependencies(common + loader + windows, loader=True, windows=True)
        for output, expected_loader, expected_windows in (
                (common + windows, False, False),
                (common + loader, False, False),
                (common + loader + windows, True, False),
                (common + loader, True, True),
                (common + windows, True, True)):
            with self.subTest(output=output, loader=expected_loader, windows=expected_windows):
                with self.assertRaises(Rejected):
                    require_native_dependencies(output, loader=expected_loader, windows=expected_windows)

    def test_reserved_vulkan_requires_completed_compiler_rejection(self):
        self.runner.state['consumer'] = {'packages': self.repository / 'crates', 'env': {}}
        cases = [case for case in consumer_profile() if '.unsupported-' in case.id]
        self.assertEqual({case.id for case in cases}, {
            f'consumer.unsupported-{config}.{package}'
            for config in ('vulkan', 'all-features') for package in ('fsr-sdk', 'fsr-sdk-sys')})
        diagnostic = 'error: the `vulkan` feature is intentionally unsupported'
        for case in cases:
            for execution, code, output, accepted in (
                    ('completed', 101, diagnostic, True),
                    ('completed', 0, diagnostic, False),
                    ('completed', 101, 'error: package does not contain feature `vulkan`', False),
                    ('completed', 101, 'error[E0432]: unrelated import failure', False),
                    ('timed_out', 1, diagnostic, False),
                    ('aborted', 1, diagnostic, False)):
                with self.subTest(case=case.id, execution=execution, code=code, output=output):
                    self.runner.current = {'evaluation': 'observed'}
                    command = {'execution': execution, 'exit_code': code}
                    with patch.object(self.runner, 'process', return_value=('', output, command)):
                        if accepted:
                            case.action(self.runner)
                            self.assertEqual(self.runner.current['evaluation'], 'passed')
                        else:
                            with self.assertRaises(Rejected):
                                case.action(self.runner)
        self.runner.current = None

    def test_deferred_features_are_selected_in_source_and_artifact_native_profiles(self):
        expected={
            'queries.utilities':'upscaler::dx12::tests::utility_queries::native_utility_queries',
            'dispatch.pre-exposure':'upscaler::dx12::native_dispatch::native_pre_exposure_dispatch',
            'dispatch.depth-motion':'upscaler::dx12::native_dispatch::native_depth_motion_dispatch',
            'dispatch.automatic-exposure':'upscaler::dx12::native_dispatch::native_automatic_exposure_dispatch',
            'dispatch.linear-hdr':'upscaler::dx12::native_dispatch::native_hdr_dispatch',
            'dispatch.supplied-masks':'upscaler::dx12::native_dispatch::native_supplied_masks_dispatch',
            'dispatch.active-render':'upscaler::dx12::native_dispatch::native_active_render_dispatch',
            'dispatch.configure':'upscaler::dx12::native_dispatch::native_configure_dispatch',
        }
        for profile,namespaces in [('native-upscaler',('native',)),('release',('native','artifact'))]:
            cases={case.id:case for case in compose(profile)}
            for namespace in namespaces:
                for name,selector in expected.items():
                    with self.subTest(profile=profile,namespace=namespace,name=name):
                        case=cases[namespace+'.'+name]
                        self.assertEqual(case.selector,selector)
                        self.assertTrue(case.expectation['markers'])
                        self.assertIn(namespace+'.queries.public',case.requires)
                        self.assertEqual(case.scope,'native-query' if name=='queries.utilities' else 'native-gpu')

    def test_expected_failure_requires_exact_diagnostic(self):
        cases=[Case('correct','compiler rejection',self.child("import sys; print('error[E0432]'); sys.exit(1)",diagnostic='E0432')),
               Case('wrong','compiler rejection',self.child("import sys; print('error[E0308]'); sys.exit(1)",diagnostic='E0432')),
               Case('success','compiler rejection',self.child("print('error[E0432]')",diagnostic='E0432'))]
        self.assertEqual(self.execute(cases),1)
        self.assertEqual([self.runner.results[key]['evaluation'] for key in ('correct','wrong','success')],['passed','failed','failed'])

    def test_expected_lint_requires_error_code_and_failure(self):
        def message(code='unused_must_use', level='error', reason='compiler-message'):
            return json.dumps({'reason': reason, 'message': {'level': level, 'code': {'code': code}}})
        cases = []
        for name, output, exit_code in (
            ('correct', message(), 1),
            ('wrong', message('dead_code'), 1),
            ('warning', message(level='warning'), 1),
            ('success', message(), 0),
            ('echo', '#![deny(unused_must_use)]', 1),
            ('not-compiler', message(reason='build-finished'), 1),
        ):
            code = f'import sys; print({output!r}); sys.exit({exit_code})'
            cases.append(Case(name, 'Exact compiler lint rejection', self.child(code, lint='unused_must_use')))
        self.assertEqual(self.execute(cases), 1)
        self.assertEqual([self.runner.results[case.id]['evaluation'] for case in cases],
            ['passed', 'failed', 'failed', 'failed', 'failed', 'failed'])

    def test_transfer_groups_keep_all_six_baselines_and_prerequisites(self):
        expected={
            'same_thread_load_create_dispatch_and_cleanup',
            'three_moving_frames_on_successive_threads',
            'load_create_then_dispatch_and_cleanup_on_other_threads',
            'same_thread_siblings_survive_runtime_release',
            'siblings_survive_runtime_release_and_thread_transfers',
            'same_thread_three_moving_frames',
        }
        for kind in ('transfer','transfer-cleanup'):
            namespace='experiment.'+kind
            cases=[case for case in compose('experiment-'+kind)
                   if case.selector.startswith('upscaler::dx12::native_dispatch::thread_transfer_baseline::')]
            self.assertEqual(len(cases),6)
            self.assertEqual({case.selector.rsplit('::',1)[1] for case in cases},expected)
            for case in cases:
                self.assertEqual(case.requires,('equipment.toolchain',)+tuple(namespace+'.'+name for name in
                                 ('dispatch.single','dispatch.temporal','dispatch.semantics')))
                self.assertEqual(case.scope,'native-gpu')
                self.assertIn('1 passed; 0 failed',case.expectation['markers'])

    def test_transfer_membership_uses_groups_independently_of_legacy_names(self):
        source=Path(__file__).resolve().parents[1]/'native-cases.json'
        definitions=json.loads(source.read_text())
        selected=[item for item in definitions if 'transfer' in item['groups']]
        for index,item in enumerate(selected):
            item['legacy_name']='renamed-baseline-'+str(index)
        # Membership can differ between the two profiles; a historical-looking
        # name with no groups must not select an unrelated test.
        selected[0]['groups']=['transfer-cleanup']
        definitions.append(dict(legacy_name='d013-unrelated',selector='unrelated',marker='unexpected',groups=[]))
        (self.root/'native-cases.json').write_text(json.dumps(definitions))
        with patch('native.HERE',self.root):
            for kind in ('transfer','transfer-cleanup'):
                cases=compose('experiment-'+kind)
                actual={case.selector for case in cases if '.renamed-baseline-' in case.id}
                expected={item['selector'] for item in selected if kind in item['groups']}
                self.assertEqual(actual,expected)
                self.assertFalse(any(case.selector=='unrelated' for case in cases))

    def test_missing_executable_blocks_dependents(self):
        cases=[Case('missing','missing equipment',lambda r:r.process([str(self.root/'absent.exe')])),
               Case('dependent','requires equipment',self.child("print('unexpected')"),('missing',))]
        self.assertEqual(self.execute(cases),1)
        self.assertEqual(self.runner.results['dependent']['execution'],'blocked')
        self.assertEqual(self.runner.results['dependent']['commands'],[])
        verify_run(self.runner.directory)

    def test_timeout_retained_and_not_a_pass(self):
        self.assertEqual(self.execute([Case('watchdog','bounded child',self.child('import time; time.sleep(30)',timeout=.2))]),1)
        record=self.runner.results['watchdog']
        self.assertEqual(record['commands'][0]['execution'],'timed_out')
        self.assertEqual(record['execution'],'timed_out')
        self.assertEqual(record['evaluation'],'failed')
        verify_artifacts(self.runner.directory)

    def test_observation_timeout_distinct_from_pass(self):
        self.assertEqual(self.execute([Case('observation','bounded observation',self.child('import time; time.sleep(30)',timeout=.2,observe=True))]),0)
        self.assertEqual(json.loads((self.runner.directory/'run.json').read_text())['status'],'observed')
        self.assertEqual(self.runner.results['observation']['execution'],'timed_out')

    def run_cli(self,cases,require_passed=False):
        output=fresh(self.root/'cli','strict' if require_passed else 'diagnostic')
        args=['run','fast','--output-root',str(output)]
        if require_passed: args+=['--require-passed']
        stdout,stderr=io.StringIO(),io.StringIO()
        with patch('run.ROOT',self.repository),patch('run.compose',return_value=cases), \
                contextlib.redirect_stdout(stdout),contextlib.redirect_stderr(stderr):
            code=main(args)
        directory,=output.iterdir()
        return code,directory,stdout.getvalue(),stderr.getvalue()

    def test_cli_require_passed_accepts_passed_run(self):
        code,directory,_,stderr=self.run_cli([Case('ok','accepted child',self.child("print('ok')"))],True)
        self.assertEqual(code,0,stderr)
        self.assertEqual(verify_run(directory)['status'],'passed')
        inputs=json.loads((directory/'inputs.json').read_text())
        self.assertTrue(inputs['configuration']['require_passed'])

    def test_cli_observed_timeout_only_succeeds_without_require_passed(self):
        for strict in (False,True):
            with self.subTest(require_passed=strict):
                case=Case('observation','bounded observation',self.child('import time; time.sleep(30)',timeout=.2,observe=True))
                code,directory,stdout,stderr=self.run_cli([case],strict)
                self.assertEqual(code,1 if strict else 0,stderr)
                summary=verify_run(directory)
                self.assertEqual(summary['status'],'observed')
                self.assertEqual(summary['cases'],{'observation':'observed'})
                result=json.loads((directory/'cases/observation/result.json').read_text())
                self.assertEqual(result['execution'],'timed_out')
                self.assertEqual(result['commands'][0]['execution'],'timed_out')
                if strict: self.assertIn('Acceptance requires passed status; recorded status: observed',stdout)

    def test_cli_require_passed_rejects_failed_blocked_and_interrupted_runs(self):
        def interrupted(r): raise KeyboardInterrupt()
        for name,action,evaluation in (
                ('failed',self.child('import sys; sys.exit(1)'),'failed'),
                ('blocked',lambda r:r.process([str(self.root/'absent.exe')]),'blocked'),
                ('interrupted',interrupted,'failed')):
            with self.subTest(outcome=name):
                code,directory,_,stderr=self.run_cli([Case(name,name,action)],True)
                self.assertEqual(code,1,stderr)
                summary=verify_run(directory)
                self.assertEqual(summary['status'],'failed')
                self.assertEqual(summary['cases'],{name:evaluation})

    def watchdog_tree(self, startup_delay=0, on_ready=None):
        heartbeat=self.root/'heartbeat'
        with ready_tree(heartbeat,startup_delay,on_ready) as tree:
            self.assertEqual(self.execute([Case('tree','bounded tree',lambda r:r.process(tree.command,timeout=.2))]),1)
            record=self.runner.results['tree']
            self.assertEqual(record['commands'][0]['execution'],'timed_out')
            self.assertEqual(record['execution'],'timed_out')
            self.assertEqual(record['evaluation'],'failed')
            tree.assert_stopped()
            self.assertGreater(len(heartbeat.read_bytes()),0)
            verify_run(self.runner.directory)

    def test_watchdog_stops_descendants(self):
        self.watchdog_tree()

    def test_watchdog_stops_descendants_after_delayed_startup(self):
        self.watchdog_tree(startup_delay=1.2)

    def test_watchdog_oracle_rejects_live_descendant(self):
        def live_control(tree):
            with self.assertRaisesRegex(AssertionError,'live descendant'):
                tree.assert_stopped(timeout=.2)
        self.watchdog_tree(on_ready=live_control)

    def test_watchdog_oracle_rejects_parent_only_termination(self):
        with ready_tree(self.root/'heartbeat') as tree:
            with patch.object(self.runner,'kill',side_effect=lambda process:process.kill()):
                self.assertEqual(self.execute([Case('tree','broken tree termination',
                    lambda r:r.process(tree.command,timeout=.2))]),1)
            self.assertEqual(self.runner.results['tree']['commands'][0]['execution'],'timed_out')
            with self.assertRaisesRegex(AssertionError,'live descendant'):
                tree.assert_stopped(timeout=.2)
        self.assertTrue(tree.stopped,'Negative-control descendant was not cleaned up')

    def test_watchdog_fixture_cleans_up_on_assertion_failure(self):
        heartbeat=self.root/'heartbeat'
        with self.assertRaisesRegex(AssertionError,'injected fixture failure'):
            with ready_tree(heartbeat) as tree:
                def fail_after_start(r):
                    r.process(tree.command,timeout=.2)
                def fail_on_ready(fixture):
                    raise AssertionError('injected fixture failure')
                tree.on_ready=fail_on_ready
                self.execute([Case('tree','startup failure',fail_after_start)])
                self.assertIn('injected fixture failure',self.runner.results['tree']['reason'])
                raise AssertionError('injected fixture failure')
        self.assertIsNotNone(tree.process.returncode)
        self.assertEqual(tree.process.returncode,0)
        self.assertTrue(tree.stopped)

    def test_watchdog_fixture_cleans_up_on_startup_failure(self):
        with ready_tree(self.root/'heartbeat',startup_delay='invalid') as tree:
            self.assertEqual(self.execute([Case('tree','failed startup',
                lambda r:r.process(tree.command,timeout=.2))]),1)
            self.assertEqual(self.runner.results['tree']['evaluation'],'failed')
        self.assertEqual(tree.process.returncode,0)
        self.assertTrue(tree.stopped)

    def test_required_marker_missing(self):
        self.assertEqual(self.execute([Case('child','exact child',self.child("print('0 passed; 0 failed')",markers=('1 passed; 0 failed',)))]),1)

    def test_source_mutation_invalidates_pass(self):
        self.assertEqual(self.execute([Case('mutation','source frozen',lambda r:(self.repository/'Cargo.toml').write_text('changed'))]),1)
        self.assertFalse(json.loads((self.runner.directory/'run.json').read_text())['inputs_unchanged'])

    def test_missing_required_record_rejected(self):
        self.execute([Case('ok','ordinary',self.child("print('ok')"))])
        (self.runner.directory/'cases/ok/result.json').unlink()
        with self.assertRaises(ValueError): verify_run(self.runner.directory)

    def test_exclusive_export_and_tamper_detection(self):
        self.execute([Case('ok','ordinary',self.child("print('ok')"))])
        output=self.root/'evidence'
        export(self.runner.directory,output,self.repository)
        verify_run(output)
        with self.assertRaises(FileExistsError): export(self.runner.directory,output,self.repository)
        (self.runner.directory/'cases/ok/result.json').write_text('{}')
        with self.assertRaises(ValueError): export(self.runner.directory,self.root/'bad',self.repository)
        self.assertFalse((self.root/'bad').exists())

    def test_sanitizer_preserves_codes_provider_ids_and_null(self):
        text=Sanitizer(self.repository).text('ptr=0x1234 provider_id=0xf5a5ca1e01001001 handle=0x0 return_code=4')
        self.assertIn('ptr=<pointer-1>',text)
        self.assertIn('provider_id=0xf5a5ca1e01001001',text)
        self.assertIn('handle=0x0 return_code=4',text)

    def test_sanitizer_normalizes_descriptor_storage_and_preserves_pointer_aliases(self):
        sanitizer=Sanitizer(self.repository)
        text=sanitizer.text('device=0x1234 pointer=0xabcd context=0xabcd\n'
                            'Dedicated descriptor storage: base=0xabcd; bytes=120\n'
                            'Destroy: return_code=0; after destroy: context=0xabcd\n'
                            'base=0x2000 handle=0x0')
        self.assertIn('device=<pointer-1> pointer=<pointer-2> context=<pointer-2>',text)
        self.assertIn('Dedicated descriptor storage: base=<pointer-2>; bytes=120',text)
        self.assertIn('after destroy: context=<pointer-2>',text)
        self.assertIn('base=0x2000 handle=0x0',text)
        self.assertEqual(sanitizer.value({'observation':'context=0xabcd'}),
                         {'observation':'context=<pointer-2>'})

    def test_malicious_archive_names_and_links_rejected(self):
        for name,kind in (('../escape',tarfile.REGTYPE),('pkg/../../escape',tarfile.REGTYPE),
                          ('C:/escape',tarfile.REGTYPE),('pkg/a\\b',tarfile.REGTYPE),
                          ('pkg/NUL.txt',tarfile.REGTYPE),('pkg/name. ',tarfile.REGTYPE),
                          ('pkg/link',tarfile.SYMTYPE),('pkg/dir',tarfile.DIRTYPE)):
            with self.subTest(name=name):
                path=self.root/'bad.crate'
                with tarfile.open(path,'w') as stream:
                    info=tarfile.TarInfo(name)
                    info.type=kind
                    stream.addfile(info)
                with self.assertRaises(Rejected): archive_files(path,'pkg')

    def test_independent_half_decoder_and_roi(self):
        import struct
        a,b=self.root/'a',self.root/'b'
        a.write_bytes(struct.pack('<16e',*([1.0]*16)))
        values=[1.0]*16
        values[5]=2.0
        b.write_bytes(struct.pack('<16e',*values))
        measured=compare(red(a,4,4),red(b,4,4),4,4,1)
        self.assertEqual(measured['roi_bit_mismatches'],1)
        self.assertEqual(measured['roi_mae_f64'],.25)
        b.write_bytes(b'bad')
        with self.assertRaises(ValueError): red(b,4,4)

    def test_all_profiles_have_complete_ordered_unique_requirements(self):
        for name in PROFILES:
            with self.subTest(profile=name):
                if name in ('experiment-performance-confirm', 'release-performance-reconfirm'):
                    # Its complete graph depends on an external frozen selection;
                    # the characterisation tests exercise a valid selected graph.
                    with self.assertRaisesRegex(ValueError, 'requires --performance-plan'):
                        compose(name)
                    continue
                known=set()
                for case in compose(name):
                    self.assertNotIn(case.id,known)
                    self.assertTrue(set(case.requires).issubset(known),case.id)
                    known.add(case.id)

    def test_registry_guard_includes_non_fsr_dependencies(self):
        home=self.root/'cargo-home'
        data={'workspace_members':['consumer'],'packages':[{'id':'consumer'},
            {'id':'dependency','name':'windows','version':'0.62.2','source':'registry+local',
             'manifest_path':str(home/'registry/windows/Cargo.toml'),'dependencies':[]}]}
        self.assertEqual(registry_provenance(data,home)[0]['name'],'windows')
        data['packages'][1]['source']=None
        with self.assertRaises(Rejected): registry_provenance(data,home)
        data['packages'][1]['source']='registry+local'
        data['packages'][1]['manifest_path']=str(self.repository/'Cargo.toml')
        with self.assertRaises(Rejected): registry_provenance(data,home)


if __name__=='__main__': unittest.main()
