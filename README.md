# interminai-plus: A fork of interminai, an interactive terminal for AI

A terminal proxy that enables programmatic interaction with interactive CLI applications.

## What It Does

Many powerful CLI tools require human interaction - `vim` waits for keystrokes, `git rebase -i` opens an editor, `apt` asks for confirmation, TUI applications respond to keyboard input. These tools can't be automated with simple shell scripts because they:

- Open full-screen interfaces
- Wait for user input
- Show dynamic menus and prompts
- Require terminal emulation

**interminai** solves this by wrapping any interactive program in a pseudo-terminal (PTY), capturing its screen output as text, and providing a simple API to send input and read the display. This allows AI agents, scripts, or automated systems to interact with vim, git, debuggers, configuration wizards, and any other interactive terminal application.

### Core Capabilities

- **Screen capture**: Read the current terminal display as ASCII text
- **Input control**: Send keystrokes, commands, and control sequences
- **Process management**: Start, monitor, signal, and stop wrapped processes
- **Daemon mode**: Run in background for long-lived interactive sessions
- **Terminal emulation**: Basic PTY with ANSI escape sequence handling

### Use Cases

- AI agents editing files with `vim`
- Automated git operations (`git rebase -i`, `git add -i`, `git commit`)
- Interactive package management (`apt`, `yum`)
- Debugging with `gdb` or `lldb`
- Configuration wizards (`rclone`, `raspi-config`)
- TUI applications (`htop`, `tmux`, `screen`)
- Any CLI tool that requires keyboard interaction

## Installation

**Prerequisites:**
- Rust toolchain (rustc 1.70+, cargo)
- Linux or macOS (requires PTY support)

**Build from source:**

```bash
# Clone the repository
git clone https://github.com/guibef/interminai-plus.git
cd interminai-plus

# Build release binary
cargo build --release

# Binary will be at: target/release/interminai

# Check available commands
./target/release/interminai --help
```

## Quick Start

```bash
# Start vim editing a file
interminai start --socket /tmp/vim.sock -- vim myfile.txt

# Send keystrokes (enter insert mode, type text, escape, save)
interminai input --socket /tmp/vim.sock --text "iHello, World!\e:wq\n"

# View the screen
interminai output --socket /tmp/vim.sock

# Stop the daemon
interminai stop --socket /tmp/vim.sock
```

## Documentation

- **[reference.md](docs/reference.md)** - Complete command reference
- **[examples.md](docs/examples.md)** - Detailed usage examples (vim, git, debugging)
- **[PROTOCOL.md](docs/PROTOCOL.md)** - Socket communication protocol specification
- **[SKILL.md](.agents/skills/interminai/SKILL.md)** - Agent skill documentation and best practices

## Commands

```bash
# Start an interactive program (runs as daemon by default)
interminai start [--socket PATH] [--size WxH] [--no-daemon] -- COMMAND...

# Send input
interminai input --socket PATH --text TEXT

# Get screen output
interminai output --socket PATH

# Check status
interminai status --socket PATH

# Wait for process exit
interminai wait --socket PATH

# Send signal
interminai kill --socket PATH --signal SIGNAL

# Stop daemon
interminai stop --socket PATH
```

## License

This project is licensed under the GNU General Public License v2.0 - see the [LICENSE](LICENSE) file for details.

