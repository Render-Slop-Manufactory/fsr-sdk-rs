# SPDX-License-Identifier: MPL-2.0
"""Inference and evidence controls must reject attractive but invalid results."""
import copy
import contextlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from execution import Rejected
from performance_characterization import (ARMS, ROOT, estimate, load_selection,
    protocol, run_batch, schedule, selection, summarize, telemetry, t_quantile)
from records import inventory, save, source_identity
from profiles import compose
from test_active_timing import log as active_log
from test_feature_comparison import log as feature_log
from active_timing import parse as parse_active
from feature_comparison import parse as parse_feature


class PerformanceCharacterization(unittest.TestCase):
    def test_student_quantiles_against_independent_tabulated_values(self):
        for df, p, value in ((1,.95,6.3137515148), (9,.95,1.8331129327),
                             (23,.975,2.0686576104), (119,.95,1.6577592849)):
            self.assertAlmostEqual(t_quantile(p, df), value, places=7)
        for df, p in ((0,.95),(10,1),(10,.4)):
            with self.assertRaises(ValueError): t_quantile(p,df)

    def test_three_verdicts_and_negative_costs(self):
        self.assertEqual(estimate([1000,2000,3000]*8,5000)['verdict'],'within-budget')
        self.assertEqual(estimate([6000,7000,8000]*8,5000)['verdict'],'exceeds-budget')
        self.assertEqual(estimate([4000,5000,6000]*8,5000)['verdict'],'inconclusive')
        self.assertEqual(estimate([-3000,-1000]*12,5000)['verdict'],'within-budget')
        self.assertEqual(estimate([5000]*24,5000)['verdict'],'inconclusive')
        for values in ([1], [1,float('nan')], [1,float('inf')]):
            with self.assertRaises(Rejected): estimate(values,5000)

    def test_bounds_use_blocks_and_retain_outliers(self):
        result=estimate([0,0,0,100000],5000)
        self.assertEqual(result['blocks'],4)
        self.assertEqual(result['mean_difference_ns'],25000)
        self.assertGreater(result['upper_one_sided_95_ns'],5000)
        self.assertGreater(result['two_sided_95_ns'][1],result['upper_one_sided_95_ns'])

    def test_schedule_is_reproducible_balanced_and_randomized(self):
        orders=schedule(10,100)
        self.assertEqual(orders,schedule(10,100))
        self.assertNotEqual(orders,schedule(10,101))
        for mode in protocol()['workloads']:
            self.assertEqual(sum(next(arm for item,arm in order if item==mode)==ARMS[0] for order in orders),5)
        for order in orders:
            self.assertEqual(len(set(order)),10)
            for i in range(0,10,2): self.assertEqual(order[i][0],order[i+1][0])

    def test_telemetry_rejects_missing_extra_and_cross_mode_instrumentation(self):
        base='CHAR_MODE timing\n'
        self.assertEqual(telemetry(base,'timing','cpp-minimal'),{})
        for bad in ('',base+base,base+'CHAR_ALLOCATIONS 0 0 0 0 0 0\n',base+'CHAR_MODE allocation\n'):
            with self.assertRaises(Rejected): telemetry(bad,'timing','cpp-minimal')
        with self.assertRaises(Rejected): telemetry('CHAR_MODE raw\n','raw','rust-wrapper')

    def test_allocation_events_bytes_and_preparation_require_complete_matrices(self):
        rows=''.join(f'CHAR_ALLOCATIONS {i} 1 1 0 0 64\n' for i in range(160))
        result=telemetry('CHAR_MODE allocations\n'+rows,'allocations','rust-wrapper')
        self.assertEqual(result['allocations'][64],[1,1,0,0,64])
        for bad in (rows.replace('CHAR_ALLOCATIONS 64 1 1 0 0 64\n',''), rows+'CHAR_ALLOCATIONS 64 1 1 0 0 64\n',
                    rows.replace(' 64\n',' -1\n')):
            with self.assertRaises(Rejected): telemetry('CHAR_MODE allocations\n'+bad,'allocations','rust-wrapper')
        prep='CHAR_MODE preparation\n'+''.join(f'CHAR_PREPARATION {i} {i}\n' for i in range(160))
        self.assertEqual(telemetry(prep,'preparation','rust-wrapper')['preparation_ticks'][-1],159)

    def test_lifecycle_distinguishes_used_unused_and_requires_warm_samples(self):
        rows='CHAR_MODE lifecycle\n'
        rows+=''.join(f'CHAR_LIFECYCLE {name} 0 100\n' for name in ('load','provider_query','first_create','used_destroy'))
        rows+=''.join(f'CHAR_LIFECYCLE {name} {i} 10\n' for name in ('warm_create','unused_destroy') for i in range(8))
        self.assertEqual(len(telemetry(rows,'lifecycle','cpp-minimal')['lifecycle_ticks']['warm_create']),8)
        with self.assertRaises(Rejected): telemetry(rows.replace('CHAR_LIFECYCLE warm_create 7 10\n',''),'lifecycle','cpp-minimal')

    def test_matched_workloads_and_transition_audits_cover_every_block(self):
        children={}
        for mode in protocol()['workloads']:
            parsed=(parse_feature(feature_log(mode),arm='cpp-minimal',feature=mode) if mode in ('masks','hdr') else
                    parse_active(active_log(mode),arm='cpp-minimal',mode=mode))
            parsed['output_sha256']='a'*64
            for block in range(1,7):
                for arm in ARMS:
                    child=copy.deepcopy(parsed)
                    if arm=='rust-wrapper': child['ticks']['frame']=[x+10 for x in child['ticks']['frame']]
                    children[f'characterization.block{block}.{mode}.{arm}']=child
        result=summarize(children,6,'confirmation')
        self.assertEqual(result['overall_cpu_verdict'],'within-budget')
        self.assertAlmostEqual(result['findings']['hdr']['all']['cpu']['mean_difference_ns'],1000)
        self.assertIsNone(result['findings']['hdr']['all']['gpu']['verdict'])
        children['characterization.block6.transition.rust-wrapper']['checkpoints'][0][5]+=1
        with self.assertRaises(Rejected): summarize(children,6,'confirmation')

    def test_pilot_selection_freezes_budget_count_sources_and_schedule(self):
        with tempfile.TemporaryDirectory() as name:
            root=Path(name); pilot=root/'pilot'; pilot.mkdir(); (pilot/'analysis').mkdir()
            save(pilot/'run.json',{'profile':'experiment-performance-pilot','status':'passed','inputs_unchanged':True})
            identity=source_identity(ROOT)
            save(pilot/'inputs.json',identity)
            findings={mode:{'all':{'cpu':estimate([1000,2000]*5)}} for mode in protocol()['workloads']}
            save(pilot/'analysis/performance-characterization.json',{'phase':'pilot','protocol':protocol(),
                 'measurement_seconds':100,'findings':findings,'equipment':{}})
            save(pilot/'artifacts.json',inventory(pilot))
            path=root/'selection.json'
            plan=selection(pilot,path)
            self.assertEqual(plan['blocks'],24)
            self.assertTrue(plan['planned_precision_feasible'])
            self.assertEqual(load_selection(path)['cpu_budget_ns'],5000)
            known=set()
            cases=compose('experiment-performance-confirm',performance_plan=path)
            for case in cases:
                self.assertNotIn(case.id,known)
                self.assertTrue(set(case.requires)<=known)
                known.add(case.id)
            self.assertEqual(sum('.block' in case.id for case in cases),240)
            self.assertEqual(sum('.lifecycle' in case.id for case in cases),32)
            self.assertEqual(sum('.supplement' in case.id and case.id != 'characterization.supplements' for case in cases),45)
            with self.assertRaises(FileExistsError): selection(pilot,path)
            for key,value in (('blocks',25),('cpu_budget_ns',10000),('producer_identity','wrong'),('schedule',[])):
                invalid=plan | {key:value}; save(root/'invalid.json',invalid)
                with self.assertRaises(Rejected): load_selection(root/'invalid.json')

    def test_profiles_are_opt_in_and_confirmation_requires_selection(self):
        cases=compose('experiment-performance-pilot')
        ids=set()
        for case in cases:
            self.assertTrue(set(case.requires)<=ids)
            self.assertNotIn(case.id,ids)
            ids.add(case.id)
        self.assertEqual(sum('.block' in case.id for case in cases),100)
        with self.assertRaises(ValueError): compose('experiment-performance-confirm')
        self.assertFalse(any(case.id.startswith('characterization.') for case in compose('fast')))


