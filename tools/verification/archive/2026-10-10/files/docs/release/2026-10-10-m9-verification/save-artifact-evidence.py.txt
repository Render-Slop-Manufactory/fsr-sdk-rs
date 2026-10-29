# SPDX-License-Identifier: MPL-2.0
"""Copy reviewed result records and confirm final artifact/input identity."""
import hashlib
import json
from pathlib import Path
import re
import tarfile

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "target/m9-release-20261010-final"
EVIDENCE = Path(__file__).resolve().parent


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sanitize(text):
    for path, replacement in ((ROOT, "<repo>"), (Path.home(), "<user-home>")):
        text = text.replace(str(path), replacement).replace(path.as_posix(), replacement)
        escaped = str(path)
        for _ in range(3):
            escaped = escaped.replace("\\", "\\\\")
            text = text.replace(escaped, replacement)
    return text


for name in ("projection.json", "archive-audit.json", "registry-inputs.json", "checks.json", "dependency-trees.json", "consumer-inputs.json", "publish-dry-run-online.log"):
    text = sanitize((OUT / name).read_text())
    assert not re.search(r"(?i)[a-z]:[\\/]+Users[\\/]+", text)
    (EVIDENCE / name).write_text(text)
archives = json.loads((OUT / "archive-audit.json").read_text())
for record in archives:
    archive = OUT / "build/package" / record["archive"]
    assert sha(archive) == record["sha256"]
    assert sha(OUT / "dry-run/package/tmp-crate" / record["archive"]) == record["sha256"]
    with tarfile.open(archive) as tar:
        for member in tar:
            content = tar.extractfile(member).read()
            assert str(ROOT).encode() not in content and ROOT.as_posix().encode() not in content
            assert str(Path.home()).encode() not in content
    # Match every extracted byte against the original archive inventory after
    # all compilation, doctests, native execution and terminal teardown.
    package = OUT / "isolation/packages" / record["archive"].removesuffix(".crate")
    for entry in record["files"]:
        assert sha(package / entry["file"]) == entry["sha256"], entry["file"]
for record in json.loads((OUT / "projection.json").read_text())["files"]:
    if "output_sha256" in record:
        assert sha(OUT / "source" / record["file"]) == record["output_sha256"]
latest = {}
for check in json.loads((OUT / "checks.json").read_text()):
    latest[check["label"]] = check
for check in latest.values():
    diagnostic = check["expected_diagnostic"]
    assert (check["exit"] != 0 and diagnostic in check["output"]) if diagnostic else check["exit"] == 0
summary = {
    "wrapper": "0.2.0", "sys": "0.1.1", "movable_feature": False,
    "final_gates": [{"label": check["label"], "exit": check["exit"], "expected_diagnostic": check["expected_diagnostic"]} for check in latest.values()],
    "archives_unchanged": True, "extracted_files_unchanged": True,
    "source_projection_unchanged": True, "publication_performed": False,
    "publish_dry_run": {"exit": 0, "aborted_uploads": (OUT / "publish-dry-run-online.log").read_text().count("warning: aborting upload due to dry run")},
}
assert summary["publish_dry_run"]["aborted_uploads"] == 2
(EVIDENCE / "artifact-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
api = sanitize((ROOT / "target/m9-api-controls.log").read_text())
(EVIDENCE / "development-consumer-controls.txt").write_text(api)
print(f"Preserved {len(latest)} final artifact gates and both unchanged archives")
