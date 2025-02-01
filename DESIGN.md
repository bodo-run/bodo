# Bodo Design Document

## 1. Core Architecture

The core of Bodo is a **Graph Manager** that builds/manages a graph representing:

- **Script files**
- **Tasks** (units of work with dependencies)
- **Commands** (shell/executable steps). Commands are the leaf nodes of the graph.

**Responsibilities**:

- Parse task files into a node graph
- Resolve task dependencies (`pre_deps`, `post_deps`, `concurrently`)
- Detect circular dependencies
- Enable concurrent execution
- Provide debugging tools (ASCII graph visualization via `bodo --graph`)

## 2. Plugin-Based Architecture

### 2.1 Core Plugins (Execution Order)

1. **Env Plugin**: Merges environment variables
2. **Path Plugin**: Computes `PATH` for each node
3. **Concurrent Plugin**: Wraps concurrent tasks
4. **Watch Plugin**: Adds file watchers
5. **Execution Plugin**: Runs processes
6. **Timeout Plugin**: Adds task timeouts
7. **Print List Plugin**: Handles task listing

### 2.3 Plugin Lifecycle and Ordering

> Currently all plugins are internal. Custom plugins are not supported and will be added in the future.

1. **Initialization**: Load configs, initialize plugins
2. **Graph Build**: Resolve tasks, apply plugin transformations
3. **Execution Prepare**: Finalize env/PATH, setup concurrency
4. **Before Run**: Allocate resources (e.g., file watchers)
5. **Execution**: Run commands/tasks
6. **After Run**: Cleanup resources

Plugins operate in a defined order to avoid conflicts:

1. Initial resolution plugins (e.g., resolver_plugin)
2. Environment and Path plugins
3. Concurrency and Timeout plugins
4. Other transformations (e.g., watchers, prefixing)
5. Execution plugins

Custom plugins fit into this order by declaring their priority and intended phases.

### 2.4 Conflict Resolution

- Plugins modify node metadata via `metadata` field
- Last-plugin-wins by default (configurable via `priority`)
- Conflicts resolved using plugin priority system
- Must handle structured data (e.g., JSON-compatible)

## 3. Task File Format

### 3.1 Example (`tasks/build.yaml`)

```yaml
tasks:
  build:
    description: "Compile project"
    command: "cargo build --release"
    env:
      RUSTFLAGS: "-C target-cpu=native"
    exec_paths:
      - ./bin
    pre_deps:
      - task: ../lint.yaml/all
```

### 3.2 Configuration (`bodo.toml`)

```toml
root_task_file = "tasks/main.yaml"
tasks_paths = ["packages/*/tasks.yaml"]

[watch]
ignore = ["*.tmp", "node_modules/"]
debounce_ms = 500

[env]
# All Bodo-specific environment variables are prefixed with BODO_
BODO_LOG_LEVEL = "info"
BODO_TASK_PATH = "./tasks"
```

## 4. Task Properties

| Property       | Description                                   |
| -------------- | --------------------------------------------- |
| `command`      | Shell command or script path                  |
| `pre_deps`     | Tasks/commands to run before this task        |
| `post_deps`    | Tasks/commands to run after this task         |
| `concurrently` | Tasks/commands to run in parallel             |
| `env`          | Environment variables (merged hierarchically) |
| `exec_paths`   | Directories added to `PATH`                   |
| `timeout`      | Maximum runtime (e.g., `10s`)                 |
| `prefix_color` | Output prefix color (e.g., `"green"`)         |
| `cwd`          | Working directory for the task                |
| `args`         | CLI argument definitions                      |
| `silent`       | Don't echo command before running             |

## 5. Task References

### 5.1 Syntax

- **Same File**: `task: build`
- **Cross-File**: `task: ../ci/test.yaml` (runs `default_task`)
- **Specific Task**: `task: ../ci/test.yaml/unit_tests`

