# Bodo Task Runner

**ALWAYS follow these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.**

Bodo is a modern task runner and build tool written in Rust. It provides intuitive task organization with powerful features like concurrency, watch mode, environment variable management, task dependencies, custom plugins, timeouts, and sandbox execution.

## Working Effectively

### Bootstrap and Build Commands
- Install Rust toolchain: Visit https://rustup.rs/ and follow installation instructions
- Check and build the project:
  - `cargo check` -- compiles and checks for errors (~25 seconds)
  - `cargo build` -- debug build (~35 seconds). NEVER CANCEL. Set timeout to 90+ seconds.
  - `cargo build --release` -- release build (~50 seconds). NEVER CANCEL. Set timeout to 120+ seconds.
- Run linting and formatting:
  - `cargo clippy --all-targets --all-features -- -D warnings` -- linting (~15 seconds). NEVER CANCEL. Set timeout to 60+ seconds.
  - `cargo fmt --all --check` -- check formatting
  - `cargo fmt --all` -- apply formatting
- Run tests:
  - `cargo test --all --all-features` -- all tests (~40 seconds). NEVER CANCEL. Set timeout to 120+ seconds.
  - Note: One test `test_find_base_directory_with_no_wildcard` in `watch_plugin_additional_test` is known to fail - this is expected

### Using the Bodo CLI Tool
- Build first: `cargo build` to create `./target/debug/bodo`
- **CRITICAL**: For non-interactive usage, always use `BODO_NO_WATCH=1` environment variable
- Run default task: `BODO_NO_WATCH=1 ./target/debug/bodo`
- List all tasks: `./target/debug/bodo --list`
- Show help: `./target/debug/bodo --help`
- Dry-run mode: `BODO_NO_WATCH=1 ./target/debug/bodo --dry-run`
- Run specific tasks: `BODO_NO_WATCH=1 ./target/debug/bodo <task_name>`

### Key Project Structure
```
├── src/               # Rust source code
│   ├── lib.rs        # Main library
│   ├── main.rs       # CLI entry point
│   ├── plugins/      # Plugin system implementation
│   └── sandbox.rs    # Sandbox execution system
├── tests/            # Integration and unit tests
├── scripts/          # Bodo task definitions
│   ├── script.yaml   # Root tasks
│   ├── build/        # Build-related tasks
│   ├── test/         # Test-related tasks
│   └── deploy/       # Deployment tasks
├── .github/          # GitHub workflows and configuration
└── docs/             # Documentation
```

## Validation and Testing

### CRITICAL Build and Test Requirements
- **NEVER CANCEL BUILDS OR LONG-RUNNING COMMANDS** - Builds may take 50+ seconds, tests may take 40+ seconds
- **ALWAYS run the complete CI validation sequence**:
  1. `cargo fmt --all --check` (instant)
  2. `cargo clippy --all-targets --all-features -- -D warnings` (~15 seconds)
  3. `cargo test --all --all-features` (~40 seconds) - expect 1 known failing test
  4. `cargo build` (~35 seconds) to ensure CLI tool works

### Manual Testing Scenarios
After making changes, **ALWAYS test these complete user scenarios**:

1. **Basic CLI functionality**:
   ```bash
   cargo build
   ./target/debug/bodo --help
   ./target/debug/bodo --list
   BODO_NO_WATCH=1 ./target/debug/bodo --dry-run
   BODO_NO_WATCH=1 ./target/debug/bodo
   ```

2. **Task execution**:
   ```bash
   # Test various tasks defined in scripts/
   BODO_NO_WATCH=1 ./target/debug/bodo commit
   BODO_NO_WATCH=1 ./target/debug/bodo env-test --dry-run
   ```

3. **Build system integration**:
   ```bash
   # The bodo tool itself can run build tasks
   BODO_NO_WATCH=1 ./target/debug/bodo build
   BODO_NO_WATCH=1 ./target/debug/bodo test
   ```

## Development Workflow

### Code Style and Standards
- Follow Rust best practices as outlined in `AGENTS.md`
- Use `thiserror` for library errors, `anyhow` for binaries
- Avoid `unwrap`/`expect`/`panic!` in non-test code
- Use `tracing` instead of `println!` for logging
- All clippy warnings are treated as errors

### Making Changes
- **ALWAYS run validation early and frequently**: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all --all-features`
- Test CLI functionality after every change that affects the executable
- Use `RUST_LOG=debug ./target/debug/bodo <command>` for debugging
- Dry-run mode is available: `./target/debug/bodo --dry-run <task>`

### Plugin System
- Bodo uses a plugin architecture with these core plugins:
  - ExecutionPlugin: Runs commands and tasks
  - PathPlugin: Manages PATH environment variable
  - WatchPlugin: File watching and auto-reload
  - TimeoutPlugin: Command timeouts
  - PrefixPlugin: Output formatting
  - PrintListPlugin: Task listing functionality
- Plugin tests are in `tests/` directory with comprehensive coverage

### Task Configuration
Tasks are defined in YAML files (see `scripts/` directory):
- `default_task`: Runs when no task is specified
- `tasks`: Named tasks with commands, dependencies, environment variables
- Supports concurrency, watch mode, timeouts, environment variables
- Cross-file task references: `"../other.yaml/task-name"`

## Common Issues and Solutions

### Build Issues
- If clippy fails with `io_other_error`, use `std::io::Error::other("message")` instead of `std::io::Error::new(std::io::ErrorKind::Other, "message")`
- Missing cargo-nextest: Install with `cargo install cargo-nextest`
- Sandbox functionality requires `bwrap` or `firejail` but degrades gracefully

### Testing Issues
- Known failing test: `test_find_base_directory_with_no_wildcard` - ignore this failure
- Tests run with temporary directories and mock environments
- Use `RUST_BACKTRACE=1` for detailed error traces

### CI Integration
The CI pipeline (`.github/workflows/ci.yml`) runs:
- Format checking, clippy (deny warnings), and tests on Ubuntu
- Cross-platform builds for Linux, macOS, and Windows
- Code coverage analysis on pull requests
- Release automation for tagged versions

## Time Expectations and Timeouts

**CRITICAL: Always set adequate timeouts and NEVER CANCEL long-running operations**

| Operation | Time | Recommended Timeout |
|-----------|------|-------------------|
| `cargo check` | ~25s | 90s |
| `cargo build` | ~35s | 90s |
| `cargo build --release` | ~50s | 120s |
| `cargo clippy` | ~15s | 60s |
| `cargo test` | ~40s | 120s |
| `cargo fmt --check` | instant | 30s |

## Additional Resources

- Design Document: `DESIGN.md` - Architecture and feature specifications
- Usage Guide: `USAGE.md` - Comprehensive user documentation  
- Contributing: `AGENTS.md` - Development guidelines and best practices
- Roadmap: `ROADMAP.md` - Planned features and milestones

**Remember**: This is a task runner tool, so always validate that task execution works correctly after making changes. The tool should be able to run its own build and test tasks using the bodo CLI.