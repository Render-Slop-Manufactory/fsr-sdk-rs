# SPDX-License-Identifier: MPL-2.0
"""Check symbol-name agreement across native equipment and its consumers."""
from pathlib import Path
import re
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from native import environment

ROOT = Path(__file__).resolve().parents[3]


class NativeEquipmentContract(unittest.TestCase):
    def test_gpu_helper_exports_match_rust_loads_and_cpp_imports(self):
        directory = ROOT / 'crates/fsr-sdk/tests/native'
        declarations = (directory / 'gpu.cpp').read_text(encoding='utf-8') + '\n' + '\n'.join(
            path.read_text(encoding='utf-8') for path in directory.glob('*.h'))
        exported = re.findall(r'extern "C" __declspec\(dllexport\)[^\n(]*\b(gpu_\w+)\s*\(', declarations)
        self.assertEqual(len(exported), len(set(exported)), 'Duplicate exported helper symbol')
        used = set()
        for path in (ROOT / 'crates/fsr-sdk').rglob('*.rs'):
            used.update(re.findall(r'(?:b|c)"(gpu_\w+)(?:\\0)?"', path.read_text(encoding='utf-8')))
        imported = set(re.findall(r'__declspec\(dllimport\)[^\n(]*\b(gpu_\w+)\s*\(', declarations))
        self.assertTrue(exported, 'No exported GPU helper symbols found')
        # The old upload symbol remains available to previously built probes.
        compatibility_exports = {'gpu_set_colour_signal': 'gpu_set_color_signal'}
        self.assertEqual(set(exported), used | compatibility_exports.keys())
        self.assertLessEqual(set(compatibility_exports.values()), used)
        self.assertTrue(imported)
        self.assertLessEqual(imported, set(exported))

    def test_shared_device_and_gpu_equipment_use_the_same_new_library(self):
        class Run:
            state = {'native_stage': Path('native')}
            class options:
                adapter_index = None
        selected = environment(Run())
        self.assertEqual(selected['FSR_SDK_TEST_GPU_DLL'], Path('native/gpu_helper.dll'))
        self.assertEqual(selected['FSR_SDK_TEST_DEVICE_DLL'], selected['FSR_SDK_TEST_GPU_DLL'])
