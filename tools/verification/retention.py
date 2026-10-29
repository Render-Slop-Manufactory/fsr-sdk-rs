# SPDX-License-Identifier: MPL-2.0
"""Bounded calibration evidence: keep results; raw diagnostics are disposable."""
import hashlib
import json
from pathlib import Path

from records import checked_path, inventory, now, save, verify_artifacts


CALIBRATION_PROFILES = ('experiment-temporal-calibration', 'experiment-v01-calibration', 'experiment-temporal-diagnosis', 'experiment-temporal-quality', 'experiment-temporal-long', 'experiment-temporal-reset-explore', 'experiment-temporal-reset', 'experiment-temporal-jitter-explore', 'experiment-temporal-jitter')
CALIBRATION_PROFILES += tuple('proton-temporal-'+kind for kind in ('quality','long','reset','jitter'))


def calibration_diagnostic(name):
    """Identify bulky calibration files by responsibility, including old names."""
    parts = Path(name).parts
    namespaces = ('experiment.temporal-calibration.', 'experiment.v01-calibration.', 'experiment.temporal-diagnosis.', 'experiment.temporal-quality.', 'experiment.temporal-long.', 'experiment.temporal-reset-explore.', 'experiment.temporal-reset.', 'experiment.temporal-jitter-explore.', 'experiment.temporal-jitter.')
    if len(parts) >= 3 and parts[0] == 'cases' and parts[1].startswith(namespaces):
        return 'sources' in parts[2:] or parts[-1] == 'input-audit.json'
    if len(parts) >= 3 and parts[0] == 'readbacks' and parts[1].startswith(namespaces):
        return parts[-1] not in ('payloads.json', 'frames.jsonl')
    if len(parts) >= 3 and parts[:2] in (('analysis', 'temporal-calibration'), ('analysis', 'v01'), ('analysis', 'temporal-diagnosis'), ('analysis', 'temporal-quality'), ('analysis', 'temporal-long'), ('analysis','temporal-reset'), ('analysis','temporal-jitter')):
        return parts[-1] not in ('summary.json', 'diagnostic-summary.json', 'contact-sheet.png')
    return False


def compact_case(value):
    """Keep payload totals and inventory identity without duplicating manifests."""
    value = dict(value)
    observations = dict(value.get('observations', {}))
    payloads = observations.pop('payloads', None)
    if payloads is not None:
        observations['payload_inventory'] = {
            'files': len(payloads),
            'sha256': hashlib.sha256(json.dumps(payloads, sort_keys=True).encode()).hexdigest(),
        }
    value['observations'] = observations
    return value


def compact_calibration(root):
    """Finalize a validated local attempt, including failures, before export.

    Only classified files inside this attempt are removed. The inventory and
    retention record disclose the loss of offline image reanalysis; outcomes
    and available diagnostics remain failures when an audit did not complete.
    """
    root = Path(root).resolve()
    summary = json.loads((root / 'run.json').read_text(encoding='utf-8'))
    if summary['profile'] not in CALIBRATION_PROFILES or summary['status'] == 'running':
        raise ValueError('Only finalized calibration attempts can be compacted')
    entries = verify_artifacts(root)
    removed = [entry for entry in entries if calibration_diagnostic(entry['path'])]
    # Resolve every deletion before modifying any file. Inventory validation
    # rejects symlinks and traversal, and preserves unrelated run artefacts.
    paths = [checked_path(root, entry['path']) for entry in removed]
    save(root / 'retention.json', {
        'schema_version': 1, 'policy': 'compact-calibration', 'applied_utc': now(),
        'original_status': summary['status'], 'offline_image_reanalysis_available': False,
        'removed_files': len(removed), 'removed_bytes': sum(entry['bytes'] for entry in removed),
        'removed_inventory_sha256': hashlib.sha256(json.dumps(removed, sort_keys=True).encode()).hexdigest(),
        'reproduction': 'Run the pinned protocol again; GPU results may change with equipment or software.',
    })
    for path in paths:
        path.unlink()
    for path in (root / 'cases').glob('*/result.json'):
        save(path, compact_case(json.loads(path.read_text(encoding='utf-8'))))
    save(root / 'artifacts.json', inventory(root))
    return len(removed)
