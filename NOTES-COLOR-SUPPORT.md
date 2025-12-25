# Notes on --color Flag Implementation Issues

## Summary

The `--color` flag was implemented in commits 14a779a through 5bc420e to preserve SGR (Select Graphic Rendition) escape sequences in output. However, this implementation has a fundamental architectural flaw that breaks vim's display of long lines.

## The Problem

### Implementation Approach (WRONG)

When `--color` is enabled, SGR escape sequences are "reconstructed" and each character is printed to the screen buffer using `self.print(c)`:

```rust
// In csi_dispatch for action 'm'
for c in sgr.chars() {
    self.print(c);  // ← THIS IS WRONG
}
```

### Why This Breaks

1. **Escape sequences take up space**: Each character in `\e[31m` (5 chars) advances the cursor position
2. **Long lines with colors accumulate fake characters**: A line with 20 color changes adds 100+ invisible "characters"
3. **Cursor position becomes incorrect**: Vim's cursor positioning commands reference the WRONG positions because our screen buffer has extra phantom characters from escape sequences
4. **Display becomes garbled**: Text appears in the wrong locations because cursor math is off

### Evidence

Test with and without `--color`:
- **Without --color**: `vim Makefile` correctly shows `.PHONY:` at line start
- **With --color**: `vim Makefile` shows `-impl install-claude` (middle of line) at line start

The escape sequences are being counted as printable characters, shifting everything.

## Root Cause

**Escape sequences should be invisible** - they control rendering but don't occupy screen positions. Our current architecture stores everything in a 2D array of `char`, which can't distinguish between:
- Printable characters (occupy a cell, move cursor)
- Control sequences (affect rendering, don't move cursor)

## Proper Solution (Not Implemented)

Would require architectural changes:

### Option 1: Store Attributes Separately
```rust
struct Cell {
    ch: char,
    fg_color: Option<Color>,
    bg_color: Option<Color>,
    bold: bool,
    underline: bool,
}
```
- Parse SGR and update cell attributes
- Don't store escape sequences as characters
- Reconstruct escape codes in `to_ascii()` based on attribute changes

### Option 2: Out-of-Band Escape Sequences
- Store escape sequences in a separate data structure
- Map them to (row, col) positions
- Insert them during output without storing in cells array

Both require significant refactoring of the Screen struct.

## Why DSR Was Investigated

We initially thought vim's display issues were due to missing DSR (Device Status Report) support. DSR allows applications to query cursor position. While implementing DSR was worthwhile and is now working, it didn't fix the vim issue because the root cause was the --color implementation.

## Current Status

- **DSR implementation**: ✅ Kept (commit 34b3644) - works correctly, useful feature
- **--color flag**: ❌ Has serious bugs, not worth fixing given the complexity
- **Commits 14a779a-5bc420e**: Left in history but --color flag should be avoided

## Recommendation

Do not use the `--color` flag. It will produce garbled output for any application that uses cursor positioning (like vim, less, etc.). The flag exists but is fundamentally broken.

If color support is needed in the future, it will require a proper implementation using one of the approaches above.

## Testing

To verify the issue:
```bash
# Broken - shows wrong position
target/release/interminai start --socket /tmp/test.sock --color -- vim Makefile

# Works - shows correct position  
target/release/interminai start --socket /tmp/test.sock -- vim Makefile
```
