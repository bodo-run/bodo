# Bodo Task Runner Development Guide

Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.

## Working Effectively

### Bootstrap, Build, and Test the Repository
- **Check Rust version**: Ensure you have Rust 1.89.0 or newer installed
- **Build (development)**: `cargo build` -- takes ~32 seconds. NEVER CANCEL. Set timeout to 90+ seconds.
- **Build (release)**: `cargo build --release` -- takes ~47 seconds. NEVER CANCEL. Set timeout to 120+ seconds.  
- **Run tests**: `cargo test --all --all-features` -- takes ~39 seconds. NEVER CANCEL. Set timeout to 120+ seconds.
- **Format check**: `cargo fmt --all --check` -- takes ~1 second
- **Lint check**: `cargo clippy --all-targets --all-features -- -D warnings` -- takes ~13 seconds. NEVER CANCEL. Set timeout to 60+ seconds.

### Required Pre-commit Validation 
ALWAYS run these commands before committing changes or the CI (.github/workflows/ci.yml) will fail:
```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings  
cargo test --all --all-features
```

**Note**: Some test files currently have formatting issues that prevent `cargo fmt --check` from passing. Focus on formatting the main source code in `src/` which is the priority for clean CI builds.

### Application Testing and Development
- **Install the application locally**: `cargo install --path . --all-features`
- **Run the application**: `./target/release/bodo` or `./target/debug/bodo`
- **Test default task**: `BODO_ROOT_SCRIPT=scripts/script.yaml BODO_NO_WATCH=1 ./target/release/bodo`
- **List available tasks**: `./target/release/bodo --list`
- **Get help**: `./target/release/bodo --help`
- **Check version**: `./target/release/bodo --version`

## Validation

### Manual Testing Scenarios
ALWAYS run through at least one complete end-to-end scenario after making changes:

1. **Basic CLI functionality test**:
   ```bash
   # Build the application
   cargo build --release
   
   # Test help and version commands
   ./target/release/bodo --help
   ./target/release/bodo --version
   
   # Test task listing (should show available tasks from scripts/script.yaml)  
   ./target/release/bodo --list
   
   # Test default task execution
   BODO_ROOT_SCRIPT=scripts/script.yaml BODO_NO_WATCH=1 ./target/release/bodo
   ```

2. **Development workflow test**:
   ```bash
   # Clean build and test cycle
   cargo clean
   cargo build  # ~32s, NEVER CANCEL
   cargo test --all --all-features  # ~39s, NEVER CANCEL  
   cargo clippy --all-targets --all-features -- -D warnings
   cargo fmt --all --check
   ```

3. **Code change validation**:
   - After modifying source code, always run: `cargo build && cargo test`
   - After modifying tests, run: `cargo test --all --all-features`  
   - Before committing, run all pre-commit validation commands

### Key Testing Commands with Timeouts
- `cargo nextest run --profile ci` -- uses nextest for faster parallel test execution
- `cargo test --doc` -- run documentation tests
- Individual test files: `cargo test --test <test_name>`

## Common Tasks

### Project Structure Overview
```
.
├── src/                    # Main application source code
│   ├── main.rs            # CLI entry point  
│   ├── lib.rs             # Library root
│   ├── cli.rs             # Command-line interface
│   ├── config.rs          # Configuration handling
│   ├── graph.rs           # Task dependency graph
│   ├── manager.rs         # Task management
│   ├── process.rs         # Process execution
│   ├── script_loader.rs   # YAML script loading
│   ├── errors.rs          # Error types
│   └── plugins/           # Plugin system
├── tests/                 # Integration and unit tests  
├── scripts/               # Example task scripts
│   ├── script.yaml        # Root script with example tasks
│   ├── build/script.yaml  # Build-related tasks  
│   └── test/script.yaml   # Test-related tasks
├── .github/               # CI/CD workflows
└── docs/                  # Documentation
```

### Key Files and Their Purpose
- **src/main.rs**: CLI application entry point
- **src/graph.rs**: Task dependency graph implementation with NodeKind enum (uses Box<TaskData>)
- **src/plugins/**: Plugin system for extending functionality  
- **scripts/script.yaml**: Example task definitions for testing
- **.github/workflows/ci.yml**: CI pipeline with build, test, lint, and release jobs
- **Cargo.toml**: Project dependencies and metadata

### Development Patterns

#### Working with the Task Graph
- `NodeKind::Task` uses `Box<TaskData>` for large enum variant optimization
- Always use `Box::new(task_data)` when creating Task nodes
- Pattern matching works automatically: `if let NodeKind::Task(task_data) = &node.kind`

#### Error Handling  
- Use `thiserror` for library error types
- Use `anyhow` for flexible error contexts in binaries
- Use `std::io::Error::other()` instead of `Error::new(ErrorKind::Other, msg)`

#### Plugin Development
- Implement the `Plugin` trait in `src/plugin.rs`
- See examples in `src/plugins/` directory
- Use type aliases for complex function signatures (see `PrefixSettingsFn`)

### Testing Conventions
- Unit tests live alongside code; integration tests in `tests/` 
- Use realistic fixtures and temp dirs for file system tests
- For CLI testing, use `assert_cmd` and `predicates` crates
- Add regression tests for every bug fix with issue reference

### Build and Release Information
- **Rust version**: 1.89.0 minimum
- **Test framework**: Uses cargo-nextest for faster test execution
- **CI platforms**: Ubuntu, macOS, Windows (multiple architectures)
- **Release targets**: Linux (x86_64, aarch64, musl), macOS (x86_64, aarch64), Windows (x86_64, aarch64)

### Environment Variables
- `BODO_ROOT_SCRIPT`: Path to the root script file
- `BODO_NO_WATCH`: Disable watch mode for testing
- `RUST_LOG`: Control logging levels (info, debug, trace)
- `RUST_BACKTRACE`: Enable backtrace on errors

### Performance Notes
- Build times: ~32s (debug), ~47s (release) 
- Test execution: ~39s for full test suite
- Application startup: Near-instantaneous for simple tasks
- Task execution depends on the specific commands being run

### CI/CD Pipeline
The `.github/workflows/ci.yml` pipeline includes:
- **Lint job**: Format check (`cargo fmt --check`) and clippy (`cargo clippy -- -D warnings`)
- **Test job**: Full test suite with cargo-nextest
- **Build job**: Multi-platform release builds  
- **Coverage job**: Code coverage analysis (PR builds only)
- **Release job**: Automated releases on version tags

### Known Issues and Workarounds
- Some test files have formatting syntax issues that prevent `cargo fmt --check` from passing - focus on `src/` code formatting
- One test (`test_find_base_directory_with_no_wildcard`) currently fails - this is a pre-existing issue
- Task discovery only works with explicit `BODO_ROOT_SCRIPT` environment variable currently  
- Watch mode requires Ctrl-C to stop

### Quick Reference Commands
```bash
# Development cycle
cargo build                                              # ~32s
cargo test --all --all-features                         # ~39s
cargo clippy --all-targets --all-features -- -D warnings  # ~13s
cargo fmt --all --check                                 # ~1s

# Application testing  
BODO_ROOT_SCRIPT=scripts/script.yaml BODO_NO_WATCH=1 ./target/release/bodo
./target/release/bodo --list
./target/release/bodo --help

# Installation
cargo install --path . --all-features
```

Always build and exercise your changes by running the application and validating that the functionality works as expected. The test suite provides confidence but manual validation ensures real-world usability.