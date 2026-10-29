# SPDX-License-Identifier: MPL-2.0
"""Explicit Vulkan core/synchronization diagnostics for translated GPU cases."""
import json
from pathlib import Path
import re

from execution import Blocked, Rejected

LAYER = 'VK_LAYER_KHRONOS_validation'


def inputs(value):
    manifest = Path(value).expanduser().resolve()
    try:
        layer = json.loads(manifest.read_text())['layer']
        library = Path(layer['library_path'])
        if not library.is_absolute():
            library = manifest.parent / library
        library = library.resolve(strict=True)
        if layer['name'] != LAYER or not library.is_file():
            raise ValueError('Incorrect layer or missing library')
    except (OSError, ValueError, KeyError, TypeError) as error:
        raise Blocked('Invalid Vulkan validation manifest/library: ' + str(error)) from error
    return manifest, library


def environment(manifest, directory):
    logfile = directory / 'validation.log'
    settings = directory / 'vk_layer_settings.txt'
    values = {
        'validate_core': 'true', 'validate_sync': 'true',
        'syncval_submit_time_validation': 'true', 'gpuav_enable': 'false',
        'report_flags': 'info,warn,error', 'enable_message_limit': 'false',
        'message_id_filter': '', 'debug_action': 'VK_DBG_LAYER_ACTION_LOG_MSG',
        # A file opened by each VkInstance can overwrite earlier diagnostics.
        # stdout is captured for the entire child lifetime by Proton instead.
        'log_filename': 'stdout',
    }
    settings.write_text(''.join(f'khronos_validation.{key} = {value}\n' for key, value in values.items()))
    return {'VK_LAYER_PATH': str(manifest.parent), 'VK_INSTANCE_LAYERS': LAYER,
            # The layer activates independently. New VKD3D's vk_debug mode
            # deliberately submits an invalid pNext as a callback self-test.
            # Do not inject that error or filter any observed validation error.
            'VK_LAYER_SETTINGS_PATH': str(settings), 'VKD3D_CONFIG': '',
            'VKD3D_DEBUG': 'info', 'VK_LOADER_DEBUG': 'layer'}, logfile


def observation(logfile):
    text = logfile.read_text(errors='replace') if logfile.is_file() else ''
    enabled = re.findall(r'\[ CURRENT-VALIDATION-ENABLED \](.*?)\n\n', text, re.DOTALL)
    return {
        'active': bool(enabled) and all('  - Core Checks' in block for block in enabled),
        'synchronization': bool(enabled) and all('  - Synchronization' in block for block in enabled),
        'instances': len(enabled),
        'errors': len(re.findall(r'Validation Error:', text)),
        'warnings': len(re.findall(r'Validation Warning:', text)),
        'log': str(logfile),
    }


def require(result):
    if not result['active'] or not result['synchronization']:
        raise Rejected('Vulkan core/synchronization validation activation was not observed')
    if result['errors']:
        raise Rejected(f'Vulkan validation reported {result["errors"]} errors')
