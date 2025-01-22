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

1. **Resolver Plugin**: Resolves task references (`task: ../other.yaml/build`)
2. **Env Plugin**: Merges environment variables
3. **Path Plugin**: Computes `PATH` for each node
4. **Concurrent Plugin**: Wraps concurrent tasks
5. **Watch Plugin**: Adds file watchers
6. **Execution Plugin**: Runs processes
7. **Timeout Plugin**: Adds task timeouts
8. **Interactive Plugin**: Enables TUI prompts

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

- **Documentation Generator**: `bodo docs` opens documentation in browser
- **Artifact Caching**: Skip tasks if outputs exist
- **Metrics Dashboard**: Track task performance
- **Documentation Site**: Generate Markdown/HTML docs and output to a directory to be served
- **Robust Editor Integration**: Real-time feedback
- **Enhanced Kill Behavior**: Fine-tuned process control
- **Task Aliasing**: Allow controlled name collisions
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
