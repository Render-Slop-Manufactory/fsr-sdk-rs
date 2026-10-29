# SPDX-License-Identifier: MPL-2.0
"""Verify the complete 0.2.0 release, including movable, without publishing."""
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "target/release-020-20261010"
OUT.mkdir(parents=True, exist_ok=True)
SOURCE = ROOT
PACKAGE = OUT / "build/package"
ISOLATION = OUT / "isolation-final"
RESUME = "--resume" in sys.argv
RESULTS = json.loads((OUT / "checks.json").read_text()) if RESUME else []


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(name, data):
    (OUT / name).write_text(json.dumps(data, indent=2) + "\n")


def run(label, args, directory, env=None, diagnostic=None):
    previous = next((item for item in reversed(RESULTS) if item["label"] == label), None)
    if RESUME and previous and previous["exit"] == 0 and label != "consumer-metadata":
        print(f"{label}: preserved prior pass", flush=True)
        return subprocess.CompletedProcess(args, 0, stdout=previous["output"], stderr="")
    started = datetime.now(timezone.utc).isoformat()
    completed = subprocess.run(args, cwd=directory, env=env, capture_output=True, text=True)
    output = completed.stdout + completed.stderr
    sanitized = output.replace(str(ROOT), "<repo>").replace(ROOT.as_posix(), "<repo>")
    sanitized = sanitized.replace(str(Path.home()), "<user-home>")
    safe_args = [str(arg).replace(str(ROOT), "<repo>").replace(ROOT.as_posix(), "<repo>") for arg in args]
    record = {"label": label, "command": safe_args, "directory": directory.relative_to(ROOT).as_posix(),
              "started_utc": started, "finished_utc": datetime.now(timezone.utc).isoformat(),
              "exit": completed.returncode, "expected_diagnostic": diagnostic, "output": sanitized}
    RESULTS.append(record)
    save("checks.json", RESULTS)
    print(f"{label}: exit={completed.returncode}", flush=True)
    if diagnostic:
        assert completed.returncode != 0 and diagnostic in output, sanitized
    else:
        assert completed.returncode == 0, sanitized
    return completed


def index_path(name):
    if len(name) == 3:
        return Path("3") / name[0] / name
    return Path(name[:2]) / name[2:4] / name


