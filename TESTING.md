# Testing Alternative Implementations

The test suite can be used to verify alternative implementations of `interminai` (e.g., Python, Go, etc.) by setting the `OVERRIDE_CARGO_BIN_EXE_interminai` environment variable.

## Usage

### Testing the Rust Implementation (default)

```bash
cargo test
```

### Testing an Alternative Implementation

```bash
OVERRIDE_CARGO_BIN_EXE_interminai=/path/to/interminai.py cargo test
```

Or for a system-installed version:

```bash
OVERRIDE_CARGO_BIN_EXE_interminai=$(which interminai-py) cargo test
```

### Isolating Server vs Client Issues

You can test server and client independently to isolate bugs:

```bash
# Test Python server with Rust client (isolate server implementation)
OVERRIDE_CARGO_BIN_EXE_interminai_SERVER=/path/to/interminai.py cargo test

# Test Rust server with Python client (isolate client implementation)
OVERRIDE_CARGO_BIN_EXE_interminai_CLIENT=/path/to/interminai.py cargo test

# Test both (equivalent to OVERRIDE_CARGO_BIN_EXE_interminai)
OVERRIDE_CARGO_BIN_EXE_interminai_SERVER=/path/to/interminai.py \
OVERRIDE_CARGO_BIN_EXE_interminai_CLIENT=/path/to/interminai.py \
cargo test
```

This is invaluable for debugging - if Python server works with Rust client, and Python client works with Rust server, but Python-to-Python fails, you know the issue is in the protocol compatibility, not the core logic.

## Requirements for Alternative Implementations

Any alternative implementation must:

1. **Implement the same CLI interface** - all commands and flags must work identically
2. **Use the same protocol** - socket-based JSON communication (see `PROTOCOL.md`)
3. **Support all commands**: `start`, `stop`, `input`, `output`, `running`, `wait`, `kill`, `resize`
4. **Handle daemonization** properly (double-fork pattern)
5. **Implement PTY management** correctly
6. **Parse terminal escape sequences** for screen emulation

## Example: Testing Python Implementation

```bash
# Build and install your Python implementation
cd python-impl/
pip install -e .

# Run the full Rust test suite against it
cd ../rust/
OVERRIDE_CARGO_BIN_EXE_interminai=$(which interminai) cargo test

# Run specific tests
OVERRIDE_CARGO_BIN_EXE_interminai=$(which interminai) cargo test --test daemon_test
OVERRIDE_CARGO_BIN_EXE_interminai=$(which interminai) cargo test test_vim
```

## CI Integration

You can test multiple implementations in parallel:

```yaml
strategy:
  matrix:
    impl: [rust, python, go]
steps:
  - name: Test Rust
    if: matrix.impl == 'rust'
    run: cargo test

  - name: Test Python
    if: matrix.impl == 'python'
    run: |
      pip install -e python-impl/
      OVERRIDE_CARGO_BIN_EXE_interminai=$(which interminai) cargo test
```

## Implementation

All test files import `interminai_bin()` from `tests/common.rs`, which checks the environment variable:

```rust
pub fn interminai_bin() -> String {
    std::env::var("OVERRIDE_CARGO_BIN_EXE_interminai")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_interminai").to_string())
}
```

This allows the test suite to be completely implementation-agnostic.
