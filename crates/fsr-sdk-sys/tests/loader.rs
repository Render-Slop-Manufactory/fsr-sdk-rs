// SPDX-License-Identifier: MPL-2.0

#![cfg(all(windows, feature = "dx12"))]

use fsr_sdk_sys::{
    api::*,
    loader::{FfxLibrary, LoadError},
};
use std::{
    error::Error,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn work_dir() -> PathBuf {
    create_work_dir(&std::env::temp_dir(), &NEXT).expect("create loader fixture directory")
}

fn create_work_dir(root: &Path, next: &AtomicUsize) -> std::io::Result<PathBuf> {
    loop {
        let path = root.join(format!(
            "fsr-sdk-loader-{}-{}",
            std::process::id(),
            next.fetch_add(1, Ordering::Relaxed)
        ));
        match std::fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
}

#[test]
fn fixture_directory_skips_stale_paths_and_preserves_other_errors() {
    let root = work_dir();
    let next = AtomicUsize::new(0);
    let stale = root.join(format!("fsr-sdk-loader-{}-0", std::process::id()));
    std::fs::create_dir(&stale).unwrap();
    let fresh = create_work_dir(&root, &next).unwrap();
    assert_ne!(fresh, stale);
    assert!(stale.is_dir());
    assert_eq!(
        create_work_dir(&root.join("missing"), &next)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::NotFound
    );
    std::fs::remove_dir(fresh).unwrap();
    std::fs::remove_dir(stale).unwrap();
    std::fs::remove_dir(root).unwrap();
}

fn fixture(dir: &Path, missing: Option<&str>) -> PathBuf {
    let path = dir.join("loader_fixture.dll");
    let mut cmd = Command::new("rustc");
    cmd.args([
        "--edition=2024",
        "--crate-type=cdylib",
        "--crate-name=loader_fixture",
    ])
    .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/loader.rs"))
    .arg("-o")
    .arg(&path);
    if let Some(symbol) = missing {
        cmd.arg("--cfg").arg(format!("missing_{symbol}"));
    }
    let output = cmd
        .output()
        .expect("rustc must be available for loader fixtures");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    path
}

fn assert_unloaded(path: &Path) {
    assert!(libloading::os::windows::Library::open_already_loaded(path).is_err());
}

#[test]
fn missing_file_reports_path_and_source() {
    std::thread::Builder::new().name("d013-load-failure".into()).spawn(|| {
        assert_eq!(std::thread::current().name(), Some("d013-load-failure"));
        let path = work_dir().join("does-not-exist.dll");
        // SAFETY: The file does not exist; no DLL code can execute.
        let error = unsafe { FfxLibrary::load(&path) }.err().expect("missing file");
        assert!(matches!(&error, LoadError::Path { path: p, .. } if p == &path));
        let source = error.source().unwrap().downcast_ref::<std::io::Error>().unwrap();
        assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
        assert!(error.to_string().contains("does-not-exist.dll"));
        assert_unloaded(&path);
        println!("worker=d013-load-failure category=Path source=io/NotFound runtime=none module=absent");
    }).unwrap().join().unwrap();
    println!("CASE PASS missing-file");
}

#[test]
fn invalid_library_reports_load_failure() {
    let path = work_dir().join("invalid.dll");
    std::fs::write(&path, b"not a PE image").unwrap();
    // SAFETY: This known invalid image contains no executable DLL code.
    let error = unsafe { FfxLibrary::load(&path) }
        .err()
        .expect("invalid DLL");
    assert!(matches!(error, LoadError::Library { .. }));
    assert!(error.source().is_some());
    assert!(error.to_string().contains("invalid.dll"));
}

#[test]
fn each_missing_symbol_reports_name_and_releases_library() {
    for symbol in [
        "ffxCreateContext",
        "ffxDestroyContext",
        "ffxConfigure",
        "ffxQuery",
        "ffxDispatch",
    ] {
        let dir = work_dir();
        let path = fixture(&dir, Some(symbol));
        std::thread::Builder::new().name("d013-load-failure".into()).spawn(move || {
            assert_eq!(std::thread::current().name(), Some("d013-load-failure"));
            assert_unloaded(&path);
            // SAFETY: Controlled DLL with matching remaining exports and no
            // custom initialization or termination routines. No observer owns it.
            let error = unsafe { FfxLibrary::load(&path) }.err().expect("missing export");
            assert!(matches!(&error, LoadError::Symbol { symbol: s, path: p, .. }
                if *s == symbol && *p == path.canonicalize().unwrap()));
            assert!(error.source().unwrap().downcast_ref::<libloading::Error>().is_some());
            assert!(error.to_string().contains(symbol));
            assert_unloaded(&path);
            println!("worker=d013-load-failure category=Symbol symbol={symbol} source=libloading runtime=none module=released");
        }).unwrap().join().unwrap();
    }
    println!("CASE PASS missing-exports");
}

#[test]
fn forwards_all_arguments_and_returns_and_unloads_on_drop() {
    let dir = work_dir();
    let path = fixture(&dir, None);
    // SAFETY: Test-controlled DLL, matching exports, no custom init/termination.
    let library = unsafe { FfxLibrary::load(&path) }.unwrap();
    // Moving the owner must retain the loaded module.
    let library = Box::new(library);
    assert!(libloading::os::windows::Library::open_already_loaded(&path).is_ok());
    // SAFETY: These fixture functions explicitly accept sentinel pointer values
    // and never dereference them. No calls are made to AMD's runtime.
    unsafe {
        assert_eq!(
            library.ffxCreateContext(
                0x10 as *mut ffxContext,
                0x20 as *mut ffxCreateContextDescHeader,
                0x30 as *const ffxAllocationCallbacks
            ),
            101
        );
        assert_eq!(
            library.ffxDestroyContext(
                0x10 as *mut ffxContext,
                0x30 as *const ffxAllocationCallbacks
            ),
            102
        );
        assert_eq!(
            library.ffxConfigure(
                0x10 as *mut ffxContext,
                0x20 as *const ffxConfigureDescHeader
            ),
            103
        );
        assert_eq!(
            library.ffxQuery(0x10 as *mut ffxContext, 0x20 as *mut ffxQueryDescHeader),
            104
        );
        assert_eq!(
            library.ffxDispatch(
                0x10 as *mut ffxContext,
                0x20 as *const ffxDispatchDescHeader
            ),
            105
        );
    }
    drop(library);
    assert_unloaded(&path);
}

#[test]
#[ignore = "requires an explicitly supplied trusted AMD v2.3.0 loader DLL"]
fn loads_amd_runtime_without_api_calls() {
    let path = std::env::var_os("FSR_SDK_TEST_DLL")
        .expect("set FSR_SDK_TEST_DLL to the trusted loader DLL");
    // SAFETY: Explicit opt-in promises the supplied SDK DLL/dependencies are
    // trusted, ABI compatible, and safe to initialize/unload in this process.
    let library = unsafe { FfxLibrary::load(PathBuf::from(path)) }.expect("AMD runtime exports");
    drop(library);
}
