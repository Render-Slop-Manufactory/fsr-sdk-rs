# SPDX-License-Identifier: MPL-2.0
"""Audit release content, local navigation, provenance and unchanged protocols."""
import ast
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tarfile
import tomllib
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[3]
EVIDENCE = Path(__file__).resolve().parent
OUT = ROOT / "target/release-020-20261010"
PATTERNS = {
    "private_home_path": re.compile(r"(?i)(?:[a-z]:[\\/]+Users[\\/]+(?!Public\b|<)[^\\/\s\"<>]+|/(?:Users|home)/[^/\s\"<>]+)"),
    "personal_username": re.compile(rf"(?i)\b{re.escape(Path.home().name)}\b"),
    "private_key_or_access_token": re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----|github_pat_[A-Za-z0-9_]{20,}|gh[pousr]_[A-Za-z0-9]{20,}|AKIA[A-Z0-9]{16}|sk-(?:proj-)?[A-Za-z0-9_-]{24,}"),
}
EMAIL = re.compile(r"(?i)[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}")


def fingerprint(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    names = subprocess.check_output(["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"], cwd=ROOT).decode().split("\0")
    text_files = 0
    skipped_binary = []
    email_locations = []
    links = 0
    mapped_historical = []
    for name in sorted(set(names) - {""}):
        path = ROOT / name
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            skipped_binary.append(name)
            continue
        text_files += 1
        for label, pattern in PATTERNS.items():
            assert not pattern.search(text), (name, label)
        if EMAIL.search(text):
            email_locations.append(name)
        if path.suffix == ".md":
            for url in re.findall(r"\]\(([^)]+)\)", text):
                if "://" in url or url.startswith("mailto:"):
                    continue
                file, _, anchor = unquote(url).partition("#")
                destination = (path.parent / file).resolve() if file else path
                if destination == EVIDENCE / "final-audit.json":
                    # This successful run creates its own linked audit result.
                    links += 1
                    continue
                if not destination.exists():
                    relative = destination.relative_to(ROOT).as_posix()
                    assert name.startswith(("docs/release/", "docs/research/records/")), (name, url)
                    assert relative.startswith("crates/fsr-sdk/src/upscaler/"), (name, url)
                    replacement = relative.replace("crates/fsr-sdk/src/upscaler/", "crates/fsr-sdk/src/upscaler/dx12/", 1)
                    assert (ROOT / replacement).exists(), (name, url)
                    mapped_historical.append({"document": name, "recorded_path": file, "current_path": replacement})
                elif anchor and destination.suffix == ".md":
                    headings = re.findall(r"(?m)^#{1,6} (.+)$", destination.read_text(encoding="utf-8"))
                    slugs = [re.sub(r"[^\w\- ]", "", heading.lower()).replace(" ", "-") for heading in headings]
                    assert anchor in slugs, (name, url)
                links += 1
    for path in EVIDENCE.rglob("*.json"):
        json.loads(path.read_text(encoding="utf-8-sig"))
    for path in EVIDENCE.glob("*.py"):
        ast.parse(path.read_text(encoding="utf-8"))
    archive_files = 0
    archives = json.loads((OUT / "archive-audit.json").read_text(encoding="utf-8"))
    for record in archives:
        archive = OUT / "build/package" / record["archive"]
        assert fingerprint(archive) == record["sha256"]
        crate_name = "fsr-sdk-sys" if record["archive"].startswith("fsr-sdk-sys-") else "fsr-sdk"
        with tarfile.open(archive) as tar:
            for member in tar:
                archive_files += 1
                inner = Path(member.name).relative_to(record["archive"].removesuffix(".crate"))
                content = tar.extractfile(member).read()
                text = content.decode("utf-8")
                for label, pattern in PATTERNS.items():
                    assert not pattern.search(text), (member.name, label)
                assert not EMAIL.search(text), member.name
                if inner.as_posix() == "LICENSE":
                    assert content == (ROOT / "LICENSE").read_bytes()
                elif inner.as_posix() == "LICENSE-AMD":
                    assert content == (ROOT / "crates/fsr-sdk-sys/LICENSE-AMD").read_bytes()
                elif inner.as_posix() == "Cargo.toml.orig":
                    assert content == (ROOT / "crates" / crate_name / "Cargo.toml").read_bytes()
                elif inner.as_posix() not in ("Cargo.toml", "Cargo.lock", ".cargo_vcs_info.json"):
                    assert content == (ROOT / "crates" / crate_name / inner).read_bytes(), member.name
    old_lock = tomllib.loads(subprocess.check_output(["git", "show", "HEAD:Cargo.lock"], cwd=ROOT).decode())
    current_lock = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
    third_party = lambda lock: [item for item in lock["package"] if not item["name"].startswith("fsr-sdk")]
    assert third_party(old_lock) == third_party(current_lock)
    unchanged_protocols = []
    for name in ("docs/release/2026-10-09-m9-native-sharpening-protocol.md",
                 "docs/research/records/2026-09-30-exp-d014-native-sharpening.md",
                 "crates/fsr-sdk/tests/native/sharpening.rs"):
        previous = subprocess.check_output(["git", "show", "HEAD:" + name], cwd=ROOT).decode().replace("\r\n", "\n")
        assert previous == (ROOT / name).read_text(encoding="utf-8")
        unchanged_protocols.append({"file": name, "sha256": fingerprint(ROOT / name)})
    expected_emails = ["docs/release/2026-10-10-m9-verification/checks.json"]
    assert email_locations == expected_emails, email_locations
    result = {
        "working_tree_text_files_scanned": text_files,
        "binary_files_skipped": skipped_binary,
        "archive_files_scanned": archive_files,
        "private_paths_private_keys_tokens": "no matches",
        "personal_username": "no matches",
        "email_locations": email_locations,
        "email_classification": "public third-party dependency author metadata in two historical checks; none in the release archives",
        "local_links_checked": links,
        "historical_source_links_mapped_via_development_document": mapped_historical,
        "current_documentation_links_valid": True,
        "third_party_lock_inputs_unchanged": True,
        "source_matches_archived_source_after_all_checks": True,
        "license_copies_and_amd_notices_match": True,
        "unchanged_protocol_and_failed_effect_evidence": unchanged_protocols,
        "evidence_json_and_python_syntax_valid": True,
        "limits": "pattern/content review of the publishable working tree and archives; no Git history rewrite, exhaustive secret guarantee or full dependency-advisory scan",
    }
    (EVIDENCE / "final-audit.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: value for key, value in result.items() if key != "historical_source_links_mapped_via_development_document"}, indent=2))


if __name__ == "__main__":
    main()