### 5.2 Resolution Rules

- Paths are relative to the referencing file
- Environment variables expanded (e.g., `$BODO_PROJECT_ROOT/build.yaml`)
- Name collisions error unless fully qualified

### 5.3 Restrictions

- Task names can't contain `/`, `.`, or `..`
- Max length: 100 characters
- Min length: 1 character
- Reserved words (cannot be used as task names):
  - `watch`
  - `default_task`
  - `pre_deps`
  - `post_deps`
  - `concurrently`
- Name collisions between files are errors unless resolved in config

## 6. CLI Commands

### 6.1 Basic Usage

```bash
bodo                          # Run default_task
bodo build                    # Run "build" task
bodo ./frontend/tasks.yaml    # Run default_task from frontend/tasks.yaml
bodo --watch test             # Re-run "test" on file changes
bodo <task_name> -- <args>    # Pass args to task
```

### 6.2 Flags

| Flag            | Description                        |
| --------------- | ---------------------------------- |
| `--dry-run`     | Simulate execution without running |
| `--list`        | List all tasks                     |
| `--sandbox`     | Restrict filesystem/network access |
| `--interactive` | Launch TUI task selector           |
| `--graph`       | Show ASCII dependency graph        |
| `--debug`       | Show verbose internal logs         |

## 7. Concurrency Model

### 7.1 Example

```yaml
deploy:
  concurrently_options:
    max_concurrent_tasks: 2
    fail_fast: true
    prefix_output: true
  concurrently:
    - task: build
    - task: migrate
    - command: ./notify.sh
```

### 7.2 Failure Handling

- `fail_fast: true`: Send SIGTERM to all processes on failure
- `signal: "SIGKILL"`: Override termination signal
- Without `fail_fast`, tasks continue and group is considered partially successful/failed
- Behavior is plugin-configurable
- Process signals controlled via `BODO_KILL_SIGNAL` environment variable

## 8. Watch Mode

- Debounces changes (default: 500ms)
- Triggers task re-runs with same arguments
- Ignores patterns from `bodo.toml`
- Prevents infinite trigger loops with concurrency

## 9. Testing Strategy

### 9.1 Test Types

- **Unit Tests**: Per-plugin functionality
- **Integration Tests**: Multi-plugin scenarios
- **E2E Tests**: Full CLI workflows
- **Cross-Platform**: Windows/Unix path handling

### 9.2 Plugin-Specific Tests

- **Lifecycle Tests**: Test each plugin phase
- **Metadata Tests**: Test conflict resolution
- **Error Tests**: Test error handling and recovery
- **Integration Tests**: Test plugin interactions

### 9.3 Error Handling

- Consistent error types across plugins
- Error bubbling to surface root causes
- User-friendly suggestions:
  - "Did you mean <closest_task>?" for typos
  - Clear messages for name collisions
  - Hints for fixing configuration issues

## 10. Example: Complex Workflow

```yaml
# tasks/ci.yaml
tasks:
  ci:
    concurrently:
      - task: build
      - task:
          concurrently:
            - task: lint
            - task: test
    timeout: 10m
    env:
      CI: "true"
      BODO_LOG_LEVEL: "debug"
      BODO_PREFIX_COLOR: "cyan"
```

**Run with**:

```bash
bodo --dry-run ci  # Validate execution plan
```

## 11. Future Enhancements

- **Documentation Generator**:
  - `bodo docs` opens documentation in browser
  - Generate Markdown/HTML docs and output to a directory to be served
- **Robust Editor Integration**: Real-time feedback
- **Custom Plugins**: Allow custom plugins to be added to the graph
- **Language Server Protocol (LSP)**:
  - Autocomplete task names/paths
  - Validate task references
  - Hover documentation
  - Warn about name collisions
  - Suggest fixes for typos
  - Auto-complete environment variables
  - Support for VS Code, Neovim, etc.
