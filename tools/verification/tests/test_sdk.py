# SPDX-License-Identifier: MPL-2.0
"""Small synthetic SDKs exercise acquisition without AMD binaries or a network."""
import contextlib
import hashlib
import http.client
import io
import json
import os
from pathlib import Path
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest
import warnings
from unittest.mock import Mock, patch
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import sdk
from native import header_sha
from run import main, parser


def digest(data):
    return hashlib.sha256(data).hexdigest()


class SdkTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.base = Path(self.temporary.name).resolve()
        self.root = self.base / 'installed'
        self.source = self.base / 'source'
        self.repository = self.base / 'repository'
        headers = []
        for name in ('api/include/ffx_api.h', 'api/include/ffx_api_types.h',
                     'api/include/dx12/ffx_api_dx12.h', 'upscalers/include/ffx_upscale.h'):
            data = ('synthetic ' + name + '\n').encode()
            path = self.source / sdk.SDK_PATH / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
            headers.append({'path': name, 'sha256': digest(data)})
        (self.repository / 'tools').mkdir(parents=True)
        (self.repository / 'tools/native-headers.json').write_text(json.dumps({'commit': 'a' * 40, 'headers': headers}))
        notices = {}
        for name in sdk.NOTICE_HASHES:
            data = ('synthetic notice ' + name + '\n').encode()
            path = self.source / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
            notices[name] = digest(data)
        self.dlls = {name: ('synthetic dll ' + name).encode() for name in sdk.DLL_HASHES}
        self.archive = self.base / 'sample.zip'
        with zipfile.ZipFile(self.archive, 'w', zipfile.ZIP_DEFLATED) as archive:
            for name, data in self.dlls.items():
                archive.writestr(sdk.DLL_PATH + name, data)
            archive.writestr('Samples/ignored.txt', b'sample is never extracted')
        for target, value in (('ROOT', self.repository), ('NOTICE_HASHES', notices),
                              ('DLL_HASHES', {name: digest(data) for name, data in self.dlls.items()}),
                              ('ARCHIVE_SHA', sdk.sha(self.archive)), ('WINDOWS', False)):
            mock = patch.object(sdk, target, value)
            mock.start()
            self.addCleanup(mock.stop)

    def install(self, **kwargs):
        return sdk.install(self.root, self.archive, self.source, **kwargs)

    def test_install_extracts_only_pinned_files_and_offline_verify_rehashes(self):
        result = self.install()
        self.assertEqual(result['integrity'], 'passed')
        self.assertEqual(set(sdk.verify_files(self.root)), set(sdk.file_pins()))
        self.assertFalse((self.root / 'Samples').exists())
        info = json.loads((self.root / sdk.METADATA).read_text())
        self.assertEqual(info['license_notices'], sdk.NOTICE_HASHES)
        self.assertNotIn('licence_notices', info)
        with patch.object(sdk, 'download', side_effect=AssertionError('network')), \
                patch.object(sdk, 'signatures', side_effect=AssertionError('trust network')):
            self.assertEqual(sdk.verify(self.root)['signatures']['status'], 'not_run')

    def test_idempotent_install_does_not_download_or_change_files(self):
        self.install()
        before = (self.root / sdk.METADATA).read_bytes()
        with patch.object(sdk, 'acquire_archive', side_effect=AssertionError('download')):
            sdk.install(self.root)
        self.assertEqual(before, (self.root / sdk.METADATA).read_bytes())

    def test_notice_provenance_aliases_verify_without_download_or_rewrite(self):
        self.install()
        path = self.root / sdk.METADATA
        info = json.loads(path.read_text())
        info.pop('license_notices')
        for names in (('license_notices',), ('licence_notices',), ('license_notices', 'licence_notices')):
            with self.subTest(names=names):
                sdk.save(path, dict(info, **{name: sdk.NOTICE_HASHES for name in names}))
                before = path.read_bytes()
                with patch.object(sdk, 'acquire_archive', side_effect=AssertionError('download')), \
                        patch.object(sdk, 'download', side_effect=AssertionError('network')), \
                        patch.object(sdk, 'signatures', side_effect=AssertionError('trust network')):
                    self.assertEqual(sdk.verify(self.root)['integrity'], 'passed')
                    self.assertEqual(sdk.install(self.root)['integrity'], 'passed')
                self.assertEqual(before, path.read_bytes())

    def test_missing_or_altered_notice_provenance_is_rejected(self):
        self.install()
        path = self.root / sdk.METADATA
        info = json.loads(path.read_text())
        info.pop('license_notices')
        changed = {name: '0' * 64 for name in sdk.NOTICE_HASHES}
        invalid = [{}] + [{name: value} for name in ('license_notices', 'licence_notices')
                          for value in (None, {}, 'not an inventory', changed)]
        invalid.append(dict(license_notices=changed, licence_notices=changed))
        for fields in invalid:
            with self.subTest(fields=fields):
                sdk.save(path, dict(info, **fields))
                with self.assertRaisesRegex(ValueError, 'Inconsistent SDK provenance: license_notices'):
                    sdk.verify(self.root)

    def test_conflicting_notice_aliases_are_rejected_without_rewrite(self):
        self.install()
        path = self.root / sdk.METADATA
        info = json.loads(path.read_text())
        for current, legacy in ((sdk.NOTICE_HASHES, {}), ({}, sdk.NOTICE_HASHES),
                                (None, sdk.NOTICE_HASHES), (sdk.NOTICE_HASHES, None)):
            with self.subTest(current=current, legacy=legacy):
                sdk.save(path, dict(info, license_notices=current, licence_notices=legacy))
                before = path.read_bytes()
                with self.assertRaisesRegex(ValueError, 'Conflicting SDK provenance'):
                    sdk.install(self.root)
                self.assertEqual(before, path.read_bytes())

    def test_legacy_notice_metadata_cannot_replace_pinned_file_hashes(self):
        self.install()
        path = self.root / sdk.METADATA
        info = json.loads(path.read_text())
        info['licence_notices'] = info.pop('license_notices')
        name = next(iter(sdk.NOTICE_HASHES))
        changed = b'changed notice'
        (self.root / name).write_bytes(changed)
        info['files'][name] = digest(changed)
        info['licence_notices'][name] = digest(changed)
        sdk.save(path, info)
        with self.assertRaisesRegex(ValueError, 'changed pinned SDK files'):
            sdk.verify(self.root)

    def test_bad_archive_hash_never_publishes(self):
        self.archive.write_bytes(b'not a trusted ZIP')
        with self.assertRaisesRegex(ValueError, 'Archive SHA-256 mismatch'):
            self.install()
        self.assertFalse(self.root.exists())
        self.assertEqual(list(self.base.glob('.installed.sdk-*')), [])

    def test_truncated_zip_with_matching_test_pin_is_rejected(self):
        self.archive.write_bytes(self.archive.read_bytes()[:-30])
        with patch.object(sdk, 'ARCHIVE_SHA', sdk.sha(self.archive)), self.assertRaisesRegex(ValueError, 'Invalid'):
            self.install()
        self.assertFalse(self.root.exists())

    def test_missing_dll_is_rejected(self):
        with zipfile.ZipFile(self.archive, 'w') as archive:
            archive.writestr('Samples/sample.txt', b'no DLLs')
        with patch.object(sdk, 'ARCHIVE_SHA', sdk.sha(self.archive)), self.assertRaisesRegex(ValueError, 'Missing required DLL'):
            self.install()

    def test_dll_baseline_is_not_adopted_from_archive(self):
        with patch.object(sdk, 'DLL_HASHES', {name: '0' * 64 for name in self.dlls}), \
                self.assertRaisesRegex(ValueError, 'DLL SHA-256 mismatch'):
            self.install()
        self.assertFalse(self.root.exists())

    def test_changed_header_or_notice_is_rejected(self):
        for name in ('docs/license.md', sdk.SDK_PATH + 'api/include/ffx_api.h'):
            with self.subTest(name=name):
                path = self.source / name
                original = path.read_bytes()
                path.write_bytes(b'changed')
                with self.assertRaisesRegex(ValueError, 'Pinned source SHA-256 mismatch'):
                    self.install()
                path.write_bytes(original)
                self.assertFalse(self.root.exists())

    def test_missing_source_is_rejected(self):
        (self.source / '3rdpartynotice.md').unlink()
        with self.assertRaises(OSError):
            self.install()
        self.assertFalse(self.root.exists())

    def test_corrupt_installation_is_not_repaired_even_with_forged_provenance(self):
        self.install()
        name = sdk.SDK_PATH + 'signedbin/' + next(iter(self.dlls))
        (self.root / name).write_bytes(b'corrupt')
        info = json.loads((self.root / sdk.METADATA).read_text())
        info['files'][name] = digest(b'corrupt')
        (self.root / sdk.METADATA).write_text(json.dumps(info))
        with self.assertRaisesRegex(ValueError, 'changed pinned'):
            self.install()
        self.assertEqual((self.root / name).read_bytes(), b'corrupt')

    def test_unmanaged_destination_is_preserved(self):
        self.root.mkdir()
        (self.root / 'user.txt').write_text('mine')
        with self.assertRaisesRegex(ValueError, 'Unexpected'):
            self.install()
        self.assertEqual((self.root / 'user.txt').read_text(), 'mine')

    def test_unexpected_installed_files_and_directories_fail(self):
        self.install()
        (self.root / 'extra').mkdir()
        with self.assertRaisesRegex(ValueError, 'Unexpected SDK directory'):
            sdk.verify(self.root)
        (self.root / 'extra').rmdir()
        (self.root / 'extra.dll').write_bytes(b'extra')
        with self.assertRaisesRegex(ValueError, 'Unexpected installed file'):
            sdk.verify(self.root)

    def test_provenance_does_not_replace_archive_pin(self):
        self.install()
        path = self.root / sdk.METADATA
        info = json.loads(path.read_text())
        info['archive_sha256'] = '0' * 64
        path.write_text(json.dumps(info))
        with self.assertRaisesRegex(ValueError, 'provenance'):
            sdk.verify(self.root)

    def test_interruption_cleans_stage_and_never_publishes(self):
        with patch.object(sdk, 'acquire_sources', side_effect=KeyboardInterrupt), self.assertRaises(KeyboardInterrupt):
            self.install()
        self.assertFalse(self.root.exists())
        self.assertEqual(list(self.base.glob('.installed*')), [])

    def test_lock_refuses_concurrent_or_abandoned_installation(self):
        with sdk.installation_lock(self.root), self.assertRaisesRegex(ValueError, 'busy or interrupted'):
            self.install()
        self.assertFalse(self.root.exists())

    @unittest.skipIf(sys.platform == 'win32', 'Portable symlink control without Windows privilege requirements')
    def test_symlink_destination_input_parent_and_installed_file_fail(self):
        self.root.symlink_to(self.source, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, 'Link/reparse'):
            self.install()
        self.root.unlink()
        link = self.base / 'link'
        link.symlink_to(self.source, target_is_directory=True)
        with self.assertRaisesRegex(ValueError, 'Link/reparse'):
            sdk.install(self.root, self.archive, link)
        self.install()
        path = self.root / '3rdpartynotice.md'
        path.unlink()
        path.symlink_to(self.source / '3rdpartynotice.md')
        with self.assertRaisesRegex(ValueError, 'Link/reparse'):
            sdk.verify(self.root)

    def test_header_normalization_contract_is_unchanged(self):
        path = self.base / 'header.h'
        path.write_bytes(b'header\r\nsecond\r\n')
        self.assertEqual(header_sha(path), digest(b'header\nsecond\n'))

    def zip_with(self, names):
        stream = io.BytesIO()
        with warnings.catch_warnings(), zipfile.ZipFile(stream, 'w') as archive:
            warnings.simplefilter('ignore', UserWarning)
            for name in names:
                entry = zipfile.ZipInfo(name)
                # Preserve deliberately unsafe names that ZipInfo normalizes on Windows.
                entry.filename = name
                archive.writestr(entry, b'')
        stream.seek(0)
        return zipfile.ZipFile(stream)

    def test_unsafe_zip_paths_including_ignored_members_are_rejected(self):
        for separator in ('/', '\\'):
            for name in ('', '.', './', '/', '../escape', '/absolute', 'C:/drive', '\\unc\\file', 'a/../b', 'a//b',
                         './a', 'a/./b', 'a\\b', 'a//', 'a/CON.txt', 'a/LPT1',
                         'a/trailing.', 'a/trailing ', 'a/\u00e9'):
                with self.subTest(separator=separator, name=name), patch('zipfile.os.sep', separator), \
                        self.zip_with([name]) as archive, self.assertRaisesRegex(ValueError, 'Unsafe'):
                    sdk.zip_members(archive)

    def test_zip_paths_use_posix_semantics_on_every_host(self):
        names = ('Samples/', 'Samples/ignored.txt', sdk.DLL_PATH + next(iter(self.dlls)))
        for separator in ('/', '\\'):
            with self.subTest(separator=separator), patch('zipfile.os.sep', separator), \
                    self.zip_with(names) as archive:
                self.assertEqual(set(sdk.zip_members(archive)), {name.rstrip('/') for name in names})

    def test_duplicates_case_and_file_directory_collisions(self):
        for names in (('a', 'a'), ('a', 'A'), ('a', 'a/b'), ('a', 'A/b'), ('a', 'a/')):
            with self.subTest(names=names), self.zip_with(names) as archive, self.assertRaises(ValueError):
                sdk.zip_members(archive)

    def test_nul_zip_name_is_rejected(self):
        info = zipfile.ZipInfo('safe\0unsafe')
        archive = Mock()
        archive.infolist.return_value = [info]
        with self.assertRaisesRegex(ValueError, 'Unsafe'):
            sdk.zip_members(archive)

    def test_zip_links_special_types_encryption_and_reparse_points(self):
        for mode in (stat.S_IFLNK, stat.S_IFIFO, stat.S_IFSOCK, stat.S_IFCHR):
            info = zipfile.ZipInfo('unsafe')
            info.external_attr = (mode | 0o600) << 16
            archive = Mock()
            archive.infolist.return_value = [info]
            with self.subTest(mode=mode), self.assertRaisesRegex(ValueError, 'entry type'):
                sdk.zip_members(archive)
        for attribute, value in (('flag_bits', 1), ('external_attr', 0x400), ('compress_type', zipfile.ZIP_BZIP2)):
            info = zipfile.ZipInfo('unsafe')
            setattr(info, attribute, value)
            archive.infolist.return_value = [info]
            with self.subTest(attribute=attribute), self.assertRaisesRegex(ValueError, 'entry type'):
                sdk.zip_members(archive)

    def test_zip_size_count_total_and_ratio_limits(self):
        with self.zip_with(('a', 'b')) as archive:
            for attribute, limit in (('MAX_ENTRIES', 1), ('MAX_ENTRY', -1), ('MAX_EXPANDED', -1)):
                with patch.object(sdk, attribute, limit), self.assertRaises(ValueError):
                    sdk.zip_members(archive)
        info = zipfile.ZipInfo('bomb')
        info.file_size, info.compress_size = 10000, 1
        archive = Mock()
        archive.infolist.return_value = [info]
        with self.assertRaisesRegex(ValueError, 'expansion limit'):
            sdk.zip_members(archive)

    def response(self, data, length=None):
        response = io.BytesIO(data)
        response.headers = {} if length is None else {'Content-Length': str(length)}
        response.geturl = lambda: sdk.URL
        return response

    def test_download_hashes_streamed_bytes_and_enforces_completion(self):
        data = b'content'
        with patch('sdk.urllib.request.build_opener') as opener:
            opener.return_value.open.return_value = self.response(data, len(data))
            path = self.base / 'download'
            self.assertEqual(sdk.download(sdk.URL, path, digest(data), 100), digest(data))
            self.assertEqual(path.read_bytes(), data)
            self.assertEqual(opener.return_value.open.call_args.kwargs['timeout'], 30)

    def test_download_wrong_hash_truncation_and_size_are_rejected(self):
        for data, length, expected, limit in ((b'x', 1, '0' * 64, 100),
                (b'x', 2, digest(b'x'), 100), (b'large', None, digest(b'large'), 1),
                (b'x', 1000, digest(b'x'), 1)):
            path = self.base / 'download'
            with self.subTest(length=length, limit=limit), patch('sdk.urllib.request.build_opener') as opener:
                opener.return_value.open.return_value = self.response(data, length)
                with self.assertRaises(ValueError):
                    sdk.download(sdk.URL, path, expected, limit)
                path.unlink(missing_ok=True)

    def test_network_failure_and_total_timeout_are_rejected(self):
        for error in (OSError('connection lost'), http.client.IncompleteRead(b'truncated chunk')):
            with patch('sdk.urllib.request.build_opener') as opener:
                opener.return_value.open.side_effect = error
                with self.assertRaisesRegex(ValueError, 'download failed'):
                    sdk.download(sdk.URL, self.base / 'download', '0' * 64, 100)
        with self.assertRaisesRegex(ValueError, 'time limit'):
            sdk.copy_stream(io.BytesIO(b'x'), self.base / 'timed-out', 100, deadline=0)

    def test_https_and_redirect_origins_are_pinned(self):
        for url in ('http://github.com/a', 'https://evil.example/a', 'https://github.com.evil/a',
                    'https://user@github.com/a', 'https://github.com:444/a'):
            with self.subTest(url=url), self.assertRaises(ValueError):
                sdk.check_url(url)
        with self.assertRaises(ValueError):
            sdk.PinnedRedirects().redirect_request(Mock(), None, 302, '', {}, 'http://github.com/a')
        sdk.check_url('https://release-assets.githubusercontent.com/a')

    def test_strict_unavailable_fails_without_claiming_success(self):
        self.install()
        with self.assertRaisesRegex(ValueError, 'unavailable'):
            sdk.verify(self.root, strict=True)
        with patch.object(sdk, 'WINDOWS', True), patch('sdk.shutil.which', return_value=None):
            self.assertEqual(sdk.signatures(self.root)['status'], 'not_run')
            with self.assertRaisesRegex(ValueError, 'unavailable'):
                sdk.signatures(self.root, strict=True)

    def test_strict_install_never_publishes_when_trust_is_unavailable(self):
        with self.assertRaisesRegex(ValueError, 'unavailable'):
            self.install(strict=True)
        self.assertFalse(self.root.exists())
        self.assertEqual(list(self.base.glob('.installed*')), [])

    def test_destination_appearing_before_publication_is_preserved(self):
        def signature_check(*args):
            self.root.mkdir()
            (self.root / 'user.txt').write_text('mine')
            return {'status': 'not_run'}
        with patch.object(sdk, 'signatures', side_effect=signature_check), \
                self.assertRaisesRegex(ValueError, 'appeared during installation'):
            self.install()
        self.assertEqual((self.root / 'user.txt').read_text(), 'mine')

    def test_hardlinked_installed_file_is_rejected(self):
        self.install()
        os.link(self.root / '3rdpartynotice.md', self.base / 'hardlink')
        with self.assertRaisesRegex(ValueError, 'unlinked regular file'):
            sdk.verify(self.root)

    def test_incomplete_transfer_is_cleaned_up_by_installation(self):
        def incomplete(local, target):
            target.write_bytes(b'incomplete')
            raise ValueError('connection interrupted')
        with patch.object(sdk, 'acquire_archive', side_effect=incomplete), self.assertRaises(ValueError):
            self.install()
        self.assertFalse(self.root.exists())
        self.assertEqual(list(self.base.glob('.installed*')), [])

    @unittest.skipUnless(shutil.which('pwsh'), 'PowerShell 7 parser unavailable')
    def test_current_windows_bridge_parses_without_execution(self):
        command = ('$tokens=$null; $errors=$null; '
                   '[System.Management.Automation.Language.Parser]::ParseFile('
                   '$env:FSR_SDK_BRIDGE_SYNTAX_SOURCE,[ref]$tokens,[ref]$errors) | Out-Null; '
                   'if ($errors.Count) { $errors | Out-String | Write-Error; exit 1 }')
        subprocess.run(['pwsh', '-NoProfile', '-NonInteractive', '-Command', command],
                       env=dict(os.environ, FSR_SDK_BRIDGE_SYNTAX_SOURCE=str(sdk.HERE / 'windows.ps1')),
                       check=True, capture_output=True, timeout=30)

    def test_windows_signature_failures_are_not_silently_downgraded(self):
        with patch.object(sdk, 'WINDOWS', True), patch('sdk.shutil.which', return_value='pwsh'), \
                patch('sdk.subprocess.run', side_effect=subprocess.TimeoutExpired('pwsh', 180)):
            with self.assertRaisesRegex(ValueError, 'failed or incomplete'):
                sdk.signatures(self.root)

    def test_strict_requires_matching_dlls_and_all_trust_checks(self):
        records = [{'file': name, 'status': 'Valid', 'signer': 'CN=Advanced Micro Devices',
                    'strict_passed': True} for name in self.dlls]
        with patch.object(sdk, 'WINDOWS', True), patch('sdk.shutil.which', return_value='pwsh'), \
                patch('sdk.subprocess.run') as run:
            run.return_value.stdout = json.dumps(records)
            self.assertEqual(sdk.signatures(self.root, True)['status'], 'passed')
            self.assertIn('-Strict', run.call_args.args[0])
            records[0]['strict_passed'] = False
            run.return_value.stdout = json.dumps(records)
            with self.assertRaisesRegex(ValueError, 'verification incomplete'):
                sdk.signatures(self.root, True)
            records[0]['status'] = 'NotTrusted'
            run.return_value.stdout = json.dumps(records)
            with self.assertRaisesRegex(ValueError, 'publisher check failed'):
                sdk.signatures(self.root)

    def test_malformed_signature_output_is_rejected(self):
        for records in ([], ['wrong', 'shape'], [{'file': name, 'status': 'Valid',
                'signer': 'CN=Advanced Micro Devices', 'strict_passed': 'true'} for name in self.dlls]):
            with self.subTest(records=records), patch.object(sdk, 'WINDOWS', True), \
                    patch('sdk.shutil.which', return_value='pwsh'), patch('sdk.subprocess.run') as run:
                run.return_value.stdout = json.dumps(records)
                with self.assertRaises(ValueError):
                    sdk.signatures(self.root, True)

    def test_cli_install_verify_and_errors(self):
        with contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(main(['sdk', 'install', '--root', str(self.root), '--archive', str(self.archive),
                                   '--source-root', str(self.source)]), 0)
        self.assertEqual(json.loads(output.getvalue())['integrity'], 'passed')
        with contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(main(['sdk', 'verify', '--root', str(self.root)]), 0)
        with contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(main(['sdk', 'verify', '--root', str(self.root), '--strict']), 2)
        self.assertEqual(parser().parse_args(['sdk', 'verify']).command, 'sdk')


if __name__ == '__main__':
    unittest.main()
