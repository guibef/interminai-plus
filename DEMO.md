# Creating Demo GIFs

This guide explains how to create the demo GIFs for interminai. Two demos are included:

1. **Git Rebase Demo** (`demo.gif`) - Claude performs an interactive git rebase with autosquash
2. **GDB Debugging Demo** (`demo-gdb.gif`) - Claude debugs a C program with GDB to discover function parameters

## Git Rebase Demo

This demo shows Claude using interminai to perform git interactive rebase.

## Prerequisites

### 1. Install Go (if not already installed)

```bash
# Fedora/RHEL
sudo dnf install golang

# Ubuntu/Debian
sudo apt install golang

# Or download from: https://go.dev/dl/
```

### 2. Install VHS

```bash
# Install VHS with go
go install github.com/charmbracelet/vhs@latest

# Add Go bin to PATH if not already (add to ~/.bashrc or ~/.zshrc)
export PATH="$HOME/go/bin:$PATH"

# Verify installation
vhs --version
```

### 3. Install ttyd (required by VHS)

```bash
# Fedora/RHEL
sudo dnf install ttyd

# Ubuntu/Debian
sudo apt install ttyd

# Arch
sudo pacman -S ttyd
```

### 4. Build interminai

```bash
make install-skill
```

## Creating the Demo

### Quick Method

Just run:

```bash
make demo
```

This will:
1. Create a clean git repository with test commits
2. Record Claude performing an interactive rebase
3. Generate `demo.gif`

### Manual Method

If you want to run the steps manually:

```bash
# 1. Setup clean demo repository
./demo-setup.sh

# 2. Generate the GIF with VHS
FORCE_COLOR=1 vhs demo-real.tape

# 3. View the result
firefox demo.gif
```

## What the Demo Shows

The demo captures a real Claude CLI session:

1. **Setup**: Shows git log with 4 commits (one is a fixup)
2. **User request**: "I have two commits where the second is a fixup. Can you squash them?"
3. **Claude's work**: 
   - Analyzes the git history
   - Uses `git rebase -i --autosquash --root`
   - Interacts with vim through interminai
   - Captures screen output
   - Completes the rebase
4. **Result**: Two commits successfully squashed into one

## Customizing the Demo

### Change Timing

Edit `demo-real.tape` and adjust the `Sleep` values:

```tape
Sleep 70s  # Time for Claude to complete the work
```

### Change Dimensions

```tape
Set Width 800    # Terminal width in pixels
Set Height 600   # Terminal height in pixels
```

### Change Theme

View available themes:

```bash
vhs themes
```

Update in `demo-real.tape`:

```tape
Set Theme "VibrantInk"
```

Preview themes at: https://iterm2colorschemes.com/

### Change Font Size

```tape
Set FontSize 13
```

## Troubleshooting

### VHS not found after installation

Make sure Go's bin directory is in your PATH:

```bash
echo 'export PATH="$HOME/go/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Browser launch error

If you get "fork/exec /path/to/chrome: exec format error", temporarily rename any chrome wrapper scripts:

```bash
mv ~/bin/chrome ~/bin/chrome.bak
make demo
mv ~/bin/chrome.bak ~/bin/chrome
```

### Claude output is monochrome

The `make demo` target sets `FORCE_COLOR=1` to enable colors. If colors still don't appear, check your Claude CLI version.

### Demo repository in bad state

If a previous recording failed, the git repository might have a rebase in progress:

```bash
rm -rf /tmp/interminai-demo
./demo-setup.sh
```

The `make demo` target does this automatically.

## File Sizes

The demo.gif will be approximately:
- **1-2 MB** - Normal recording (~75 seconds)
- Larger GIFs can be optimized with `gifsicle`:

```bash
sudo dnf install gifsicle
gifsicle -O3 --colors 256 demo.gif -o demo-optimized.gif
```

## GDB Debugging Demo

This demo shows Claude using interminai to debug a C program with GDB and discover function parameters.

### Files

- `demo-gdb-setup.sh` - Creates test environment with C program
- `demo-gdb.tape` - VHS recording script
- `demo-gdb.gif` - The generated GIF

### Creating the Demo

```bash
make demo-gdb
```

Or manually:

```bash
./demo-gdb-setup.sh
FORCE_COLOR=1 vhs demo-gdb.tape
```

### Setup

```bash
./demo-gdb-setup.sh
cd /tmp/interminai-gdb-demo
```

This creates:
- `mystery.c` - A C program with a `process_data()` function called indirectly through `helper()`
- `mystery` - Compiled binary with debug symbols
- `.claude/skills/interminai` - The interminai skill

### Demo Prompt

Start Claude and ask:

```
Use gdb to find out what parameters process_data() is called with when running ./mystery
```

### Expected Behavior

Claude should:
1. Start a gdb session with interminai
2. Set a breakpoint on `process_data`
3. Run the program
4. When the breakpoint hits, use `info args` to show the parameters
5. Report: `operation="divide"`, `count=7`, `factor=2.71828`

### Variations

Try with different inputs:
```
Use gdb to find what parameters process_data() receives when running ./mystery 15
```

This should show: `operation="multiply"`, `count=30`, `factor=3.14159`
