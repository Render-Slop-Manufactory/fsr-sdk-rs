# SPDX-License-Identifier: MPL-2.0
"""Retain sanitized native observations and independently audit red half images."""
import array
import csv
import hashlib
import json
from pathlib import Path
import re
import struct
import sys

ROOT = Path(__file__).resolve().parents[3]
EVIDENCE = Path(__file__).resolve().parent


def sanitize(text):
    for path, replacement in ((ROOT, "<repo>"), (Path.home(), "<user-home>")):
        text = text.replace(str(path), replacement).replace(path.as_posix(), replacement)
    return text


def collect(stage_name, label):
    stage = ROOT / stage_name
    destination = EVIDENCE / label
    destination.mkdir(exist_ok=True)
    for name in ("source-inputs.json", "native-inputs.json", "native.environment.txt", "checks.status.txt", "execution-plan.json", "payload-hashes.json", "test-package-inputs.json"):
        path = stage / name
        if path.exists():
            (destination / name).write_text(sanitize(path.read_text(encoding="utf-8-sig")))
    cases = []
    comparisons = []
    frames = []
    for name in ("native-explicit", "native-drop", "gate-a", "gate-b", "gate-c", "m9-repeat-1", "m9-repeat-2", "m9-repeat-3"):
        state = (stage / f"{name}.status.txt").read_text().strip()
        assert "timeout=False exit=0" in state, (name, state)
        stdout = (stage / f"{name}.stdout.txt").read_text()
        assert "1 passed; 0 failed" in stdout
        stderr = (stage / f"{name}.stderr.txt").read_text()
        observations = [line for line in stderr.splitlines() if line.startswith(("M9_", "M9 integration:", "Provider", "M6 provider:", "Adapter:", "Driver query:", "Production owner:", "Loader after", "Gate ", "GPU readback:", "Temporal frame", "Semantic deltas:", "Analytic red MAE:", "Device checkpoint:"))]
        cases.append({"case": name, "status": state, "observations": observations})
        if not name.startswith("m9-repeat"):
            continue
        repeat = int(name[-1])
        image_dir = stage / f"{name}.images"
        assert len(list(image_dir.glob("*.red.u16le"))) == 48
        for scene in range(2):
            for arm in ("switch-a", "switch-b", "disabled"):
                for frame in range(8):
                    prefix = f"M9_FRAME scene={scene} arm={arm} frame={frame} "
                    observation = next(line for line in observations if line.startswith(prefix))
                    fields = dict(re.findall(r"(\w+)=([^ ]+)", observation))
                    for key, value in (("finish_hr", "0x0"), ("fence", "1"), ("width", "640"), ("height", "360"), ("finite", "230400"), ("written", "230400"), ("debug_errors", "0"), ("device_removed", "0x0")):
                        assert fields[key] == value, (key, observation)
                    path = image_dir / f"scene-{scene}-{arm}-frame-{frame}.red.u16le"
                    assert path.stat().st_size == 460800
                    frames.append({"repeat": repeat, "scene": scene, "arm": arm, "frame": frame,
                                   "sha256": hashlib.sha256(path.read_bytes()).hexdigest(), "observation": observation})
            for frame in range(8):
                for pair, second in (("a-b", "switch-b"), ("a-disabled", "disabled")):
                    if pair == "a-disabled" and frame < 3:
                        continue
                    a = (image_dir / f"scene-{scene}-switch-a-frame-{frame}.red.u16le").read_bytes()
                    b = (image_dir / f"scene-{scene}-{second}-frame-{frame}.red.u16le").read_bytes()
                    bits_a, bits_b = array.array("H"), array.array("H")
                    bits_a.frombytes(a)
                    bits_b.frombytes(b)
                    assert sys.byteorder == "little"
                    red_a = struct.unpack("<230400e", a)
                    red_b = struct.unpack("<230400e", b)
                    roi = [y * 640 + x for y in range(32, 328) for x in range(32, 608)]
                    mismatches = sum(bits_a[i] != bits_b[i] for i in roi)
                    mean = sum(abs(red_a[i] - red_b[i]) for i in roi) / len(roi)
                    prefix = f"M9_COMPARE scene={scene} frame={frame} pair={pair} "
                    fields = dict(re.findall(r"(\w+)=([^ ]+)", next(line for line in observations if line.startswith(prefix))))
                    assert int(fields["roi_bit_mismatches"]) == mismatches
                    assert (fields["roi_bit_equal"] == "true") == (mismatches == 0)
                    assert abs(float(fields["roi_mae_f64"]) - mean) <= 1e-16
                    comparisons.append({"repeat": repeat, "scene": scene, "frame": frame, "pair": pair,
                                        "roi_bit_equal": mismatches == 0, "roi_bit_mismatches": mismatches,
                                        "roi_mae_f64": mean, "diagnostic_only": True})
    assert len(frames) == 144 and len(comparisons) == 78
    (destination / "native-cases.json").write_text(sanitize(json.dumps(cases, indent=2)) + "\n")
    (destination / "frame-observations.json").write_text(json.dumps(frames, indent=2) + "\n")
    with (destination / "diagnostic-audit.csv").open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=list(comparisons[0]))
        writer.writeheader()
        writer.writerows(comparisons)
    summary = {"outputs": 144, "process_repetitions": 3, "comparisons_independently_audited": 78,
               "all_execution_checks_passed": True, "image_effect_thresholds": None,
               "a_b_roi_bit_equal": all(row["roi_bit_equal"] for row in comparisons if row["pair"] == "a-b"),
               "return_to_disabled_roi_bit_equal": all(row["roi_bit_equal"] for row in comparisons if row["pair"] == "a-disabled" and row["frame"] == 7),
               "mode_diagnostics_first_repetition": [row for row in comparisons if row["repeat"] == 1 and row["pair"] == "a-disabled"]}
    (destination / "audit-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    collect(sys.argv[1], sys.argv[2])
