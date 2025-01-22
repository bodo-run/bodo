# Bodo Design Document

## 1. Core Architecture

The core of Bodo is a **Graph Manager** that builds/manages a graph representing:

- **Script files**
- **Tasks** (units of work with dependencies)
- **Commands** (shell/executable steps)

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

### 2.2 Optional Plugins

- **Timeout Plugin**: Adds task timeouts
- **Interactive Plugin**: Enables TUI prompts
- **Sandbox Plugin**: Restricts filesystem/network access

### 2.3 Plugin Lifecycle

1. **Initialization**: Load configs, initialize plugins
2. **Graph Build**: Resolve tasks, apply plugin transformations
3. **Execution Prepare**: Finalize env/PATH, setup concurrency
4. **Before Run**: Allocate resources (e.g., file watchers)
5. **Execution**: Run commands/tasks
6. **After Run**: Cleanup resources

### 2.4 Conflict Resolution

- Plugins modify node metadata via `metadata` field
- Last-plugin-wins by default (configurable via `priority`)

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

## 5. Task References

### 5.1 Syntax

- **Same File**: `task: build`
- **Cross-File**: `task: ../ci/test.yaml` (runs `default_task`)
- **Specific Task**: `task: ../ci/test.yaml/unit_tests`

### 5.2 Resolution Rules

- Paths are relative to the referencing file
- Environment variables expanded (e.g., `$PROJECT_ROOT/build.yaml`)
- Name collisions error unless fully qualified

## 6. CLI Commands

### 6.1 Basic Usage

```bash
bodo                          # Run default_task
bodo build                    # Run "build" task
bodo ./frontend/tasks.yaml    # Run default_task from frontend/tasks.yaml
bodo --watch test             # Re-run "test" on file changes
```

### 6.2 Flags

| Flag            | Description                        |
| --------------- | ---------------------------------- |
| `--dry-run`     | Simulate execution without running |
| `--list`        | List all tasks                     |
| `--sandbox`     | Restrict filesystem/network access |
| `--interactive` | Launch TUI task selector           |

## 7. Concurrency Model

### 7.1 Example

```yaml
deploy:
  concurrently_options:
    max_concurrent_tasks: 2
    fail_fast: true
  concurrently:
    - task: build
    - task: migrate
    - command: ./notify.sh
```

### 7.2 Failure Handling

- `fail_fast: true`: Send SIGTERM to all processes on failure
- `signal: "SIGKILL"`: Override termination signal

## 8. Watch Mode

- Debounces changes (default: 500ms)
- Triggers task re-runs with same arguments
- Ignores patterns from `bodo.toml`

## 9. Security

### 9.1 Sandbox Mode

```bash
bodo --sandbox run-untrusted
```

- Blocks network access
- Restricts filesystem to `cwd`

## 10. Language Server (LSP)

**Features**:

- Autocomplete task names/paths
- Validate task references
- Hover documentation

**Editors**: VS Code, Neovim, etc.

## 11. Testing Strategy

### 11.1 Test Types

- **Unit Tests**: Per-plugin functionality
- **Integration Tests**: Multi-plugin scenarios
- **E2E Tests**: Full CLI workflows
- **Cross-Platform**: Windows/Unix path handling

### 11.2 Plugin-Specific Tests

- **Lifecycle Tests**: Test each plugin phase
- **Metadata Tests**: Test conflict resolution
- **Error Tests**: Test error handling and recovery
- **Integration Tests**: Test plugin interactions

## 12. Example: Complex Workflow

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
```

**Run with**:

```bash
bodo --dry-run ci  # Validate execution plan
```

## 13. Roadmap

- **Documentation Generator**: `bodo docs`
- **Artifact Caching**: Skip tasks if outputs exist
- **Metrics Dashboard**: Track task performance

## 14. Philosophy

- **Unix-like**: Composability, clear failure signals
- **User-Centric**: Helpful errors, interactive prompts
- **Extensible**: Plugin API > hardcoded features