- **Sandbox Mode**:
  - Restrict filesystem access to `cwd`
  - Block network access
  - Run untrusted tasks safely via `bodo --sandbox run-untrusted`
- **Automatic Migration Scripts**: Generate scripts for migrating from `Makefile`/`package.json`/other script runners

## 12. Philosophy

- **Unix-like**: Composability, clear failure signals
- **User-Centric**: Helpful errors, interactive prompts
- **Extensible**: Plugin API > hardcoded features

## 13. Environment Variables

All Bodo-specific environment variables are prefixed with `BODO_`. Common variables include:

| Variable              | Description                               | Default     |
| --------------------- | ----------------------------------------- | ----------- |
| `BODO_LOG_LEVEL`      | Logging verbosity (error/warn/info/debug) | `"info"`    |
| `BODO_TASK_PATH`      | Default path for task files               | `"./tasks"` |
| `BODO_PREFIX_COLOR`   | Default color for task output prefix      | `"white"`   |
| `BODO_KILL_SIGNAL`    | Signal used to terminate tasks            | `"SIGTERM"` |
| `BODO_WATCH_DEBOUNCE` | Watch mode debounce in ms                 | `500`       |
| `BODO_MAX_CONCURRENT` | Default max concurrent tasks              | `4`         |
| `BODO_PROJECT_ROOT`   | Root directory for relative paths         | `cwd`       |

These variables can be set in:

- Environment
- `bodo.toml` configuration
- Task-specific `env` section

## 14. Graph

The graph is a directed acyclic graph (DAG) that represents the task dependencies and commands.

To illustrate the graph, consider the following example:

```
scripts/
├── script.yaml      <== ROOT SCRIPT
├── build
│   └── script.yaml
├── check
│   └── script.yaml
├── fail-fast
│   └── script.yaml
├── ks
│   └── script.yaml
└── script.yaml
```

And script contents

```
==> scripts/script.yaml <==
description: Root level tasks

default_task:
  command: "echo 'Hello from bodo!'"
  description: "Default greeting"

tasks:
  echo:
    command: "echo 'Hello from `bodo echo`!'"
    description: "echo task"
  echo2:
    command: "echo 'Hello from `bodo echo2`!'"
    description: "echo2 task"

==> scripts/build/script.yaml <==
name: Build Script
description: Build tasks for the project

# Add paths to the script's execution context
exec_paths:
  - target/debug
  - target/release

# Set environment variables for this script
env:
  RUST_BACKTRACE: "1"

# The default task is invoked by simply running `bodo build`
default_task:
  command: cargo build

# tasks
tasks:
  release:
    description: Build the project in release mode
    command: cargo build --release

  check:
    description: Check the project
    command: cargo check

  clippy:
    description: Run clippy
    command: cargo clippy
    silent: true

  fmt:
    description: Format the project
    command: cargo fmt
==> scripts/check/script.yaml <==
name: check
description: Run all checks for the project, including clippy

default_task:
  command: echo "All checks completed"
  pre_deps:
    - command: echo "Running checks..."
    - task: clean
    - task: check
    - task: clippy
    - task: test
    - task: fmt-check
  concurrently:
    - command: cargo clippy
    - task: test
    - task: fmt-check
  env:
    RUST_BACKTRACE: "1"

tasks:
  check:
    command: cargo check

  clean:
    command: cargo clean

  clippy:
    command: cargo clippy

  test:
    command: cargo test

  fmt-check:
    description: Check code formatting
    command: cargo fmt --check
==> scripts/fail-fast/script.yaml <==
name: "Fail Fast Demo"
description: "Demonstrate fail-fast behavior with concurrent tasks"

default_task:
  description: "Run concurrent tasks with fail-fast behavior"
  concurrently_options:
    fail_fast: true
  concurrently:
    - command: 'echo "First output"'
      name: first
    - command: 'echo "Second output"' # no name given (should be named "command1")
    - command: 'sleep 1 && echo "Failing..." && exit 1'
      name: failing
    - command: 'echo "Third output"'
      name: third
    - command: 'sleep 2 && echo "Should not be run"'
      name: should_not_be_run
==> scripts/ks/script.yaml <==
name: Kitchen Sink

default_task:
  description: "Run all tasks in parallel"

  concurrently:
    - command: 'echo "A command"'
    - task: hello
    - task: world
    - task: slow
    - task: fast

tasks:
  hello:
    command: 'echo "Hello task"'

  world:
    command: 'echo "World task"'

  slow:
    command: |
      sleep $((RANDOM % 5)) && echo "Slow task output 1" && \
      sleep $((RANDOM % 5)) && echo "Slow task output 2"

  fast:
    command: |
      sleep $((RANDOM % 2)) && echo "Fast task output 1" && \
      sleep $((RANDOM % 2)) && echo "Fast task output 2"

  fail-fast:
    description: "Fail fast concurrently"
    concurrently_options:
      fail_fast: true
    concurrently:
      - command: 'echo "First output"'
      - command: 'sleep 1 && echo "Failing..." && exit 1'
      - command: 'echo "Third output"'
      - command: 'sleep 2 && echo "Should not be run"'
```