def registry():
    if RESUME and ISOLATION.exists():
        assert (OUT / "archive-audit.json").exists()
        # A test workspace owns the extracted packages without editing either
        # normalized archive manifest. The wrapper dependency remains registry-only.
        (ISOLATION / "packages/Cargo.toml").write_text('[workspace]\nresolver = "3"\nmembers = ["fsr-sdk-0.2.0", "fsr-sdk-sys-0.1.1"]\n')
        return ISOLATION / "cargo-home", ISOLATION / "packages"
    assert not ISOLATION.exists(), "Preserve prior isolation; do not silently overwrite"
    registry_dir = ISOLATION / "registry"
    cargo_home = ISOLATION / "cargo-home"
    packages = ISOLATION / "packages"
    packages.mkdir(parents=True)
    (packages / "Cargo.toml").write_text('[workspace]\nresolver = "3"\nmembers = ["fsr-sdk-0.2.0", "fsr-sdk-sys-0.1.1"]\n')
    (registry_dir / "index").mkdir(parents=True)
    cargo_home.mkdir()
    cache = Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo")))
    lock = tomllib.loads((SOURCE / "Cargo.lock").read_text())
    inventories = []
    entries = []
    for item in lock["package"]:
        name, version = item["name"], item["version"]
        filename = f"{name}-{version}.crate"
        if name.startswith("fsr-sdk"):
            archive = PACKAGE / filename
            index = json.loads((PACKAGE / "tmp-registry/index" / index_path(name)).read_text())
            assert index["cksum"] == sha(archive)
            with tarfile.open(archive) as tar:
                inventory = []
                seen = set()
                for member in tar.getmembers():
                    relative = Path(member.name)
                    assert not relative.is_absolute() and ".." not in relative.parts
                    assert member.isfile() and member.name not in seen
                    seen.add(member.name)
                    inner = relative.relative_to(f"{name}-{version}")
                    assert inner.suffix.lower() not in (".dll", ".exe", ".pdb", ".obj", ".lib", ".ps1")
                    assert not any(part in ("external", "target", ".git", ".cargo", ".aws") for part in inner.parts)
                    content = tar.extractfile(member).read()
                    digest = hashlib.sha256(content).hexdigest()
                    inventory.append({"file": inner.as_posix(), "bytes": len(content), "sha256": digest})
                    if inner.as_posix() not in ("Cargo.toml", "Cargo.toml.orig", "Cargo.lock", ".cargo_vcs_info.json"):
                        assert content == (SOURCE / "crates" / name / inner).read_bytes(), inner
                    if inner.as_posix() == "LICENSE":
                        assert content == (ROOT / "LICENSE").read_bytes()
                    if name == "fsr-sdk" and inner.as_posix() == "Cargo.toml":
                        manifest = tomllib.loads(content.decode())
                        assert "movable" in manifest["features"]
                        assert manifest["package"]["metadata"]["docs"]["rs"]["features"] == ["movable"]
                        assert manifest["dependencies"]["fsr-sdk-sys"]["version"] == "0.1.1"
                        assert "path" not in manifest["dependencies"]["fsr-sdk-sys"]
                    if name == "fsr-sdk-sys" and inner.as_posix() == "LICENSE-AMD":
                        assert content == (ROOT / "crates/fsr-sdk-sys/LICENSE-AMD").read_bytes()
                tar.extractall(packages, filter="data")
            inventories.append({"archive": filename, "bytes": archive.stat().st_size,
                                "sha256": sha(archive), "files": inventory})
        else:
            candidates = list((cache / "registry/cache").glob(f"*/{filename}"))
            assert candidates, filename
            archive = candidates[0]
            assert sha(archive) == item["checksum"], filename
            paths = list((cache / "registry/index").glob("*/.cache/" + index_path(name).as_posix()))
            assert paths, name
            index = next(json.loads(piece) for piece in paths[0].read_bytes().split(b"\0")
                         if piece.startswith(b"{") and json.loads(piece)["vers"] == version)
            assert index["cksum"] == sha(archive)
        shutil.copyfile(archive, registry_dir / filename)
        destination = registry_dir / "index" / index_path(name)
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(json.dumps(index, separators=(",", ":")) + "\n")
        entries.append({"archive": filename, "sha256": sha(archive)})
    save("archive-audit.json", inventories)
    save("registry-inputs.json", entries)
    (cargo_home / "config.toml").write_text('[source.crates-io]\nreplace-with = "release-artifacts"\n\n'
        '[source.release-artifacts]\nlocal-registry = ' + json.dumps(registry_dir.as_posix()) + "\n")
    return cargo_home, packages


