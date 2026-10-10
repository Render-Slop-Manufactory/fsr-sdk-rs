// SPDX-License-Identifier: MPL-2.0

//! Isolated test execution. Polling is only a watchdog, never test coordination.
use std::{
    fs::File,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

pub(crate) fn run(dir: &Path, test: &str, variable: &str, case: &str) {
    let stdout = dir.join("stdout.txt");
    let stderr = dir.join("stderr.txt");
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--nocapture"])
        .env(variable, case)
        .stdout(File::create(&stdout).unwrap())
        .stderr(File::create(&stderr).unwrap())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    let (status, timeout) = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break (status, false);
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            break (child.wait().unwrap(), true);
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let output = std::fs::read_to_string(stdout).unwrap();
    let errors = std::fs::read_to_string(stderr).unwrap();
    println!("CHILD {case}: timeout={timeout}, exit={status}\n{output}\n{errors}");
    assert!(!timeout, "child watchdog: {case}");
    assert!(status.success(), "isolated case failed: {case}");
    assert!(
        output
            .lines()
            .any(|line| line == format!("CASE PASS {case}")),
        "missing case completion: {case}"
    );
}
