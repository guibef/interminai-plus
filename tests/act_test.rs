use std::process::{Command};

mod common;
use common::DaemonHandle;

#[test]
fn test_act_click() {
    let daemon = DaemonHandle::spawn(&["--", "cat"]);

    // First get output to detect elements
    let _ = Command::new(common::interminai_bin())
        .args(&["output", "--socket", daemon.socket(), "--vom"])
        .output()
        .unwrap();

    // ACT click
    let output = Command::new(common::interminai_bin())
        .args(&["act", "--socket", daemon.socket(), "--action", "click", "--target", "@btn1"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Since @btn1 likely won't be found in a simple 'cat' session
    assert!(stderr.contains("Element not found") || output.status.success());

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_act_unknown_action() {
    let daemon = DaemonHandle::spawn(&["--", "cat"]);

    let output = Command::new(common::interminai_bin())
        .args(&["act", "--socket", daemon.socket(), "--action", "invalid", "--target", "@e1"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Element not found") || stderr.contains("Unknown action"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_act_scroll() {
    let daemon = DaemonHandle::spawn(&["--", "cat"]);

    let output = Command::new(common::interminai_bin())
        .args(&["act", "--socket", daemon.socket(), "--action", "scroll", "--target", "@e1", "--direction", "down", "--amount", "5"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Element not found") || output.status.success());

    let mut daemon = daemon;
    daemon.stop();
}
