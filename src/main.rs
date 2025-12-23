// 🌀 an Interactive Terminal for AI (interminai)
//
// Author: Michael S. Tsirkin <mst@kernel.org>
//
// A PTY-based tool for interacting with terminal applications (Rust version).

use clap::{Parser as ClapParser, Subcommand};
use anyhow::{Result, Context, bail};
use std::process::{Command as ProcessCommand};
use std::os::unix::process::CommandExt;
use tempfile::Builder;
use serde::{Deserialize, Serialize};
use std::os::unix::net::{UnixListener, UnixStream};
use std::io::{BufRead, BufReader, Write, Read};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use nix::pty::{openpty, Winsize};
use nix::unistd::{setsid, Pid};
use nix::sys::wait::{waitpid, WaitStatus, WaitPidFlag};
use nix::sys::signal::{kill, Signal};
use std::os::fd::{AsRawFd, OwnedFd};
use std::fs;
use std::path::Path;
use vte::Perform;


#[derive(ClapParser)]
#[command(name = "interminai")]
#[command(about = "🌀 an Interactive Terminal for AI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a new interactive terminal session
    Start {
        /// Unix socket path (auto-generated if not specified)
        #[arg(long)]
        socket: Option<String>,

        /// Terminal size (e.g., 80x24)
        #[arg(long, default_value = "80x24")]
        size: String,

        /// Run in foreground (for debugging/testing, default: daemon mode)
        #[arg(long)]
        no_daemon: bool,

        /// Command to run
        #[arg(required = true, last = true)]
        command: Vec<String>,
    },

    /// Send input to running session
    Input {
        /// Unix socket path (required)
        #[arg(long, required = true)]
        socket: String,

        /// Input text with escape sequences (alternative to stdin)
        /// Supports: \n \r \t \a \b \f \v \\ \e \xHH
        #[arg(long)]
        text: Option<String>,
    },

    /// Get screen output from running session
    Output {
        /// Unix socket path (required)
        #[arg(long, required = true)]
        socket: String,

        /// Output format (ascii or json)
        #[arg(long, default_value = "ascii")]
        format: String,

        /// Cursor display mode (none, inverse, print, both)
        #[arg(long, default_value = "none")]
        cursor: String,
    },

    /// Stop running session
    Stop {
        /// Unix socket path (required)
        #[arg(long, required = true)]
        socket: String,
    },

    /// Check if session is still running
    Running {
        /// Unix socket path (required)
        #[arg(long, required = true)]
        socket: String,
    },

    /// Wait until session exits
    Wait {
        /// Unix socket path (required)
        #[arg(long, required = true)]
        socket: String,
    },

    /// Send signal to running process
    Kill {
        /// Unix socket path (required)
        #[arg(long, required = true)]
        socket: String,

        /// Signal to send (named like SIGTERM, SIGKILL, SIGINT or numeric like 9, 15, 2)
        #[arg(long, default_value = "SIGTERM")]
        signal: String,
    },

    /// Resize the terminal
    Resize {
        /// Unix socket path (required)
        #[arg(long, required = true)]
        socket: String,

        /// New terminal size (e.g., 120x40)
        #[arg(long)]
        size: String,
    },
}

