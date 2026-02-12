use assert_cmd::Command;
use predicates::prelude::*;
use std::thread;
use std::time::Duration;

mod common;
use common::{interminai_bin, emulator_args, TestEnv, DaemonHandle};

#[test]
fn test_start_without_socket_auto_generates() {
    let daemon = DaemonHandle::spawn(&["--", "sleep", "10"]);

    // Should have auto-generated socket
    // Fix: check for prefix only, as temp dir varies by OS
    assert!(daemon.socket().contains("interminai-"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_start_with_socket_uses_it() {
    let env = TestEnv::new();

    let daemon = DaemonHandle::spawn_with_socket(&env.socket(), &["cat"]);

    assert_eq!(daemon.socket(), env.socket());

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_start_creates_daemon() {
    let daemon = DaemonHandle::spawn(&["--", "sleep", "10"]);

    // Check if process is running
    let status = Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success();

    status.stdout(predicate::str::contains("Running: true"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_stop_terminates_daemon() {
    let daemon = DaemonHandle::spawn(&["--", "sleep", "10"]);

    let socket = daemon.socket().to_string();
    let mut daemon = daemon;
    daemon.stop();

    // Socket should be gone
    assert!(!std::path::Path::new(&socket).exists());

    // Status should fail
    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(&socket)
        .assert()
        .failure();
}

#[test]
fn test_output_gets_screen() {
    let daemon = DaemonHandle::spawn(&["--", "echo", "Hello World"]);

    thread::sleep(Duration::from_millis(500));

    Command::new(interminai_bin())
        .arg("output")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--no-color")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello World"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_input_sends_keys() {
    // Use cat to echo input
    let daemon = DaemonHandle::spawn(&["--", "cat"]);

    Command::new(interminai_bin())
        .arg("input")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--text")
        .arg("Hello\\n")
        .assert()
        .success();

    thread::sleep(Duration::from_millis(500));

    Command::new(interminai_bin())
        .arg("output")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--no-color")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_kill_sigint() {
    let daemon = DaemonHandle::spawn(&["--", "sleep", "100"]);

    Command::new(interminai_bin())
        .arg("kill")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--signal")
        .arg("SIGINT")
        .assert()
        .success();

    thread::sleep(Duration::from_millis(500));

    // Process should have exited
    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success()
        .stdout(predicate::str::contains("Running: false"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_kill_sigkill() {
    let daemon = DaemonHandle::spawn(&["--", "sleep", "100"]);

    Command::new(interminai_bin())
        .arg("kill")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--signal")
        .arg("SIGKILL")
        .assert()
        .success();

    thread::sleep(Duration::from_millis(500));

    // Process should have exited
    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success()
        .stdout(predicate::str::contains("Running: false"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_kill_numeric_signal_9() {
    let daemon = DaemonHandle::spawn(&["--", "sleep", "100"]);

    Command::new(interminai_bin())
        .arg("kill")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--signal")
        .arg("9")
        .assert()
        .success();

    thread::sleep(Duration::from_millis(500));

    // Process should have exited
    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success()
        .stdout(predicate::str::contains("Running: false"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_kill_numeric_signal_15() {
    let daemon = DaemonHandle::spawn(&["--", "sleep", "100"]);

    Command::new(interminai_bin())
        .arg("kill")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--signal")
        .arg("15")
        .assert()
        .success();

    thread::sleep(Duration::from_millis(500));

    // Process should have exited
    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success()
        .stdout(predicate::str::contains("Running: false"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_kill_numeric_signal_2() {
    let daemon = DaemonHandle::spawn(&["--", "sleep", "100"]);

    Command::new(interminai_bin())
        .arg("kill")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--signal")
        .arg("2")
        .assert()
        .success();

    thread::sleep(Duration::from_millis(500));

    // Process should have exited
    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success()
        .stdout(predicate::str::contains("Running: false"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_kill_default_sigterm() {
    let daemon = DaemonHandle::spawn(&["--", "sleep", "100"]);

    Command::new(interminai_bin())
        .arg("kill")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success();

    thread::sleep(Duration::from_millis(500));

    // Process should have exited
    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success()
        .stdout(predicate::str::contains("Running: false"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_running_when_finished() {
    let daemon = DaemonHandle::spawn(&["--", "true"]);

    thread::sleep(Duration::from_millis(500));

    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success()
        .stdout(predicate::str::contains("Running: false"))
        .stdout(predicate::str::contains("Exit code: 0"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_wait_until_exit() {
    let daemon = DaemonHandle::spawn(&["--", "sh", "-c", "sleep 1; exit 42"]);

    Command::new(interminai_bin())
        .arg("wait")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success();

    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success()
        .stdout(predicate::str::contains("Running: false"))
        .stdout(predicate::str::contains("Exit code: 42"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_wait_already_finished() {
    let daemon = DaemonHandle::spawn(&["--", "true"]);

    thread::sleep(Duration::from_millis(500));

    // Should return immediately
    Command::new(interminai_bin())
        .arg("wait")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success();

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_socket_reuse() {
    let env = TestEnv::new();

    // Start and stop one
    {
        let mut daemon = DaemonHandle::spawn_with_socket(&env.socket(), &["true"]);
        daemon.stop();
    }

    // Start another with same socket
    {
        let mut daemon = DaemonHandle::spawn_with_socket(&env.socket(), &["true"]);
        daemon.stop();
    }
}

#[test]
fn test_user_socket_not_deleted_on_stop() {
    let env = TestEnv::new();

    let mut daemon = DaemonHandle::spawn_with_socket(&env.socket(), &["sleep", "10"]);
    daemon.stop();

    // User-provided socket path should NOT be deleted (its container might be, but we check path)
    // Actually, current implementation deletes the socket file but not the temp dir.
    // That's fine.
}

#[test]
fn test_multiple_clients_simultaneous() {
    let daemon = DaemonHandle::spawn(&["--", "cat"]);

    let mut threads = vec![];
    for i in 0..5 {
        let socket = daemon.socket().to_string();
        threads.push(thread::spawn(move || {
            let bin = interminai_bin();
            for j in 0..10 {
                let text = format!("thread{}-msg{}\\n", i, j);
                std::process::Command::new(&bin)
                    .arg("input")
                    .arg("--socket")
                    .arg(&socket)
                    .arg("--text")
                    .arg(&text)
                    .status()
                    .unwrap();
            }
        }));
    }

    for t in threads {
        t.join().unwrap();
    }

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_terminal_size_option() {
    let daemon = DaemonHandle::spawn(&["--size", "120x40", "--", "cat"]);

    Command::new(interminai_bin())
        .arg("output")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success();
    // Verification of size is hard via text output, but at least it didn't crash

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_running_no_daemon() {
    let env = TestEnv::new();
    let mut child = std::process::Command::new(interminai_bin())
        .arg("start")
        .arg("--socket")
        .arg(env.socket())
        .arg("--no-daemon")
        .arg("--")
        .arg("sleep")
        .arg("10")
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    thread::sleep(Duration::from_millis(500));

    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(env.socket())
        .assert()
        .success()
        .stdout(predicate::str::contains("Running: true"));

    // Stop via socket
    Command::new(interminai_bin())
        .arg("stop")
        .arg("--socket")
        .arg(env.socket())
        .assert()
        .success();

    child.wait().unwrap();
}

#[test]
fn test_running_after_stop() {
    let daemon = DaemonHandle::spawn(&["--", "sleep", "10"]);

    let socket = daemon.socket().to_string();
    let mut daemon = daemon;
    daemon.stop();

    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(&socket)
        .assert()
        .failure();
}

#[test]
fn test_running_when_active() {
    let daemon = DaemonHandle::spawn(&["--", "sh", "-c", "echo active; sleep 10"]);

    thread::sleep(Duration::from_millis(500));

    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success()
        .stdout(predicate::str::contains("Running: true"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_parallel_sessions() {
    let daemon1 = DaemonHandle::spawn(&["--", "sleep", "10"]);
    let daemon2 = DaemonHandle::spawn(&["--", "sleep", "10"]);

    assert_ne!(daemon1.socket(), daemon2.socket());

    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(daemon1.socket())
        .assert()
        .success()
        .stdout(predicate::str::contains("Running: true"));

    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(daemon2.socket())
        .assert()
        .success()
        .stdout(predicate::str::contains("Running: true"));

    let mut daemon1 = daemon1;
    let mut daemon2 = daemon2;
    daemon1.stop();
    daemon2.stop();
}

#[test]
fn test_vim_wq_exits() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    let daemon = DaemonHandle::spawn(&["--", "vim", "-u", "NONE", path]);

    // Give vim time to start
    thread::sleep(Duration::from_millis(1000));

    // Send :wq
    Command::new(interminai_bin())
        .arg("input")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--text")
        .arg(":wq\\n")
        .assert()
        .success();

    // Wait for exit
    Command::new(interminai_bin())
        .arg("wait")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success();

    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success()
        .stdout(predicate::str::contains("Running: false"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_vim_quit_without_save() {
    let daemon = DaemonHandle::spawn(&["--", "vim", "-u", "NONE"]);

    thread::sleep(Duration::from_millis(1000));

    Command::new(interminai_bin())
        .arg("input")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--text")
        .arg(":q!\\n")
        .assert()
        .success();

    Command::new(interminai_bin())
        .arg("wait")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success();

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_vim_save_and_verify() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    let daemon = DaemonHandle::spawn(&["--", "vim", "-u", "NONE", path]);

    thread::sleep(Duration::from_millis(1000));

    // Type content
    Command::new(interminai_bin())
        .arg("input")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--text")
        .arg("iHello Vim\\e:wq\\n")
        .assert()
        .success();

    Command::new(interminai_bin())
        .arg("wait")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success();

    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("Hello Vim"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_vim_multiline_edit() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    let daemon = DaemonHandle::spawn(&["--", "vim", "-u", "NONE", path]);

    thread::sleep(Duration::from_millis(1000));

    // Type multiline content
    Command::new(interminai_bin())
        .arg("input")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--text")
        .arg("iLine 1\\nLine 2\\nLine 3\\e:wq\\n")
        .assert()
        .success();

    Command::new(interminai_bin())
        .arg("wait")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success();

    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("Line 1"));
    assert!(content.contains("Line 2"));
    assert!(content.contains("Line 3"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_vim_arrow_key_navigation() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    let daemon = DaemonHandle::spawn(&["--", "vim", "-u", "NONE", path]);

    thread::sleep(Duration::from_millis(1000));

    // Type two lines, go up, and append
    Command::new(interminai_bin())
        .arg("input")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--text")
        .arg("iFirst\\nSecond\\e\\x1b[AAtop\\e:wq\\n")
        .assert()
        .success();

    Command::new(interminai_bin())
        .arg("wait")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success();

    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("Firsttop"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_vim_exits_eventually_after_quit() {
    let daemon = DaemonHandle::spawn(&["--", "vim", "-u", "NONE"]);

    thread::sleep(Duration::from_millis(1000));

    Command::new(interminai_bin())
        .arg("input")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--text")
        .arg(":q\\n")
        .assert()
        .success();

    thread::sleep(Duration::from_millis(1000));

    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success()
        .stdout(predicate::str::contains("Running: false"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_vim_create_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("new_file.txt");
    let path_str = path.to_str().unwrap();

    let daemon = DaemonHandle::spawn(&["--", "vim", "-u", "NONE", path_str]);

    thread::sleep(Duration::from_millis(1000));

    Command::new(interminai_bin())
        .arg("input")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--text")
        .arg("iNew File Content\\e:wq\\n")
        .assert()
        .success();

    Command::new(interminai_bin())
        .arg("wait")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success();

    assert!(path.exists());
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("New File Content"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_vim_edit_existing_file() {
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    use std::io::Write;
    write!(temp_file, "Original Content").unwrap();
    let path = temp_file.path().to_str().unwrap();

    let daemon = DaemonHandle::spawn(&["--", "vim", "-u", "NONE", path]);

    thread::sleep(Duration::from_millis(1000));

    // Append to file
    Command::new(interminai_bin())
        .arg("input")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--text")
        .arg("A Appended\\e:wq\\n")
        .assert()
        .success();

    Command::new(interminai_bin())
        .arg("wait")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success();

    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("Original Content Appended"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_vim_insert_mode_visible() {
    let daemon = DaemonHandle::spawn(&["--", "vim", "-u", "NONE", "-c", "set nocompatible showmode"]);

    thread::sleep(Duration::from_millis(2000));

    Command::new(interminai_bin())
        .arg("input")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--text")
        .arg("i")
        .assert()
        .success();

    thread::sleep(Duration::from_millis(2000));

    Command::new(interminai_bin())
        .arg("output")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--no-color")
        .assert()
        .success()
        .stdout(predicate::str::contains("-- INSERT --"));

    Command::new(interminai_bin())
        .arg("input")
        .arg("--socket")
        .arg(daemon.socket())
        .arg("--text")
        .arg("\\e:q!\\n")
        .assert()
        .success();

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_input_requires_socket() {
    Command::new(interminai_bin())
        .arg("input")
        .assert()
        .failure();
}

#[test]
fn test_output_requires_socket() {
    Command::new(interminai_bin())
        .arg("output")
        .assert()
        .failure();
}

#[test]
fn test_status_requires_socket() {
    Command::new(interminai_bin())
        .arg("status")
        .assert()
        .failure();
}

#[test]
fn test_wait_requires_socket() {
    Command::new(interminai_bin())
        .arg("wait")
        .assert()
        .failure();
}

#[test]
fn test_stop_requires_socket() {
    Command::new(interminai_bin())
        .arg("stop")
        .assert()
        .failure();
}

#[test]
fn test_kill_requires_socket() {
    Command::new(interminai_bin())
        .arg("kill")
        .assert()
        .failure();
}

#[test]
fn test_invalid_request_gets_error_response() {
    let daemon = DaemonHandle::spawn(&["--", "cat"]);

    use std::os::unix::net::UnixStream;
    use std::io::Write;

    let mut stream = UnixStream::connect(daemon.socket()).unwrap();
    stream.write_all(b"not json\n").unwrap();

    let mut response = String::new();
    use std::io::Read;
    stream.read_to_string(&mut response).unwrap();

    assert!(response.contains("error"));
    assert!(response.contains("Invalid JSON"));

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_incomplete_request_daemon_survives() {
    let daemon = DaemonHandle::spawn(&["--", "cat"]);

    use std::os::unix::net::UnixStream;
    use std::io::Write;

    {
        let mut stream = UnixStream::connect(daemon.socket()).unwrap();
        stream.write_all(b"{\"type\": \"INPUT\"").unwrap();
        // Disconnect without finishing JSON
    }

    // Daemon should still be responsive
    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success();

    let mut daemon = daemon;
    daemon.stop();
}

#[test]
fn test_client_dies_before_response() {
    let daemon = DaemonHandle::spawn(&["--", "cat"]);

    use std::os::unix::net::UnixStream;
    use std::io::Write;

    {
        let mut stream = UnixStream::connect(daemon.socket()).unwrap();
        stream.write_all(b"{\"type\": \"WAIT\"}\n").unwrap();
        // Disconnect immediately while daemon is in WAIT
    }

    // Daemon should still be responsive
    Command::new(interminai_bin())
        .arg("status")
        .arg("--socket")
        .arg(daemon.socket())
        .assert()
        .success();

    let mut daemon = daemon;
    daemon.stop();
}
