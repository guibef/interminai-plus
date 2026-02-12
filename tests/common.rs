use std::time::Duration;
use std::thread;
use tempfile::TempDir;
use std::path::PathBuf;

/// Get the interminai binary path to use for testing.
pub fn interminai_bin() -> String {
    std::env::var("OVERRIDE_CARGO_BIN_EXE_interminai")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_interminai").to_string())
}

/// Get the interminai server binary (for start command).
#[allow(dead_code)]
pub fn interminai_server_bin() -> String {
    std::env::var("OVERRIDE_CARGO_BIN_EXE_interminai_SERVER")
        .unwrap_or_else(|_| interminai_bin())
}

/// Get the interminai client binary (for output, input, stop, etc.).
#[allow(dead_code)]
pub fn interminai_client_bin() -> String {
    std::env::var("OVERRIDE_CARGO_BIN_EXE_interminai_CLIENT")
        .unwrap_or_else(|_| interminai_bin())
}

/// Get the terminal emulator to use for testing.
#[allow(dead_code)]
pub fn emulator() -> String {
    std::env::var("INTERMINAI_EMULATOR").unwrap_or_else(|_| "xterm".to_string())
}

/// Get emulator arguments to pass to the start command.
#[allow(dead_code)]
pub fn emulator_args() -> Vec<String> {
    vec!["--emulator".to_string(), emulator()]
}

/// Helper to create a test environment with temporary directory and socket
#[allow(dead_code)]
pub struct TestEnv {
    _temp_dir: TempDir,
    socket_path: PathBuf,
}

impl TestEnv {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let socket_path = temp_dir.path().join("test.sock");

        Self {
            _temp_dir: temp_dir,
            socket_path,
        }
    }

    #[allow(dead_code)]
    pub fn socket(&self) -> String {
        self.socket_path.to_str().unwrap().to_string()
    }
}

// Helper to spawn daemon in foreground and manage its lifecycle
#[allow(dead_code)]
pub struct DaemonHandle {
    pub child: std::process::Child,
    pub socket_path: String,
}

impl DaemonHandle {
    #[allow(dead_code)]
    pub fn spawn(args: &[&str]) -> Self {
        use std::process::Stdio;
        use std::io::BufRead;

        let mut cmd = std::process::Command::new(interminai_bin());
        cmd.arg("start").args(emulator_args());

        for arg in args {
            cmd.arg(arg);
        }

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn daemon");

        let stdout = child.stdout.take().unwrap();
        let reader = std::io::BufReader::new(stdout);
        let lines: Vec<String> = reader.lines().take(3).map(|l| l.unwrap()).collect();

        let socket_line = lines.iter().find(|l| l.starts_with("Socket:")).expect("No socket line");
        let socket_path = socket_line.split_whitespace().nth(1).unwrap().to_string();

        thread::sleep(Duration::from_millis(300));

        DaemonHandle { child, socket_path }
    }

    #[allow(dead_code)]
    pub fn spawn_with_socket(socket: &str, command_args: &[&str]) -> Self {
        use std::process::Stdio;
        use std::io::BufRead;

        let mut cmd = std::process::Command::new(interminai_bin());
        cmd.arg("start")
            .args(emulator_args())
            .arg("--socket")
            .arg(socket)
            .arg("--no-daemon")
            .arg("--");

        for arg in command_args {
            cmd.arg(arg);
        }

        let mut child = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn daemon");

        let stdout = child.stdout.take().unwrap();
        let reader = std::io::BufReader::new(stdout);
        let _lines: Vec<String> = reader.lines().take(3).map(|l| l.unwrap()).collect();

        thread::sleep(Duration::from_millis(300));

        DaemonHandle {
            child,
            socket_path: socket.to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn socket(&self) -> &str {
        &self.socket_path
    }

    #[allow(dead_code)]
    pub fn stop(&mut self) {
        let _ = std::process::Command::new(interminai_bin())
            .arg("stop")
            .arg("--socket")
            .arg(&self.socket_path)
            .status();

        thread::sleep(Duration::from_millis(200));
        let _ = self.child.wait();
    }
}