class UnattendedBatch(unittest.TestCase):
    def exercise(self, failed_stage=None):
        with tempfile.TemporaryDirectory() as temporary:
            directory=Path(temporary)
            calls=[]

            def profile(arguments):
                name=arguments[1]
                calls.append(name)
                self.assertIn('--require-passed',arguments)
                self.assertIn('--native',arguments)
                root=Path(arguments[arguments.index('--output-root')+1])
                run=root/('test-'+name+'-attempt')
                run.mkdir(parents=True)
                save(run/'run.json',{'status':'failed' if failed_stage==name else 'passed'})
                if name.endswith('-confirm'):
                    plan=Path(arguments[arguments.index('--performance-plan')+1])
                    self.assertEqual(json.loads(plan.read_text())['blocks'],110)
                return int(failed_stage==name)

            def select(pilot, output):
                self.assertEqual(json.loads((pilot/'run.json').read_text())['status'],'passed')
                if failed_stage=='selection': raise Rejected('Selection rejected')
                plan={'blocks':110,'predicted_measurement_seconds':7166}
                save(output,plan)
                return plan

            with patch('run.main',side_effect=profile), patch('performance_characterization.selection',side_effect=select) as selected:
                with contextlib.redirect_stdout(io.StringIO()):
                    if failed_stage:
                        with self.assertRaises(Rejected): run_batch('sdk',0,directory)
                    else: run_batch('sdk',0,directory)
            status=json.loads(next(directory.glob('*/status.json')).read_text())
            return status,calls,selected.call_count

    def test_runs_pilot_selection_and_confirmation_and_records_results(self):
        status,calls,selected=self.exercise()
        self.assertEqual(calls,['experiment-performance-pilot','experiment-performance-confirm'])
        self.assertEqual(selected,1)
        self.assertEqual(status['status'],'completed')
        self.assertEqual(status['selection']['blocks'],110)
        self.assertTrue(status['confirmation_analysis'].endswith('performance-characterization.json'))

    def test_failed_pilot_stops_before_selection_or_confirmation(self):
        status,calls,selected=self.exercise('experiment-performance-pilot')
        self.assertEqual(calls,['experiment-performance-pilot'])
        self.assertEqual(selected,0)
        self.assertEqual(status['status'],'failed')

    def test_selection_or_confirmation_failure_cannot_look_completed(self):
        for stage in ('selection','experiment-performance-confirm'):
            with self.subTest(stage=stage):
                status,calls,selected=self.exercise(stage)
                self.assertEqual(status['status'],'failed')
                self.assertEqual(len(calls),1 if stage=='selection' else 2)
                self.assertEqual(selected,1)
                self.assertIn('reason',status)


if __name__=='__main__': unittest.main()