def main():
    env = dict(os.environ, CARGO_INCREMENTAL="0", RUSTDOCFLAGS="-D warnings", CARGO_TARGET_DIR=str(OUT / "build"))
    run("joint-package", ["cargo", "package", "--workspace", "--locked", "--offline", "--allow-dirty"], SOURCE, env)
    run("candidate-format", ["cargo", "fmt", "--all", "--", "--check"], SOURCE, env)
    for label, arguments in (
        ("candidate-clippy", ["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"]),
        ("candidate-tests", ["test", "--workspace"]),
        ("candidate-movable-tests", ["test", "--workspace", "--features", "fsr-sdk/movable"]),
        ("candidate-movable-clippy", ["clippy", "--workspace", "--all-targets", "--features", "fsr-sdk/movable", "--", "-D", "warnings"]),
        ("candidate-no-default-tests", ["test", "--workspace", "--no-default-features"]),
        ("candidate-no-default-clippy", ["clippy", "--workspace", "--all-targets", "--no-default-features", "--", "-D", "warnings"]),
        ("candidate-doc", ["doc", "--workspace", "--no-deps"]),
        ("candidate-no-default-doc", ["doc", "--workspace", "--no-deps", "--no-default-features"]),
        ("candidate-movable-doc", ["doc", "--workspace", "--no-deps", "--features", "fsr-sdk/movable"]),
        ("candidate-no-default-movable-tests", ["test", "--workspace", "--no-default-features", "--features", "fsr-sdk/movable"]),
        ("candidate-no-default-movable-clippy", ["clippy", "--workspace", "--all-targets", "--no-default-features", "--features", "fsr-sdk/movable", "--", "-D", "warnings"]),
    ):
        args = ["cargo"] + arguments
        insertion = args.index("--") if "--" in args else len(args)
        args[insertion:insertion] = ["--locked", "--offline"]
        run(label, args, SOURCE, env)
    for label, arguments in (
        ("msrv-check", ["check", "--workspace", "--all-targets"]),
        ("msrv-tests", ["test", "--workspace"]),
        ("msrv-movable-tests", ["test", "--workspace", "--features", "fsr-sdk/movable"]),
        ("msrv-no-default-tests", ["test", "--workspace", "--no-default-features"]),
        ("msrv-no-default-movable-check", ["check", "--workspace", "--all-targets", "--no-default-features", "--features", "fsr-sdk/movable"]),
    ):
        run(label, ["cargo", "+1.88.0"] + arguments + ["--locked", "--offline"], SOURCE, env)
    # Cargo itself produced both archives before this verifier. Preserve them.
    cargo_home, packages = registry()
    isolated_env = dict(env, CARGO_HOME=str(cargo_home))
    run("packaged-workspace-lock", ["cargo", "generate-lockfile", "--offline"], packages, isolated_env)
    for name, version in (("fsr-sdk", "0.2.0"), ("fsr-sdk-sys", "0.1.1")):
        directory = packages / f"{name}-{version}"
        run(f"packaged-{name}-all-targets", ["cargo", "check", "--all-targets", "--locked", "--offline"], directory, isolated_env)
        run(f"packaged-{name}-tests", ["cargo", "test", "--locked", "--offline"], directory, isolated_env)
        if name == "fsr-sdk":
            run("packaged-movable-tests", ["cargo", "test", "--features", "movable", "--locked", "--offline"], directory, isolated_env)
            run("packaged-movable-clippy", ["cargo", "clippy", "--all-targets", "--features", "movable", "--locked", "--offline", "--", "-D", "warnings"], directory, isolated_env)
        args = ["cargo", "doc", "--no-deps", "--locked", "--offline"]
        if name == "fsr-sdk-sys":
            args += ["--features", "dx12"]
        else:
            args += ["--features", "movable"]
        run(f"packaged-{name}-doc", args, directory, isolated_env)
    wrapper = ISOLATION / "consumer"
    (wrapper / "src").mkdir(parents=True, exist_ok=True)
    (wrapper / "Cargo.toml").write_text('''# SPDX-License-Identifier: MPL-2.0
[package]
name = "release-registry-consumer"
version = "0.0.0"
edition = "2024"
[workspace]
[features]
default = ["dx12"]
dx12 = ["fsr-sdk/dx12"]
movable = ["fsr-sdk/movable"]
[dependencies]
fsr-sdk = { version = "=0.2.0", default-features = false }
fsr-sdk-sys = { version = "=0.1.1", default-features = false }
''')
    (wrapper / "src/lib.rs").write_text('''// SPDX-License-Identifier: MPL-2.0
use fsr_sdk::upscaler::{Dimensions, common};
pub fn size() -> Dimensions { common::Dimensions::new(320, 180).unwrap() }
#[cfg(feature = "dx12")]
pub mod native {
    use fsr_sdk::{error::Error, runtime::Runtime, upscaler::*};
    pub fn frame<'a>(list: &'a ID3D12GraphicsCommandList, resources: UpscaleResources<'a>) -> Result<UpscaleDispatch<'a>, Error> {
        let _borrowed = resources.clone();
        let mut frame = UpscaleDispatch::new(list, resources, JitterOffsetPixels::new(0.25, -0.25)?,
            FrameTimeMillis::new(16.667)?, CameraParameters::new(0.1, 100.0, std::f32::consts::FRAC_PI_3, 1.0)?, true);
        frame.set_sharpening(Sharpening::Native(NativeSharpness::new(0.35)?));
        let _: &Sharpening = frame.sharpening();
        match frame.sharpening() { Sharpening::Disabled => {}, Sharpening::Native(v) => { let _ = v.value(); }, _ => {} }
        frame.set_sharpening(Sharpening::Disabled);
        let _ = (frame.resources().exposure(), frame.reset());
        Ok(frame)
    }
    pub fn traits() {
        fn send_sync<T: Send + Sync>() {}
        send_sync::<UpscaleDispatch<'static>>(); send_sync::<UpscaleResources<'static>>();
        send_sync::<Sharpening>(); send_sync::<NativeSharpness>();
        let _: fn(&Runtime, &ID3D12Device, UpscalerOptions) -> Result<Upscaler, Error> = Upscaler::new;
    }
}
''')
    (wrapper / "examples").mkdir(exist_ok=True)
    shutil.copyfile(packages / "fsr-sdk-0.2.0/examples/dx12_upscale.rs", wrapper / "examples/dx12_upscale.rs")
    shutil.copyfile(packages / "fsr-sdk-0.2.0/examples/movable_upscale.rs", wrapper / "examples/movable_upscale.rs")
    root_readme = (ROOT / "README.md").read_text()
    snippet = root_readme.split("```rust\n", 1)[1].split("```", 1)[0]
    (wrapper / "examples/readme.rs").write_text('''use fsr_sdk::upscaler::{ID3D12Device, ID3D12GraphicsCommandList, ID3D12Resource};
unsafe fn integration(loader_path: &std::path::Path, device: ID3D12Device,
    command_list: ID3D12GraphicsCommandList, color: ID3D12Resource,
    depth: ID3D12Resource, motion_vectors: ID3D12Resource,
    exposure: ID3D12Resource, output: ID3D12Resource)
    -> Result<(), Box<dyn std::error::Error>> {
''' + snippet + '\nOk(())\n}\nfn main() {}\n')
    # Compile the complete Windows Rustdoc snippets, including hidden setup.
    lines = (packages / "fsr-sdk-0.2.0/src/lib.rs").read_text().splitlines()
    snippets, current = [], None
    for line in lines:
        if line == "//! ```no_run":
            current = []
        elif line == "//! ```" and current is not None:
            snippets.append(current)
            current = None
        elif current is not None:
            code = line[3:].removeprefix(" ")
            current.append(code.removeprefix("# ") if code.startswith("# ") else code)
    for i, snippet in enumerate(snippets):
        (wrapper / f"examples/rustdoc-{i}.rs").write_text("\n".join(snippet) + "\n")
    run("consumer-lock", ["cargo", "generate-lockfile", "--offline"], wrapper, isolated_env)
    run("consumer-default", ["cargo", "check", "--all-targets", "--locked", "--offline"], wrapper, isolated_env)
    run("consumer-msrv", ["cargo", "+1.88.0", "check", "--all-targets", "--locked", "--offline"], wrapper, isolated_env)
    run("consumer-movable-examples", ["cargo", "check", "--all-targets", "--features", "movable", "--locked", "--offline"], wrapper, isolated_env)
    run("consumer-movable-msrv", ["cargo", "+1.88.0", "check", "--all-targets", "--features", "movable", "--locked", "--offline"], wrapper, isolated_env)
    # The DX12 example has its own cfg feature; compile only the portable library here.
    run("consumer-no-default", ["cargo", "check", "--lib", "--no-default-features", "--locked", "--offline"], wrapper, isolated_env)
    metadata = json.loads(run("consumer-metadata", ["cargo", "metadata", "--format-version=1", "--locked", "--offline"], wrapper, isolated_env).stdout)
    assert len(metadata["workspace_members"]) == 1
    for item in metadata["packages"]:
        if item["id"] in metadata["workspace_members"]:
            continue
        assert item["source"].startswith("registry+")
        assert Path(item["manifest_path"]).is_relative_to(cargo_home)
        assert all(dep.get("path") is None for dep in item["dependencies"])
    for item in tomllib.loads((wrapper / "Cargo.lock").read_text())["package"]:
        if item["name"].startswith("fsr-sdk"):
            assert item["checksum"] == sha(PACKAGE / f'{item["name"]}-{item["version"]}.crate')
    positive = (wrapper / "src/lib.rs").read_text()
    movable_positive = '''use fsr_sdk::{runtime::MovableRuntime, upscaler::MovableUpscaler};
pub fn traits() { fn send<T: Send>() {} send::<MovableRuntime>(); send::<MovableUpscaler>(); }
'''
    (wrapper / "src/lib.rs").write_text(movable_positive)
    run("consumer-movable-traits", ["cargo", "check", "--lib", "--features", "movable", "--locked", "--offline"], wrapper, isolated_env)
    run("consumer-movable-traits-msrv", ["cargo", "+1.88.0", "check", "--lib", "--features", "movable", "--locked", "--offline"], wrapper, isolated_env)
    for label, code, diagnostic in (
        ("movable-api-absent", "use fsr_sdk::runtime::MovableRuntime;", "E0432"),
        ("private-frame-field", "use fsr_sdk::upscaler::UpscaleDispatch; pub fn modify(f: &mut UpscaleDispatch<'_>) { f.reset = true; }", "E0616"),
        ("native-api-without-dx12", "use fsr_sdk::upscaler::NativeSharpness;", "E0432"),
    ):
        (wrapper / "src/lib.rs").write_text(code + "\n")
        args = ["cargo", "check", "--lib", "--locked", "--offline"]
        if label == "native-api-without-dx12":
            args += ["--no-default-features"]
        run(label, args, wrapper, isolated_env, diagnostic)
    for label, code, diagnostic in (
        ("movable-runtime-not-sync", "use fsr_sdk::runtime::MovableRuntime; fn check<T: Sync>() {} pub fn wrong() { check::<MovableRuntime>(); }", "E0277"),
        ("movable-upscaler-not-sync", "use fsr_sdk::upscaler::MovableUpscaler; fn check<T: Sync>() {} pub fn wrong() { check::<MovableUpscaler>(); }", "E0277"),
        ("local-runtime-not-send", "use fsr_sdk::runtime::Runtime; fn check<T: Send>() {} pub fn wrong() { check::<Runtime>(); }", "E0277"),
        ("local-upscaler-not-send", "use fsr_sdk::upscaler::Upscaler; fn check<T: Send>() {} pub fn wrong() { check::<Upscaler>(); }", "E0277"),
        ("local-to-movable", "use fsr_sdk::{runtime::Runtime, upscaler::*}; pub fn wrong(r: &Runtime, d: &ID3D12Device, o: UpscalerOptions) { let _ = MovableUpscaler::new(r,d,o); }", "E0308"),
        ("movable-to-local", "use fsr_sdk::{runtime::MovableRuntime, upscaler::*}; pub fn wrong(r: &MovableRuntime, d: &ID3D12Device, o: UpscalerOptions) { let _ = Upscaler::new(r,d,o); }", "E0308"),
        ("no-local-conversion", "use fsr_sdk::runtime::{Runtime,MovableRuntime}; pub fn wrong(r: Runtime) -> MovableRuntime { r.into() }", "E0277"),
        ("no-movable-conversion", "use fsr_sdk::runtime::{Runtime,MovableRuntime}; pub fn wrong(r: MovableRuntime) -> Runtime { r.into() }", "E0277"),
        ("movable-without-dx12", "use fsr_sdk::runtime::MovableRuntime;", "E0432"),
    ):
        (wrapper / "src/lib.rs").write_text(code + "\n")
        args = ["cargo", "check", "--lib", "--features", "movable", "--locked", "--offline"]
        if label == "movable-without-dx12":
            args += ["--no-default-features"]
        run(label, args, wrapper, isolated_env, diagnostic)
    (wrapper / "src/lib.rs").write_text(positive)
    # Feature/target dependency selection from the exact packaged registry crates.
    trees = []
    for name in ("fsr-sdk", "fsr-sdk-sys"):
        directory = packages / (name + ("-0.2.0" if name == "fsr-sdk" else "-0.1.1"))
        for target in ("x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"):
            configs = [("default", []), ("no-default", ["--no-default-features"]), ("dx12", ["--no-default-features", "--features", "dx12"])]
            if name == "fsr-sdk":
                configs += [("movable", ["--features", "movable"]), ("no-default-movable", ["--no-default-features", "--features", "movable"])]
            for config, flags in configs:
                output = run(f"tree-{name}-{target}-{config}", ["cargo", "tree", "--target", target, "--edges", "normal", "--prefix", "none", "--locked", "--offline"] + flags, directory, isolated_env).stdout
                expected_loader = target == "x86_64-pc-windows-msvc" and (config == "dx12" or (name == "fsr-sdk" and config in ("default", "movable")))
                assert ("libloading" in output) == expected_loader
                trees.append({"crate": name, "target": target, "config": config, "libloading": expected_loader, "tree": output})
    save("dependency-trees.json", trees)
    save("consumer-inputs.json", {"manifest": (wrapper / "Cargo.toml").read_text(), "source": positive,
                                  "lockfile": (wrapper / "Cargo.lock").read_text(), "registry_only": True,
                                  "complete_rustdoc_examples": len(snippets)})
    # Verify nothing changed after testing/extraction.
    for record in json.loads((OUT / "archive-audit.json").read_text()):
        assert sha(PACKAGE / record["archive"]) == record["sha256"]
        extracted = packages / record["archive"].removesuffix(".crate")
        for entry in record["files"]:
            assert sha(extracted / entry["file"]) == entry["sha256"], entry["file"]
    print("Complete release and packaged movable consumers: PASSED", flush=True)


if __name__ == "__main__":
    main()
