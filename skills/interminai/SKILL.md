---
name: interminai
description: Control interactive terminal applications like vim, git rebase -i, git add -i, git add -p, apt, rclone config, and TUI apps. Even another CLI LLM. Use when you need to interact with applications that require keyboard input, show prompts, menus, or have full-screen interfaces. Also use when commands fail or hang with errors like "Input is not a terminal" or "Output is not a terminal". Better than application specific hacks such as GIT_SEQUENCE_EDITOR or bypassing interactivity through file use.
allowed-tools: Shell
license: See LICENSE file
metadata:
  author: Michael S. Tsirkin <mst@kernel.org>
  version: 0.1.0
  category: terminal
---

# 🌀 an Interactive Terminal for AI (interminai)

Author: Michael S. Tsirkin <mst@kernel.org>

A terminal proxy for interactive CLI applications. See [examples.md](examples.md) and [reference.md](reference.md) for details.

## When to Use

**Use for interactive commands** that wait for input, show menus/prompts, or use full-screen interfaces (vim, git rebase -i, htop, apt).

**Use if you get errors like this "Warning: Output is not to a terminal" or "Warning: Input is not from a terminal".

**Don't use** for simple commands that just run and exit - use Shell instead.

## Quick Start

```bash
# 1. Start session
SOCKET=`mktemp -d /tmp/interminai-XXXXXX`/sock
./scripts/interminai start --socket "$SOCKET" -- COMMAND

# 2. Send input (--text supports escapes: \r \n \e \t \xHH etc.)
./scripts/interminai input --socket "$SOCKET" --text ':wq\r'

# 3. Check screen
./scripts/interminai output --socket "$SOCKET"

# 4. Clean up (always!)
./scripts/interminai stop --socket "$SOCKET"
rm "$SOCKET"; rmdir `dirname "$SOCKET"`
```

## Essential Commands

- `start --socket PATH -- COMMAND` - Start application
- `input --socket PATH --text 'text'` - Send input (escapes: `\n` `\e` `\t` `\xHH`)
- `output --socket PATH` - Get screen (add `--cursor print` for cursor position)
- `stop --socket PATH` - Stop session

## Key Best Practices

1. **Unique sockets**: Use `` SOCKET=`mktemp -d /tmp/interminai-XXXXXX`/sock ``
    Alernatively, use sockets in the current directory: ./interminai-project-sock
2. **Always clean up**: `stop`, then `rm` the socket directory
3. **Check output after each input** - don't blindly chain commands
4. **Add delays**: `sleep 0.2` after input for processing
5. **Set GIT_EDITOR=vim** for git rebase -i, git commit, etc.
6. **If screen garbled**: Send `\f` (Ctrl+L) to redraw

## Terminal Size

Default terminal size is 80x24. If not enough context fits on screen, use `--size` on start or `resize` to increase the window. Don't go overboard to avoid filling your context with excessive output.

```bash
# Start with larger terminal
./scripts/interminai start --socket "$SOCKET" --size 80x256 -- COMMAND

# Or resize during session
./scripts/interminai resize --socket "$SOCKET" --size 80x256
```

## Vim Navigation Tips

Exact counts for `h`/`j`/`k`/`l` are critical - cursor position after `dd` isn't always intuitive. Prefer:

- `:<number>` - Go to line directly (`:5\rdd`)
- `/<pattern>` - Search for text (`/goodbye\rdd`)
- `gg`/`G` - Anchor from known position
- `--cursor print` - Check position after operations
- `:%s/old/new/gc` - Search and replace with confirmation (`y`/`n` for each match)

## Complex Edits Shortcut

For complex multi-line edits, another option is to edit outside vim:

1. Use `output` to observe the file name
2. Use the Edit tool to modify the file directly
3. In vim, reload the file (`:e!\r`) or simply exit (`:q!\r`)

This avoids tricky vim navigation for large or intricate changes.

However, if editing is going to use sed anyway, using :%s/old/new/gc within vim
is more robust and more powerful as it shows you each change.

## Pressing Enter: `\n` vs `\r`

Traditional Unix apps accept either `\n` (LF) or `\r` (CR) for Enter because the
kernel TTY driver translates CR to LF when the ICRNL flag is set (default in
"cooked" mode). However, some modern apps (especially React/Ink-based TUIs like
cursor-agent) run in raw mode with ICRNL disabled, and only recognize `\r`.

**Best practice**: Always use `\r` for Enter/submit. It works universally:
- Traditional apps: kernel translates `\r` → `\n` (if ICRNL set)
- Raw-mode apps: receive `\r` directly as expected

Use `\n` only when you specifically want to add a newline to multiline input
(like continuing to type on a new line without submitting).

```bash
# Submit a command (use \r)
interminai input --socket "$SOCKET" --text 'hello world\r'

# Multiline input (use \n for newlines, \r to submit)
interminai input --socket "$SOCKET" --text 'line1\nline2\nline3\r'
```

## Menu prompts needing Enter

For git add -i/-p, the menu shown is not interactive, an action does not
take effect until you press enter. For example:

- Stage this hunk [y,n,q,a,d,K,j,J,g,/,e,?]?

Actually expects `y` followed by Enter and will not take effect until Enter is sent.

If you observe a tool is waiting for Enter once, it is safe to assume
it will be waiting for it next time, too.

## Modern TUI Apps (Ink/React-based)

Apps built with React/Ink (like cursor-agent) need a delay between typing text
and pressing Enter. Sending text and Enter in a single input call doesn't work
because React's internal state isn't ready for the Enter keystroke.

**Key finding**: Output polling alone is NOT sufficient. Even after text appears
on screen, React's state machine may not be ready. A time delay is required.

**Simple pattern (recommended)**:

```bash
# Send text, wait, then send Enter
interminai input --socket "$SOCKET" --text 'your prompt here'
sleep 0.1
interminai input --socket "$SOCKET" --text '\r'
```

**Why this works**: The 100ms delay gives React's event loop time to process the
input and update its internal state before receiving the Enter keystroke.

Use `debug` command to check if app is in raw mode (no ICRNL flag).

## Long lived shell sessions

It is possible to create a shell interminai session and pass commands to it.

```bash
SOCKET=`mktemp -d /tmp/interminai-XXXXXX`/sock
GIT_EDITOR=vim ./scripts/interminai start --socket "$SOCKET" -- bash
sleep 0.5
./scripts/interminai output --socket "$SOCKET"
# ... send commands now ..
./scripts/interminai input --socket "$SOCKET" --text 'vim foo.txt\r'
./scripts/interminai input --socket "$SOCKET" --text ':wq\r'
./scripts/interminai input --socket "$SOCKET" --text 'vim bar.txt\r'
./scripts/interminai input --socket "$SOCKET" --text ':wq\r'
./scripts/interminai input --socket "$SOCKET" --text 'exit\r'
./scripts/interminai wait --socket "$SOCKET"
rm "$SOCKET"; rmdir `dirname "$SOCKET"`
```

## Git Example

```bash
SOCKET=`mktemp -d /tmp/interminai-XXXXXX`/sock
GIT_EDITOR=vim ./scripts/interminai start --socket "$SOCKET" -- git rebase -i HEAD~3
sleep 0.5
./scripts/interminai output --socket "$SOCKET"
# ... edit with input commands ...
./scripts/interminai input --socket "$SOCKET" --text ':wq\r'
./scripts/interminai wait --socket "$SOCKET"
rm "$SOCKET"; rmdir `dirname "$SOCKET"`
```
