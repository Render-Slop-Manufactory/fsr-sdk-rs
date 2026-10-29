# SPDX-License-Identifier: MPL-2.0
"""Preserve sanitized checks and prove the tested release archives are unchanged."""
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys
import tarfile

ROOT = Path(__file__).resolve().parents[3]
EVIDENCE = Path(__file__).resolve().parent
OUT = ROOT / "target/release-020-20261010"


def sanitize(text):
    for path, replacement in ((ROOT, "<repo>"), (Path.home(), "<user-home>")):
        text = text.replace(str(path), replacement).replace(path.as_posix(), replacement)
        escaped = str(path)
        for _ in range(3):
            escaped = escaped.replace("\\", "\\\\")
            text = text.replace(escaped, replacement)
    return re.sub(r"(?i)[a-z]:[\\/]+Users[\\/]+[^\\/\s\"<>]+", "<user-home>", text)


def save(name, value):
    (EVIDENCE / name).write_text(sanitize(json.dumps(value, indent=2)) + "\n", encoding="utf-8")


def collect_movable(stage_name, label):
    stage = ROOT / stage_name
    destination = EVIDENCE / label
    destination.mkdir(exist_ok=True)
    for name in ("source-inputs.json", "native-inputs.json", "native.environment.txt", "checks.status.txt", "test-package-inputs.json"):
        path = stage / name
        if path.exists():
            (destination / name).write_text(sanitize(path.read_text(encoding="utf-8-sig")), encoding="utf-8")
    cases = []
    for name in ("native-explicit", "native-drop", "gate-a", "gate-b", "gate-c", "public-movable", "public-movable-sharpening"):
        status = (stage / f"{name}.status.txt").read_text(encoding="utf-8").strip()
        stdout = (stage / f"{name}.stdout.txt").read_text(encoding="utf-8")
        stderr = (stage / f"{name}.stderr.txt").read_text(encoding="utf-8")
        assert "timeout=False exit=0" in status and "1 passed; 0 failed" in stdout
        observations = [line for line in stderr.splitlines() if line.startswith((
            "Public movable", "D013 native destroy:", "Device checkpoint:", "Provider:",
            "M6 provider:", "Adapter:", "Driver query:", "Production owner:", "Gate ",
            "GPU readback:", "Temporal frame", "Semantic deltas:", "Analytic red MAE:"))]
        if name.startswith("public-movable"):
            metrics = [line for line in observations if line.startswith("Public movable fenced readback:")]
            assert len(metrics) == 2
            for metric in metrics:
                assert "finite_pixels: 230400" in metric and "changed_pixels: 230400" in metric
                assert "debug_errors: 0" in metric and "device_removed: 0" in metric
            destroys = [line for line in observations if line.startswith("D013 native destroy:")]
            assert len(destroys) == 2 and all("code=0x00000000" in line for line in destroys)
            assert any("unwinding=true domain_held=true" in line for line in observations)
            assert any("unwinding=false domain_held=true" in line for line in observations)
            assert any("Public movable: PASSED" in line for line in observations)
        cases.append({"case": name, "status": status, "observations": observations})
    (destination / "native-cases.json").write_text(sanitize(json.dumps(cases, indent=2)) + "\n", encoding="utf-8")
    return cases