Running `bodo --list` will show the following:

```
  (default_task)   Default greeting
  echo             echo task
  echo2            echo2 task

Build Script (scripts/build/script.yaml)
Build tasks for the project

  build
  build release   Build the project in release mode
  build check     Check the project
  build clippy    Run clippy
  build fmt       Format the project

check (scripts/check/script.yaml)
Run all checks for the project, including clippy

  check
  check check     Run all checks for the project, including clippy
  check clean     Clean the project
  check clippy    Run clippy
  check test      Run tests
  check fmt-check Check code formatting

fail-fast (scripts/fail-fast/script.yaml)
Fail Fast Demo

  fail-fast       Demonstrate fail-fast behavior with concurrent tasks

Kitchen Sink (scripts/ks/script.yaml)
Kitchen Sink

  ks               Run all tasks in parallel
  ks hello         Hello task
  ks world         World task
  ks slow          Slow task
  ks fast          Fast task
  ks fail-fast     Fail fast concurrently
```

The graph is visualized as follows:

```
📦 ROOT (default_task)
└── 📦 scripts/script.yaml/default_task
    └── 🚀 "echo 'Hello from bodo!'"

📦 scripts/build.yaml
├── 📦 build
│   └── 🚀 "cargo build"
├── 📦 release
│   └── 🚀 "cargo build --release"
├── 📦 check
│   └── 🚀 "cargo check"
├── 📦 clippy
│   └── 🚀 "cargo clippy" (silent)
└── 📦 fmt
    └── 🚀 "cargo fmt"

📦 scripts/check.yaml
└── 📦 default_task
    ├── 📦 pre_deps_chain
    │   ├── 🚀 "echo 'Running checks...'"
    │   ├── 📦 clean
    │   │   └── 🚀 "cargo clean"
    │   ├── 📦 check
    │   │   └── 🚀 "cargo check"
    │   ├── 🌐 ../build.yaml/clippy
    │   │   └── 📦 clippy
    │   │       └── 🚀 "cargo clippy"
    │   ├── 📦 test
    │   │   └── 🚀 "cargo test"
    │   └── 📦 fmt-check
    │       └── 🚀 "cargo fmt --check"
    └── 🔀 concurrent_group
        ├── 🚀 "cargo clippy"
        ├── 📦 test
        │   └── 🚀 "cargo test"
        └── 📦 fmt-check
            └── 🚀 "cargo fmt --check"

📦 scripts/fail-fast.yaml
└── 📦 default_task
    └── 🔀 concurrent_group (fail_fast: true)
        ├── 🚀 "echo 'First output'" (first)
        ├── 🚀 "echo 'Second output'"
        ├── 🚀 "sleep 1 && echo 'Failing...' && exit 1" (failing)
        ├── 🚀 "echo 'Third output'" (third)
        └── 🚀 "sleep 2 && echo 'Should not be run'"

📦 scripts/ks.yaml
└── 📦 default_task
    └── 🔀 concurrent_group
        ├── 🚀 "echo 'A command'"
        ├── 📦 hello
        │   └── 🚀 "echo 'Hello task'"
        ├── 📦 world
        │   └── 🚀 "echo 'World task'"
        ├── 📦 slow
        │   └── 🚀 "sleep RANDOM && echo..."
        ├── 📦 fast
        │   └── 🚀 "sleep RANDOM && echo..."
        └── 🌐 ../fail-fast.yaml/default_task
            └── 🔀 concurrent_group (fail_fast)
                ├── 🚀 "echo 'First output'"
                ├── 🚀 "sleep 1 && fail..."
                ├── 🚀 "echo 'Third output'"
                └── 🚀 "sleep 2 && echo..."
```