// Protocol messages
#[derive(Deserialize)]
struct Request {
    #[serde(rename = "type")]
    req_type: String,
    #[serde(flatten)]
    data: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct Response {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl Response {
    fn ok(data: serde_json::Value) -> Self {
        Response {
            status: "ok".to_string(),
            data: Some(data),
            error: None,
        }
    }

    fn error(msg: String) -> Self {
        Response {
            status: "error".to_string(),
            data: None,
            error: Some(msg),
        }
    }
}

// Simple terminal emulator
struct Screen {
    rows: usize,
    cols: usize,
    cells: Vec<Vec<char>>,
    cursor_row: usize,
    cursor_col: usize,
}

impl Screen {
    fn new(rows: usize, cols: usize) -> Self {
        Screen {
            rows,
            cols,
            cells: vec![vec![' '; cols]; rows],
            cursor_row: 0,
            cursor_col: 0,
        }
    }

    fn to_ascii(&self) -> String {
        let mut result = String::new();
        for row in &self.cells {
            let line: String = row.iter().collect();
            result.push_str(&line.trim_end());
            result.push('\n');
        }
        result
    }

    fn scroll_up(&mut self) {
        // Remove the top row and add a blank row at the bottom
        self.cells.remove(0);
        self.cells.push(vec![' '; self.cols]);
    }
}

impl Perform for Screen {
    fn print(&mut self, c: char) {
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            self.cells[self.cursor_row][self.cursor_col] = c;
            self.cursor_col += 1;
            if self.cursor_col >= self.cols {
                self.cursor_col = 0;
                self.cursor_row += 1;
                if self.cursor_row >= self.rows {
                    self.scroll_up();
                    self.cursor_row = self.rows - 1;
                }
            }
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.cursor_row += 1;
                if self.cursor_row >= self.rows {
                    self.scroll_up();
                    self.cursor_row = self.rows - 1;
                }
                self.cursor_col = 0;
            }
            b'\r' => {
                self.cursor_col = 0;
            }
            b'\t' => {
                self.cursor_col = ((self.cursor_col / 8) + 1) * 8;
                if self.cursor_col >= self.cols {
                    self.cursor_col = self.cols - 1;
                }
            }
            b'\x08' => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
            }
            _ => {}
        }
    }

    fn hook(&mut self, _: &vte::Params, _: &[u8], _: bool, _: char) {}
    fn put(&mut self, _: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _: &[&[u8]], _: bool) {}
    fn csi_dispatch(&mut self, params: &vte::Params, _intermediates: &[u8], _ignore: bool, action: char) {
        match action {
            'H' | 'f' => {
                // Cursor position
                let row = params.iter().nth(0).and_then(|p| p.first()).copied().unwrap_or(1).saturating_sub(1) as usize;
                let col = params.iter().nth(1).and_then(|p| p.first()).copied().unwrap_or(1).saturating_sub(1) as usize;
                self.cursor_row = row.min(self.rows - 1);
                self.cursor_col = col.min(self.cols - 1);
            }
            'A' => {
                // Cursor up
                let n = params.iter().nth(0).and_then(|p| p.first()).copied().unwrap_or(1).max(1) as usize;
                self.cursor_row = self.cursor_row.saturating_sub(n);
            }
            'B' => {
                // Cursor down
                let n = params.iter().nth(0).and_then(|p| p.first()).copied().unwrap_or(1).max(1) as usize;
                self.cursor_row = (self.cursor_row + n).min(self.rows - 1);
            }
            'C' => {
                // Cursor forward
                let n = params.iter().nth(0).and_then(|p| p.first()).copied().unwrap_or(1).max(1) as usize;
                self.cursor_col = (self.cursor_col + n).min(self.cols - 1);
            }
            'D' => {
                // Cursor back
                let n = params.iter().nth(0).and_then(|p| p.first()).copied().unwrap_or(1).max(1) as usize;
                self.cursor_col = self.cursor_col.saturating_sub(n);
            }
            'J' => {
                // Erase display
                let mode = params.iter().nth(0).and_then(|p| p.first()).copied().unwrap_or(0);
                match mode {
                    0 => {
                        // Clear from cursor to end
                        for col in self.cursor_col..self.cols {
                            self.cells[self.cursor_row][col] = ' ';
                        }
                        for row in (self.cursor_row + 1)..self.rows {
                            for col in 0..self.cols {
                                self.cells[row][col] = ' ';
                            }
                        }
                    }
                    2 => {
                        // Clear entire screen
                        for row in 0..self.rows {
                            for col in 0..self.cols {
                                self.cells[row][col] = ' ';
                            }
                        }
                        self.cursor_row = 0;
                        self.cursor_col = 0;
                    }
                    _ => {}
                }
            }
            'K' => {
                // Erase line
                let mode = params.iter().nth(0).and_then(|p| p.first()).copied().unwrap_or(0);
                match mode {
                    0 => {
                        // Clear from cursor to end of line
                        for col in self.cursor_col..self.cols {
                            self.cells[self.cursor_row][col] = ' ';
                        }
                    }
                    2 => {
                        // Clear entire line
                        for col in 0..self.cols {
                            self.cells[self.cursor_row][col] = ' ';
                        }
                    }
                    _ => {}
                }
            }
            'M' => {
                // Delete Line (DL) - used by vim when deleting lines
                let n = params.iter().nth(0).and_then(|p| p.first()).copied().unwrap_or(1).max(1) as usize;
                for _ in 0..n {
                    if self.cursor_row < self.rows {
                        // Remove current line
                        self.cells.remove(self.cursor_row);
                        // Add blank line at bottom
                        self.cells.push(vec![' '; self.cols]);
                    }
                }
            }
            'L' => {
                // Insert Line (IL) - used by vim when inserting lines
                let n = params.iter().nth(0).and_then(|p| p.first()).copied().unwrap_or(1).max(1) as usize;
                for _ in 0..n {
                    if self.cursor_row < self.rows {
                        // Remove bottom line
                        self.cells.pop();
                        // Insert blank line at cursor position
                        self.cells.insert(self.cursor_row, vec![' '; self.cols]);
                    }
                }
            }
            'm' => {
                // SGR - ignore for now (colors/attributes)
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

struct DaemonState {
    master_fd: OwnedFd,
    child_pid: Pid,
    screen: Screen,
    parser: vte::Parser,
    exit_code: Option<i32>,
    socket_path: String,
    socket_was_auto_generated: bool,
    should_shutdown: bool,
}

impl DaemonState {
    fn check_child_status(&mut self) {
        if self.exit_code.is_some() {
            return;
        }

        match waitpid(self.child_pid, Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::Exited(_, code)) => {
                self.exit_code = Some(code);
            }
            Ok(WaitStatus::Signaled(_, sig, _)) => {
                self.exit_code = Some(128 + sig as i32);
            }
            _ => {}
        }
    }

    fn read_pty_output(&mut self) {
        let mut buf = [0u8; 4096];
        loop {
            match nix::unistd::read(self.master_fd.as_raw_fd(), &mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    for byte in &buf[..n] {
                        self.parser.advance(&mut self.screen, *byte);
                    }
                }
                Err(_) => break,
            }
        }
    }
}

fn parse_terminal_size(size: &str) -> Result<(u16, u16)> {
    let parts: Vec<&str> = size.split('x').collect();
    if parts.len() != 2 {
        bail!("Invalid size format, expected WxH like 80x24");
    }
    let cols = parts[0].parse::<u16>().context("Invalid columns")?;
    let rows = parts[1].parse::<u16>().context("Invalid rows")?;
    Ok((cols, rows))
}

/// Unescape C-style escape sequences in a string.
/// Supports: \n \r \t \a \b \f \v \\ \e \xHH
fn unescape(s: &str) -> Result<String> {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('a') => result.push('\x07'),  // bell
                Some('b') => result.push('\x08'),  // backspace
                Some('f') => result.push('\x0C'),  // form feed (Ctrl+L)
                Some('v') => result.push('\x0B'),  // vertical tab
                Some('\\') => result.push('\\'),
                Some('e') | Some('E') => result.push('\x1B'),  // ESC
                Some('x') => {
                    // Parse two hex digits
                    let h1 = chars.next().ok_or_else(|| anyhow::anyhow!("incomplete \\x escape"))?;
                    let h2 = chars.next().ok_or_else(|| anyhow::anyhow!("incomplete \\x escape"))?;
                    let hex_str: String = [h1, h2].iter().collect();
                    let byte = u8::from_str_radix(&hex_str, 16)
                        .context(format!("invalid hex escape: \\x{}", hex_str))?;
                    result.push(byte as char);
                }
                Some(other) => {
                    // Unknown escape - keep as-is
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

fn parse_signal(sig: &str) -> Result<Signal> {
    // Try parsing as number first
    if let Ok(num) = sig.parse::<i32>() {
        return Signal::try_from(num).context("Invalid signal number");
    }

    // Parse named signals
    let sig_upper = sig.to_uppercase();
    let sig_name = if sig_upper.starts_with("SIG") {
        sig_upper.as_str()
    } else {
        // Add SIG prefix if not present
        return parse_signal(&format!("SIG{}", sig));
    };

    match sig_name {
        "SIGHUP" => Ok(Signal::SIGHUP),
        "SIGINT" => Ok(Signal::SIGINT),
        "SIGQUIT" => Ok(Signal::SIGQUIT),
        "SIGKILL" => Ok(Signal::SIGKILL),
        "SIGTERM" => Ok(Signal::SIGTERM),
        "SIGUSR1" => Ok(Signal::SIGUSR1),
        "SIGUSR2" => Ok(Signal::SIGUSR2),
        _ => bail!("Unknown signal: {}", sig),
    }
}

fn auto_generate_socket_path() -> Result<String> {
    // Create a temporary directory with proper prefix
    let temp_dir = Builder::new()
        .prefix("interminai-")
        .tempdir()?;

    // Get the path and keep it persistent (leak the TempDir)
    let socket_path = temp_dir.path().join("socket").to_string_lossy().to_string();

    // Leak the temp dir so it doesn't get deleted
    std::mem::forget(temp_dir);

    Ok(socket_path)
}

fn cmd_start(socket: Option<String>, size: String, daemon: bool, command: Vec<String>) -> Result<()> {
    let socket_was_auto_generated = socket.is_none();
    let socket_path = match socket {
        Some(path) => path,
        None => auto_generate_socket_path()?,
    };

    let (cols, rows) = parse_terminal_size(&size)?;

    if !daemon {
        // Run in foreground (default for now)
        println!("Socket: {}", socket_path);
        println!("PID: {}", std::process::id());
        println!("Auto-generated: {}", socket_was_auto_generated);

        return run_daemon(socket_path, socket_was_auto_generated, rows, cols, command);
    }

    // Double-fork to properly daemonize
    // Use fork crate which provides a safe wrapper around libc::fork()
    use fork::{fork as safe_fork, Fork};

    match safe_fork() {
        Ok(Fork::Parent(child)) => {
            // Parent process: wait for intermediate child to exit (avoid zombie)
            use nix::sys::wait::waitpid;
            use nix::unistd::Pid;
            let _ = waitpid(Pid::from_raw(child), None);

            // The intermediate child has printed the grandchild PID to stdout
            // Now print the rest of the info
            println!("Socket: {}", socket_path);
            println!("Auto-generated: {}", socket_was_auto_generated);
            Ok(())
        }
        Ok(Fork::Child) => {
            // Intermediate child: fork again, print grandchild PID, and exit
            match safe_fork() {
                Ok(Fork::Parent(grandchild_pid)) => {
                    // Intermediate parent: print grandchild PID to stdout and exit
                    println!("PID: {}", grandchild_pid);
                    std::process::exit(0);
                }
                Ok(Fork::Child) => {
                    // Grandchild: become daemon
                    setsid().expect("Failed to create new session");

                    // Redirect stdin/stdout/stderr to /dev/null (standard daemon behavior)
                    // Note: Programs running in the PTY are unaffected - they get their own
                    // stdin/stdout/stderr connected to the PTY slave
                    use std::fs::OpenOptions;
                    use std::os::unix::io::AsRawFd;
                    use nix::unistd::dup2;

                    // Open /dev/null - use std::fs for safe file opening
                    match OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open("/dev/null")
                    {
                        Ok(devnull) => {
                            let devnull_fd = devnull.as_raw_fd();

                            // Redirect stdin/stdout/stderr to /dev/null using safe dup2 wrapper
                            // dup2 handles both open and closed fds - it closes them first if needed
                            let _ = dup2(devnull_fd, 0);
                            let _ = dup2(devnull_fd, 1);
                            let _ = dup2(devnull_fd, 2);

                            // devnull will be automatically closed when it goes out of scope
                        }
                        Err(_) => {
                            // If we can't open /dev/null, continue anyway
                            // The daemon will just have closed fds
                        }
                    }

                    // Run daemon
                    if let Err(e) = run_daemon(socket_path, socket_was_auto_generated, rows, cols, command) {
                        // Daemon errors go to /dev/null in daemon mode, which is fine
                        eprintln!("Daemon error: {}", e);
                        std::process::exit(1);
                    }
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Second fork failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            bail!("Failed to fork: {}", e)
        }
    }
}

fn run_daemon(socket_path: String, socket_was_auto_generated: bool, rows: u16, cols: u16, command: Vec<String>) -> Result<()> {
    // Create PTY
    let winsize = Winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    let pty = openpty(Some(&winsize), None)?;

    // Fork to spawn child in PTY
    // Use fork crate which provides a safe wrapper around libc::fork()
    use fork::{fork as safe_fork, Fork};

    match safe_fork() {
        Ok(Fork::Parent(child)) => {
            // Close slave side in parent by dropping it (not using close() to avoid double-close)
            drop(pty.slave);

            // Set master to non-blocking
            // Use nix's safe fcntl wrapper (requires 'fs' feature)
            use nix::fcntl::{fcntl, FcntlArg, OFlag};

            let flags = fcntl(pty.master.as_raw_fd(), FcntlArg::F_GETFL)
                .context("Failed to get PTY flags")?;
            let mut oflags = OFlag::from_bits_truncate(flags);
            oflags.insert(OFlag::O_NONBLOCK);
            fcntl(pty.master.as_raw_fd(), FcntlArg::F_SETFL(oflags))
                .context("Failed to set PTY non-blocking")?;

            // Create state
            let state = Arc::new(Mutex::new(DaemonState {
                master_fd: pty.master,
                child_pid: Pid::from_raw(child),
                screen: Screen::new(rows as usize, cols as usize),
                parser: vte::Parser::new(),
                exit_code: None,
                socket_path: socket_path.clone(),
                socket_was_auto_generated,
                should_shutdown: false,
            }));

            // Start PTY reader thread
            let state_clone = state.clone();
            thread::spawn(move || {
                loop {
                    thread::sleep(Duration::from_millis(50));
                    let mut state = state_clone.lock().unwrap();
                    state.check_child_status();
                    state.read_pty_output();

                    if state.exit_code.is_some() {
                        break;
                    }
                }
            });

            // Create socket and listen
            let _ = fs::remove_file(&socket_path); // Clean up if exists
            let listener = UnixListener::bind(&socket_path)?;

            // Set socket to non-blocking so we can check shutdown flag
            listener.set_nonblocking(true)?;

            // Accept connections
            loop {
                // Check if we should exit
                {
                    let state_locked = state.lock().unwrap();
                    if state_locked.should_shutdown {
                        break;
                    }
                }

                match listener.accept() {
                    Ok((stream, _)) => {
                        // Process commands sequentially - no parallelism
                        if let Err(e) = handle_client(stream, state.clone()) {
                            eprintln!("Client handler error: {}", e);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No connection available, sleep and try again
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(e) => {
                        eprintln!("Connection error: {}", e);
                    }
                }
            }

            // Give time for final requests to complete
            thread::sleep(Duration::from_millis(200));

            // Cleanup
            let state_locked = state.lock().unwrap();
            if state_locked.socket_was_auto_generated {
                let _ = fs::remove_file(&state_locked.socket_path);
                // Also remove the parent directory (the temp dir)
                if let Some(parent) = Path::new(&state_locked.socket_path).parent() {
                    let _ = fs::remove_dir(parent);
                }
            }

            Ok(())
        }
        Ok(Fork::Child) => {
            // Close master side in child by dropping it
            drop(pty.master);

            // Create new session - this makes the child a session leader
            // This is required for the PTY slave to become the controlling terminal
            setsid().context("Failed to create new session")?;

            // Redirect stdin/stdout/stderr to slave using nix
            use nix::unistd::dup2;
            let slave_fd = pty.slave.as_raw_fd();
            dup2(slave_fd, 0).context("Failed to dup2 stdin")?;
            dup2(slave_fd, 1).context("Failed to dup2 stdout")?;
            dup2(slave_fd, 2).context("Failed to dup2 stderr")?;

            // Make the PTY slave the controlling terminal for this session
            // TIOCSCTTY = "set controlling tty" - this must be done AFTER setsid()
            // and AFTER making stdin/stdout/stderr point to the slave
            if let Err(e) = rustix::process::ioctl_tiocsctty(&pty.slave) {
                // Non-fatal - continue anyway
                eprintln!("Warning: Failed to set controlling terminal: {}", e);
            }

            // Drop slave after dup2 (automatically closes it)
            drop(pty.slave);

            // Unset TERM to force applications to use basic escape sequences that our
            // simple terminal emulator can handle. With TERM set (e.g. xterm-256color),
            // vim uses advanced features like scroll regions which we don't support,
            // causing the screen display to desync from the actual buffer.
            std::env::remove_var("TERM");

            // Exec command
            let program = &command[0];
            let args = &command[1..];

            let _ = ProcessCommand::new(program)
                .args(args)
                .exec();

            std::process::exit(1);
        }
        Err(e) => bail!("Failed to fork for child: {}", e),
    }
}

fn handle_client(mut stream: UnixStream, state: Arc<Mutex<DaemonState>>) -> Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();

    // Read line, checking for EOF (client disconnected before sending complete request)
    let bytes_read = reader.read_line(&mut line)?;
    if bytes_read == 0 {
        // EOF - client disconnected without sending complete request
        return Ok(());
    }

    let request: Request = match serde_json::from_str(&line) {
        Ok(req) => req,
        Err(e) => {
            let response = Response::error(format!("Invalid JSON: {}", e));
            write_response(&mut stream, &response)?;
            return Ok(());
        }
    };

    let response = match request.req_type.as_str() {
        "INPUT" => handle_input(request.data, &state),
        "OUTPUT" => handle_output(request.data, &state),
        "RUNNING" => handle_running(&state),
        "WAIT" => handle_wait(&state, &stream),
        "KILL" => handle_kill(request.data, &state),
        "STOP" => handle_stop(&state),
        "RESIZE" => handle_resize(request.data, &state),
        _ => Response::error(format!("Unknown command: {}", request.req_type)),
    };

    write_response(&mut stream, &response)?;

    Ok(())
}

fn write_response(stream: &mut UnixStream, response: &Response) -> Result<()> {
    let json = serde_json::to_string(response)?;
    stream.write_all(json.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;
    Ok(())
}

fn handle_input(data: serde_json::Value, state: &Arc<Mutex<DaemonState>>) -> Response {
    let input_data = match data.get("data").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Response::error("Missing 'data' field".to_string()),
    };

    let state = state.lock().unwrap();

    match nix::unistd::write(state.master_fd.as_raw_fd(), input_data.as_bytes()) {
        Ok(_) => Response::ok(serde_json::json!({})),
        Err(e) => Response::error(format!("Failed to write to PTY: {}", e)),
    }
}

fn handle_output(data: serde_json::Value, state: &Arc<Mutex<DaemonState>>) -> Response {
    let _format = data.get("format").and_then(|v| v.as_str()).unwrap_or("ascii");

    let mut state = state.lock().unwrap();
    state.read_pty_output();

    let screen_text = state.screen.to_ascii();

    let data = serde_json::json!({
        "screen": screen_text,
        "cursor": {
            "row": state.screen.cursor_row,
            "col": state.screen.cursor_col
        },
        "size": {
            "rows": state.screen.rows,
            "cols": state.screen.cols
        }
    });

    Response::ok(data)
}

fn handle_running(state: &Arc<Mutex<DaemonState>>) -> Response {
    let mut state = state.lock().unwrap();
    state.check_child_status();

    if let Some(exit_code) = state.exit_code {
        Response::ok(serde_json::json!({
            "running": false,
            "exit_code": exit_code
        }))
    } else {
        Response::ok(serde_json::json!({
            "running": true
        }))
    }
}

fn handle_wait(state: &Arc<Mutex<DaemonState>>, stream: &UnixStream) -> Response {
    use rustix::net::{recv, RecvFlags};

    loop {
        // Check if client disconnected using recv with MSG_PEEK | MSG_DONTWAIT
        let mut buf = [0u8; 1];
        let flags = RecvFlags::PEEK | RecvFlags::DONTWAIT;
        match recv(stream, &mut buf, flags) {
            Ok((_, 0)) => {
                // EOF - client disconnected
                return Response::error("Client disconnected".to_string());
            }
            Ok(_) => {
                // Unexpected data from client - ignore
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data, client still connected - continue waiting
            }
            Err(_) => {
                // Real error - assume client disconnected
                return Response::error("Client disconnected".to_string());
            }
        }

        {
            let mut state = state.lock().unwrap();
            state.check_child_status();

            if let Some(exit_code) = state.exit_code {
                return Response::ok(serde_json::json!({
                    "exit_code": exit_code
                }));
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

fn handle_kill(data: serde_json::Value, state: &Arc<Mutex<DaemonState>>) -> Response {
    let signal_str = match data.get("signal").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Response::error("Missing 'signal' field".to_string()),
    };

    let signal = match parse_signal(signal_str) {
        Ok(sig) => sig,
        Err(e) => return Response::error(format!("Invalid signal: {}", e)),
    };

    let state = state.lock().unwrap();

    match kill(state.child_pid, signal) {
        Ok(_) => Response::ok(serde_json::json!({
            "signal_sent": signal_str
        })),
        Err(e) => Response::error(format!("Failed to send signal: {}", e)),
    }
}

fn handle_stop(state: &Arc<Mutex<DaemonState>>) -> Response {
    let mut state = state.lock().unwrap();

    // Kill child if still running
    if state.exit_code.is_none() {
        let _ = kill(state.child_pid, Signal::SIGTERM);
    }

    // Set shutdown flag to exit daemon loop
    state.should_shutdown = true;

    Response::ok(serde_json::json!({
        "message": "Shutting down"
    }))
}

fn handle_resize(data: serde_json::Value, state: &Arc<Mutex<DaemonState>>) -> Response {
    let cols = match data.get("cols").and_then(|v| v.as_u64()) {
        Some(c) => c as u16,
        None => return Response::error("Missing 'cols' field".to_string()),
    };

    let rows = match data.get("rows").and_then(|v| v.as_u64()) {
        Some(r) => r as u16,
        None => return Response::error("Missing 'rows' field".to_string()),
    };

    let mut state = state.lock().unwrap();

    // Send TIOCSWINSZ to update terminal size using rustix's safe wrapper
    use rustix::termios::{tcsetwinsize, Winsize as RustixWinsize};

    let winsize = RustixWinsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    if let Err(_) = tcsetwinsize(&state.master_fd, winsize) {
        return Response::error("Failed to resize terminal".to_string());
    }

    // Update screen buffer dimensions
    // Create new screen with new dimensions
    let mut new_screen = Screen::new(rows as usize, cols as usize);

    // Copy old content to new screen (preserve as much as possible)
    let old_screen = &state.screen;
    for row in 0..old_screen.rows.min(new_screen.rows) {
        for col in 0..old_screen.cols.min(new_screen.cols) {
            new_screen.cells[row][col] = old_screen.cells[row][col];
        }
    }
    new_screen.cursor_row = old_screen.cursor_row.min(new_screen.rows - 1);
    new_screen.cursor_col = old_screen.cursor_col.min(new_screen.cols - 1);

    state.screen = new_screen;

    Response::ok(serde_json::json!({
        "cols": cols,
        "rows": rows
    }))
}

fn apply_cursor_inverse(screen: &str, cursor_row: usize, cursor_col: usize) -> String {
    let lines: Vec<&str> = screen.lines().collect();
    
    // Check if cursor_row is valid
    if cursor_row >= lines.len() {
        return screen.to_string();
    }
    
    let mut result = String::new();
    
    for (row_idx, line) in lines.iter().enumerate() {
        if row_idx == cursor_row {
            let chars: Vec<char> = line.chars().collect();
            
            // Check if cursor_col is valid
            if cursor_col >= chars.len() {
                result.push_str(line);
            } else {
                // Build the line with inverse video at cursor position
                for (col_idx, ch) in chars.iter().enumerate() {
                    if col_idx == cursor_col {
                        result.push_str("\x1b[7m"); // Start inverse video
                        result.push(*ch);
                        result.push_str("\x1b[27m"); // End inverse video
                    } else {
                        result.push(*ch);
                    }
                }
            }
        } else {
            result.push_str(line);
        }
        
        if row_idx < lines.len() - 1 {
            result.push('\n');
        }
    }
    
    result
}

fn send_request(socket_path: &str, request: serde_json::Value) -> Result<Response> {
    let mut stream = UnixStream::connect(socket_path)
        .context("Failed to connect to daemon socket")?;

    let json = serde_json::to_string(&request)?;
    stream.write_all(json.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;

    let response: Response = serde_json::from_str(&line)?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_cursor_inverse_basic() {
        let screen = "Hello World\nSecond Line";
        let result = apply_cursor_inverse(screen, 0, 6);
        
        // Should have inverse codes around character at position 6 (the 'W')
        assert!(result.contains("\x1b[7mW\x1b[27m"), "Should wrap 'W' with inverse codes");
        assert!(result.contains("Hello"));
        assert!(result.contains("orld")); // After the wrapped W
        assert!(result.contains("Second Line"));
    }

    #[test]
    fn test_apply_cursor_inverse_first_char() {
        let screen = "Test";
        let result = apply_cursor_inverse(screen, 0, 0);
        
        // Should start with inverse code
        assert!(result.starts_with("\x1b[7m"));
        assert!(result.contains("\x1b[27m"));
    }

    #[test]
    fn test_apply_cursor_inverse_multiline() {
        let screen = "Line 1\nLine 2\nLine 3";
        let result = apply_cursor_inverse(screen, 1, 5);
        
        // Should have all lines
        assert!(result.contains("Line 1"));
        assert!(result.contains("Line ")); // Before wrapped character
        assert!(result.contains("Line 3"));
        
        // Should have inverse codes wrapping character at position 5 of line 1 (the '2')
        assert!(result.contains("\x1b[7m2\x1b[27m"));
    }

    #[test]
    fn test_apply_cursor_inverse_invalid_row() {
        let screen = "Only one line";
        let result = apply_cursor_inverse(screen, 5, 0);
        
        // Should return original screen unchanged
        assert_eq!(result, screen);
    }

    #[test]
    fn test_apply_cursor_inverse_invalid_col() {
        let screen = "Short";
        let result = apply_cursor_inverse(screen, 0, 100);
        
        // Should return original line (no inverse codes)
        assert!(!result.contains("\x1b[7m"));
        assert!(result.contains("Short"));
    }

    #[test]
    fn test_apply_cursor_inverse_empty_screen() {
        let screen = "";
        let result = apply_cursor_inverse(screen, 0, 0);
        
        // Should handle gracefully
        assert_eq!(result, screen);
    }

    #[test]
    fn test_apply_cursor_inverse_preserves_all_chars() {
        let screen = "ABCDEFGHIJKLMNOP";
        let result = apply_cursor_inverse(screen, 0, 7);
        
        // Strip ANSI codes
        let stripped = result
            .replace("\x1b[7m", "")
            .replace("\x1b[27m", "");
        
        // All characters should be preserved
        assert_eq!(stripped, screen);
    }

    #[test]
    fn test_apply_cursor_inverse_last_char() {
        let screen = "Test";
        let result = apply_cursor_inverse(screen, 0, 3);
        
        // Should end with inverse codes and then the 't'
        assert!(result.contains("\x1b[7mt\x1b[27m"));
    }

    #[test]
    fn test_apply_cursor_inverse_special_chars() {
        let screen = "Hello\tWorld\nNext";
        let result = apply_cursor_inverse(screen, 0, 5);
        
        // Should handle tab character
        assert!(result.contains("\x1b[7m\t\x1b[27m"));
        assert!(result.contains("Hello"));
        assert!(result.contains("World"));
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { socket, size, no_daemon, command } => {
            cmd_start(socket, size, !no_daemon, command)?;
        }
        Commands::Input { socket, text } => {
            // Use --text if provided, otherwise read from stdin
            let input = if let Some(text_arg) = text {
                unescape(&text_arg)?
            } else {
                let mut buf = String::new();
                std::io::stdin().read_to_string(&mut buf)?;
                buf
            };

            let request = serde_json::json!({
                "type": "INPUT",
                "data": input
            });

            let response = send_request(&socket, request)?;

            if response.status == "error" {
                eprintln!("Error: {}", response.error.unwrap_or_default());
                std::process::exit(1);
            }
        }
        Commands::Output { socket, format, cursor } => {
            let request = serde_json::json!({
                "type": "OUTPUT",
                "format": format
            });

            let response = send_request(&socket, request)?;

            if response.status == "error" {
                eprintln!("Error: {}", response.error.unwrap_or_default());
                std::process::exit(1);
            }

            if let Some(data) = response.data {
                let cursor_mode = cursor.as_str();
                
                // Print cursor info if requested (convert to 1-based for display)
                if cursor_mode == "print" || cursor_mode == "both" {
                    if let (Some(cursor_row), Some(cursor_col)) = (
                        data.get("cursor").and_then(|c| c.get("row")).and_then(|v| v.as_u64()),
                        data.get("cursor").and_then(|c| c.get("col")).and_then(|v| v.as_u64())
                    ) {
                        println!("Cursor: row {}, col {}", cursor_row + 1, cursor_col + 1);
                    }
                }

                if let Some(screen) = data.get("screen").and_then(|v| v.as_str()) {
                    // Apply inverse video if requested
                    if cursor_mode == "inverse" || cursor_mode == "both" {
                        if let (Some(cursor_row), Some(cursor_col)) = (
                            data.get("cursor").and_then(|c| c.get("row")).and_then(|v| v.as_u64()),
                            data.get("cursor").and_then(|c| c.get("col")).and_then(|v| v.as_u64())
                        ) {
                            print!("{}", apply_cursor_inverse(screen, cursor_row as usize, cursor_col as usize));
                        } else {
                            print!("{}", screen);
                        }
                    } else {
                        print!("{}", screen);
                    }
                }
            }
        }
        Commands::Running { socket } => {
            let request = serde_json::json!({
                "type": "RUNNING"
            });

            let response = send_request(&socket, request)?;

            if response.status == "error" {
                eprintln!("Error: {}", response.error.unwrap_or_default());
                std::process::exit(1);
            }

            if let Some(data) = response.data {
                let running = data.get("running").and_then(|v| v.as_bool()).unwrap_or(false);

                if running {
                    // Exit 0 if running
                    std::process::exit(0);
                } else {
                    // Print exit code and exit 1 if not running
                    if let Some(exit_code) = data.get("exit_code") {
                        println!("{}", exit_code);
                    }
                    std::process::exit(1);
                }
            }
        }
        Commands::Wait { socket } => {
            let request = serde_json::json!({
                "type": "WAIT"
            });

            let response = send_request(&socket, request)?;

            if response.status == "error" {
                eprintln!("Error: {}", response.error.unwrap_or_default());
                std::process::exit(1);
            }

            if let Some(data) = response.data {
                if let Some(exit_code) = data.get("exit_code") {
                    println!("{}", exit_code);
                }
            }
        }
        Commands::Kill { socket, signal } => {
            let request = serde_json::json!({
                "type": "KILL",
                "signal": signal
            });

            let response = send_request(&socket, request)?;

            if response.status == "error" {
                eprintln!("Error: {}", response.error.unwrap_or_default());
                std::process::exit(1);
            }
        }
        Commands::Stop { socket } => {
            let request = serde_json::json!({
                "type": "STOP"
            });

            let response = send_request(&socket, request)?;

            if response.status == "error" {
                eprintln!("Error: {}", response.error.unwrap_or_default());
                std::process::exit(1);
            }
        }

        Commands::Resize { socket, size } => {
            // Parse and validate size
            let (cols, rows) = parse_terminal_size(&size)?;

            let request = serde_json::json!({
                "type": "RESIZE",
                "cols": cols,
                "rows": rows
            });

            let response = send_request(&socket, request)?;

            if response.status == "error" {
                eprintln!("Error: {}", response.error.unwrap_or_default());
                std::process::exit(1);
            }

            println!("Terminal resized to {}x{}", cols, rows);
        }
    }

    Ok(())
}