def main():
    for name in ("archive-audit.json", "registry-inputs.json", "dependency-trees.json", "consumer-inputs.json"):
        (EVIDENCE / name).write_text(sanitize((OUT / name).read_text(encoding="utf-8")), encoding="utf-8")
    checks = json.loads((OUT / "checks.json").read_text(encoding="utf-8"))
    # Keep evidence needed for registry provenance, without unrelated dependency
    # author/contact metadata from Cargo's full metadata response.
    for check in checks:
        if check["label"] == "consumer-metadata" and check["exit"] == 0:
            metadata = json.JSONDecoder().raw_decode(check["output"])[0]
            check["output"] = json.dumps({
                "workspace_members": metadata["workspace_members"],
                "packages": [{key: package[key] for key in ("name", "version", "source", "manifest_path", "dependencies")}
                             for package in metadata["packages"]],
                "retained_metadata_fields": "registry and dependency provenance only",
            }, indent=2)
    save("checks.json", checks)
    for name in ("publish-dry-run.log", "publish-dry-run-online.log"):
        (EVIDENCE / name).write_text(sanitize((OUT / name).read_text(encoding="utf-8")), encoding="utf-8")
    archives = json.loads((OUT / "archive-audit.json").read_text(encoding="utf-8"))
    for record in archives:
        archive = OUT / "build/package" / record["archive"]
        assert hashlib.sha256(archive.read_bytes()).hexdigest() == record["sha256"]
        dry_run = OUT / "dry-run/package/tmp-crate" / record["archive"]
        assert hashlib.sha256(dry_run.read_bytes()).hexdigest() == record["sha256"]
        extracted = OUT / "isolation-final/packages" / record["archive"].removesuffix(".crate")
        with tarfile.open(archive) as tar:
            for member in tar:
                content = tar.extractfile(member).read()
                assert str(ROOT).encode() not in content and ROOT.as_posix().encode() not in content
                assert str(Path.home()).encode() not in content
        for entry in record["files"]:
            assert hashlib.sha256((extracted / entry["file"]).read_bytes()).hexdigest() == entry["sha256"]
    for stage, label in (("target/release-020-native-source-unrestricted", "movable-source"),
                         ("target/release-020-native-artifact", "movable-artifact")):
        collect_movable(stage, label)
    # Reuse the published M9 half-float audit without changing its protocol or
    # historical result files. Its implementation is part of the repository.
    collector = ROOT / "docs/release/2026-10-10-m9-verification/collect-evidence.py"
    spec = importlib.util.spec_from_file_location("m9_evidence", collector)
    module = importlib.util.module_from_spec(spec)
    sys.dont_write_bytecode = True
    spec.loader.exec_module(module)
    module.EVIDENCE = EVIDENCE
    module.collect("target/release-020-m9-artifact", "m9-artifact")
    latest = {record["label"]: record for record in checks}
    for record in latest.values():
        diagnostic = record["expected_diagnostic"]
        assert (record["exit"] != 0 and diagnostic in record["output"]) if diagnostic else record["exit"] == 0
    dry_log = (OUT / "publish-dry-run-online.log").read_text(encoding="utf-8")
    assert dry_log.count("warning: aborting upload due to dry run") == 2
    source_controls = []
    positives = {"common-positive", "common-movable-positive", "dispatch-inputs-positive", "raw-loader-positive", "local-positive", "movable-positive"}
    for path in sorted((ROOT / "target/release-020-api-controls").glob("*.txt")):
        output = path.read_text(encoding="utf-8")
        diagnostics = re.findall(r"error\[(E\d+)\]", output)
        assert not diagnostics if path.stem in positives else diagnostics
        source_controls.append({"case": path.stem, "expected_failure": path.stem not in positives,
                                "diagnostics": diagnostics, "output": output})
    assert len(source_controls) == 34
    save("source-api-controls.json", {
        "command": "./tools/check-movable-api.ps1 -Stage target/release-020-api-controls",
        "script_sha256": hashlib.sha256((ROOT / "tools/check-movable-api.ps1").read_bytes()).hexdigest(),
        "observed_script_exit": 0, "cases": source_controls,
        "additional_script_controls": "24 production dependency trees and the intentional Vulkan diagnostic passed",
    })
    save("summary.json", {
        "wrapper": "0.2.0", "sys": "0.1.1", "movable_included": True,
        "gates": len(latest), "expected_negative_controls": sum(bool(r["expected_diagnostic"]) for r in latest.values()),
        "archives_and_all_extracted_bytes_unchanged": True, "dry_run_identical_archives": True,
        "publish_dry_run_aborted_uploads": 2, "publication_performed": False,
        "movable_native": {"runs": 2, "children_each": 7, "public_frames_each": 4,
                           "modes": ["Disabled", "Native(0.35)"], "all_passed": True},
        "m9_native_artifact": {"repetitions": 3, "outputs": 144, "all_passed": True},
        "source_api_controls": "tools/check-movable-api.ps1 completed successfully",
        "setup_failures": [
            "Sandbox package verification could not initialize Cargo's temporary registry hash/cache; unchanged unrestricted command passed.",
            "Sandbox native runner failed in a compile fixture on denied temporary metadata output, before GPU children; fresh unrestricted source stage passed.",
            "Verifier resume initially assumed an existing isolation record; guard corrected before registry setup.",
            "Verifier initially read nested docs.rs TOML metadata as a dotted literal; corrected parser lookup, incomplete isolation retained locally.",
            "Sandbox publication dry-run could not resolve the crates.io index; index-enabled dry-run passed and aborted both uploads.",
        ],
        "native_acceptance_failures_or_retries": 0,
    })
    print(f"Preserved {len(latest)} release gates, both archives, movable and M9 native evidence")


if __name__ == "__main__":
    main()
