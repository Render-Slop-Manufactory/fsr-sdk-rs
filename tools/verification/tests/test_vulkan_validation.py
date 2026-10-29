# SPDX-License-Identifier: MPL-2.0
"""Validation cannot pass when inactive, filtered by inheritance, or erroneous."""
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from execution import Blocked, Rejected
import proton
import vulkan_validation as validation


class VulkanValidation(unittest.TestCase):
    def test_manifest_requires_correct_layer_and_existing_library(self):
        with tempfile.TemporaryDirectory() as directory:
            # Windows temporary directories can contain 8.3 aliases.
            root = Path(directory).resolve()
            manifest = root / 'layer.json'
            with self.assertRaises(Blocked):
                validation.inputs(manifest)
            library = root / 'layer.so'
            library.write_bytes(b'controlled placeholder')
            for name in ('other-layer', validation.LAYER):
                manifest.write_text(json.dumps({'layer': {'name': name, 'library_path': 'layer.so'}}))
                if name == validation.LAYER:
                    self.assertEqual(validation.inputs(manifest), (manifest, library))
                else:
                    with self.assertRaises(Blocked):
                        validation.inputs(manifest)
            library.unlink()
            with self.assertRaises(Blocked):
                validation.inputs(manifest)

    def test_absent_activation_sync_and_error_fail_while_warning_is_recorded(self):
        active = 'Validation Information: [ CURRENT-VALIDATION-ENABLED ]\nvkCreateInstance(): Current Validaiton Enabled:\n  - Core Checks\n'
        sync = '  - Synchronization\n\n'
        with tempfile.TemporaryDirectory() as directory:
            logfile = Path(directory) / 'validation.log'
            for text in ('', active + '\n', active + sync + 'Validation Error: [ VUID-test ]\n',
                         active + sync + active + '\n'):
                logfile.write_text(text)
                with self.assertRaises(Rejected):
                    validation.require(validation.observation(logfile))
            logfile.write_text(active + sync + 'Validation Warning: [ warning-test ]\n')
            result = validation.observation(logfile)
            validation.require(result)
            self.assertEqual(result['warnings'], 1)
            self.assertEqual(result['errors'], 0)

    def test_inherited_filters_and_disables_are_removed_but_driver_is_preserved(self):
        env = {key: 'bad' for key in ('VK_LAYER_PATH', 'VK_INSTANCE_LAYERS', 'VK_LOADER_LAYERS_DISABLE',
                                      'VK_LAYER_SETTINGS_PATH', 'VK_LAYER_MESSAGE_ID_FILTER',
                                      'VK_VALIDATION_FEATURE_DISABLE_ALL_EXT', 'VVL_DISABLES')}
        env['VK_DRIVER_FILES'] = '/host/driver.json'
        self.assertEqual(proton.clean_environment(env), {'VK_DRIVER_FILES': '/host/driver.json'})

    def test_layer_activation_does_not_enable_vkd3d_error_injection_or_filters(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            controls, _ = validation.environment(root / 'layer.json', root)
            self.assertEqual(controls['VK_INSTANCE_LAYERS'], validation.LAYER)
            self.assertEqual(controls['VKD3D_CONFIG'], '')
            settings = (root / 'vk_layer_settings.txt').read_text()
            self.assertIn('validate_core = true', settings)
            self.assertIn('validate_sync = true', settings)
            self.assertIn('message_id_filter = \n', settings)
            self.assertIn('enable_message_limit = false', settings)


if __name__ == '__main__':
    unittest.main()