KEY:

- 📦 = Task node
- 🔀 = Concurrent group
- 🚀 = Command node
- 🌐 = Cross-file reference

NOTES:

1. All paths terminate at 🚀 command nodes
2. Silent commands marked with "(silent)"
3. Named concurrent tasks show (name)
4. Fail-fast groups marked with (fail_fast)
5. Random delays shown as RANDOM
6. Cross-file references use 🌐 emoji

## 15. TODO

### High Priority

1. **Interactive Plugin (TUI)**
   - Implement TUI for task selection (`--interactive` flag)
   - Show available tasks with descriptions
   - Allow fuzzy search/filtering
   - Return chosen task name for execution

2. **Graph Visualization**
   - Implement `--graph` flag
   - Create ASCII diagram of task dependencies
   - Show in topological order
   - Include task types, dependencies, and concurrent groups

3. **Dry Run Mode**
   - Add `--dry-run` flag support
   - Modify ExecutionPlugin to skip actual command execution
   - Log/print commands that would be run
   - Show environment and path modifications

4. **Task Name Validation**
   - Add checks for reserved words in script_loader.rs
   - Validate against: watch, default_task, pre_deps, post_deps, concurrently
   - Provide clear error messages for invalid names

### Medium Priority

5. **User-Friendly Errors**
   - Add "Did you mean?" suggestions for unknown tasks
   - Implement Levenshtein distance checks
   - Enhance TaskNotFound error handling
   - Add context to common error scenarios

6. **Windows Support**
   - Replace hardcoded `sh` with platform-specific shells
   - Support cmd.exe on Windows
   - Implement cross-shell command execution
   - Test on various Windows environments

7. **Custom Plugin Support**
   - Design stable plugin trait interface
   - Enable third-party plugin loading
   - Document plugin development process
   - Provide example custom plugins

### Lower Priority

8. **Documentation Generator**
   - Implement `bodo docs` command
   - Generate Markdown/HTML documentation
   - Include task descriptions and dependencies
   - Add examples and usage patterns

9. **Test Coverage**
   - Unit tests for each plugin
   - Integration tests for task execution
   - E2E tests with CLI snapshots
   - Cross-platform testing

10. **Sandbox Mode**
    - Research sandboxing approaches
    - Implement filesystem/network restrictions
    - Add `--sandbox` flag
    - Document security implications

### Misc Improvements

11. **Environment Variable Support**
    - Implement BODO_KILL_SIGNAL for process termination
    - Add more environment variable configurations
    - Document all supported variables

12. **Output Formatting**
    - Extend PrefixPlugin for non-concurrent tasks
    - Add more color customization options
    - Improve error message formatting

13. **Performance Optimizations**
    - Profile task execution
    - Optimize graph traversal
    - Improve concurrent task handling
    - Reduce memory usage for large task graphs
