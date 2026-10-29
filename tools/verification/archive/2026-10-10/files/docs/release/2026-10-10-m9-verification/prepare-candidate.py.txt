# SPDX-License-Identifier: MPL-2.0
"""Prepare the bounded 0.2.0 candidate without changing the development checkout.

Versions identify new artifacts: wrapper 0.2.0; sys 0.1.1 carries the existing
optional/target-specific loader dependency change. D013 is not selected for this
release: omit its public modules/examples/tests and make its remaining cfg items
unconditionally inactive. Retain private ownership infrastructure and fixtures.
Every byte change and omission is recorded; no archive is edited after Cargo.
"""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[3]
OUT = (ROOT / (sys.argv[1] if len(sys.argv) > 1 else "target/m9-release-20261010-final")).resolve()
if not OUT.is_relative_to(ROOT / "target"):
    raise SystemExit("Candidate output must remain inside the repository target directory")
SOURCE = OUT / "source"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    if SOURCE.exists():
        raise SystemExit("Use a fresh candidate directory; preserve previous inputs")
    SOURCE.mkdir(parents=True)
    omitted = {
        "crates/fsr-sdk/src/runtime/movable.rs",
        "crates/fsr-sdk/src/upscaler/dx12/movable.rs",
        "crates/fsr-sdk/src/upscaler/dx12/tests/movable.rs",
        "crates/fsr-sdk/tests/movable.rs",
        "crates/fsr-sdk/tests/native/movable_public.rs",
        "crates/fsr-sdk/examples/movable_upscale.rs",
    }
    files = [ROOT / name for name in ("Cargo.toml", "Cargo.lock", "LICENSE")]
    files += sorted(path for path in (ROOT / "crates").rglob("*") if path.is_file())
    records = []
    for path in files:
        name = path.relative_to(ROOT).as_posix()
        original = path.read_bytes()
        if name in omitted:
            records.append({"file": name, "input_sha256": sha(original), "action": "omit D013 public family"})
            continue
        data = original
        changes = []
        if name == "crates/fsr-sdk/Cargo.toml":
            text = data.decode()
            assert text.count('version = "0.1.0"') == 2
            text = text.replace('version = "0.1.0"', 'version = "0.2.0"', 1)
            text = text.replace('version = "0.1.0"', 'version = "0.1.1"', 1)
            text, count = re.subn(r"(?m)^# Opt-in caller-established[^\n]*\nmovable = \[\]\r?\n", "", text)
            assert count == 1
            data = text.encode()
            changes.append("wrapper 0.2.0; sys dependency 0.1.1; omit movable feature")
        elif name == "crates/fsr-sdk-sys/Cargo.toml":
            data = data.replace(b'version = "0.1.0"', b'version = "0.1.1"', 1)
            changes.append("sys 0.1.1: existing loader packaging/dependency maintenance")
        elif name == "Cargo.lock":
            data = data.replace(b'name = "fsr-sdk"\r\nversion = "0.1.0"', b'name = "fsr-sdk"\r\nversion = "0.2.0"')
            data = data.replace(b'name = "fsr-sdk-sys"\r\nversion = "0.1.0"', b'name = "fsr-sdk-sys"\r\nversion = "0.1.1"')
            data = data.replace(b'name = "fsr-sdk"\nversion = "0.1.0"', b'name = "fsr-sdk"\nversion = "0.2.0"')
            data = data.replace(b'name = "fsr-sdk-sys"\nversion = "0.1.0"', b'name = "fsr-sdk-sys"\nversion = "0.1.1"')
            changes.append("workspace package identities only; third-party lock entries unchanged")
        if name.endswith(".rs"):
            text = data.decode()
            text, count = re.subn(r'(?m)^(\s*#!?)\[cfg\([^\n]*feature = "movable"[^\n]*\)\]', r'\1[cfg(any())]', text)
            if count:
                changes.append(f"disable {count} D013 cfg items; feature absent")
            text, removed = re.subn(r'(?m)^\s*#\[cfg\(any\(\)\)\]\r?\n(?:mod movable(?:_public)?;|pub use (?:movable|dx12)::Movable(?:Runtime|Upscaler);)\r?\n', "\n", text)
            if removed:
                changes.append(f"remove {removed} inactive D013 module/reexport declarations")
            if name == "crates/fsr-sdk/src/lib.rs":
                text, count = re.subn(r"(?ms)^//! With the optional `movable` feature,.*?(?=^//! # Minimal integration)", "", text)
                assert count == 1
                changes.append("omit D013 public documentation")
            if name == "crates/fsr-sdk/src/runtime.rs":
                text = text.replace("Explicit local ownership and optional movable ownership", "Explicit local ownership")
            data = text.encode()
        if name == "crates/fsr-sdk/README.md":
            text, count = re.subn(r"(?ms)^## Optional movable ownership in current source.*?(?=^## Consumer workflow)", "", data.decode())
            assert count == 1
            text = text.replace('Install the [published `0.1.0` crate](https://crates.io/crates/fsr-sdk):', 'The locally verified release candidate uses this dependency after publication:')
            text = text.replace('fsr-sdk = "0.1.0"', 'fsr-sdk = "0.2.0"')
            data = text.encode()
            changes.append("candidate install version and omit D013 overview")
        destination = SOURCE / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(data)
        records.append({"file": name, "input_sha256": sha(original), "output_sha256": sha(data), "changes": changes})
    subprocess.run(["cargo", "fmt", "--manifest-path", str(SOURCE / "Cargo.toml"), "--all"], check=True)
    for record in records:
        if "output_sha256" in record:
            final_hash = sha((SOURCE / record["file"]).read_bytes())
            if final_hash != record["output_sha256"]:
                record["changes"].append("rustfmt after removing inactive declarations")
                record["output_sha256"] = final_hash
    (OUT / "projection.json").write_text(json.dumps({
        "source": "<repo>", "destination": SOURCE.relative_to(ROOT).as_posix(),
        "wrapper": "0.2.0", "sys": "0.1.1", "movable_selected": False,
        "files": records,
    }, indent=2) + "\n")
    print(f"Prepared candidate: {SOURCE.relative_to(ROOT)} ({len(records)} input records)")


if __name__ == "__main__":
    main()
