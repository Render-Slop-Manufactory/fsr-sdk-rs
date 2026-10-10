// SPDX-License-Identifier: MPL-2.0

//! Exclusive fixture directories, including after process-ID reuse.
use std::{
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn directory(prefix: &str) -> PathBuf {
    create_in(&std::env::temp_dir(), prefix, &NEXT).expect("create fixture directory")
}

fn create_in(root: &Path, prefix: &str, next: &AtomicUsize) -> io::Result<PathBuf> {
    loop {
        let path = root.join(format!(
            "{prefix}-{}-{}",
            std::process::id(),
            next.fetch_add(1, Ordering::Relaxed)
        ));
        match std::fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            // Old processes can leave files or directories at these names.
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
}

#[test]
fn skips_existing_fixture_paths_without_reusing_or_overwriting_them() {
    let root = directory("fsr-sdk-directory-regression");
    let next = AtomicUsize::new(0);
    let candidate = |number| root.join(format!("fixture-{}-{number}", std::process::id()));
    std::fs::create_dir(candidate(0)).unwrap();
    std::fs::write(candidate(1), b"existing fixture").unwrap();

    let fresh = create_in(&root, "fixture", &next).unwrap();
    assert_eq!(fresh, candidate(2));
    assert!(candidate(0).is_dir());
    assert_eq!(std::fs::read(candidate(1)).unwrap(), b"existing fixture");
    assert_eq!(
        create_in(&root.join("missing"), "fixture", &next)
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound
    );

    std::fs::remove_dir(fresh).unwrap();
    std::fs::remove_file(candidate(1)).unwrap();
    std::fs::remove_dir(candidate(0)).unwrap();
    std::fs::remove_dir(root).unwrap();
}
