# Bodo Usage

This document describes the current features of Bodo and how to use them.

## Installation & Setup

You'll need [Rust](https://www.rust-lang.org/tools/install) installed.

```bash
git clone https://github.com/bodo-run/bodo
cd bodo
cargo install --path . --all-features
```

## Command Overview

### Basic Command Structure

```bash
bodo [OPTIONS] [TASK] [SUBTASK] [ARGS...]
```

- **TASK**: The name of the task to run. If omitted, Bodo will attempt to run the default task (if available).
- **SUBTASK**: An optional subtask name.
- **ARGS...**: Additional arguments passed to the task's command.

### Common Flags

| Flag | Shorthand | Description |
|------|-----------|-------------|
| `--list` | `-l` | Lists all available tasks from all loaded scripts. |
| `--watch` | `-w` | Runs the specified task and re-runs it whenever watched files change. |
| `--auto-watch` | | Automatically enables watch mode if tasks define watch configurations. |
| `--debug` | | Enables debug logging (sets `RUST_LOG=bodo=debug`). |

### Examples

- Run the default task:
  ```bash
  bodo
  ```

- Run a specific task:
  ```bash
  bodo test
  ```

- Run a subtask:
  ```bash
  bodo deploy prod
  ```

- Pass additional arguments:
  ```bash
  bodo test watch -- --nocapture --test-threads=1
  ```

- Enable watch mode:
  ```bash
  bodo --watch test
  ```

- List tasks:
  ```bash
  bodo --list
  ```

## How Bodo Finds and Loads Tasks

Bodo searches for task definitions in:
- A root-level `script.yaml` (if provided).
- Any YAML files in a `scripts/` directory (searched recursively).
- Tasks defined in these files are parsed and stored in a dependency graph.

### Key Points
- The `default_task` in each script is executed if you invoke that script without specifying a task.
- Tasks are defined under a `tasks:` section.
- Cross-file task references (e.g., `"../other.yaml/some-task"`) are automatically resolved.

## Defining Tasks

Tasks are defined in YAML files. Here is an example of a task file:

```yaml
name: "exampleScript"
description: "This is an example script."

default_task:
  command: echo "Hello from default task!"
  description: "Runs by default if no task is specified."

tasks:
  example:
    description: "An example task."
    command: echo "Running example task..."
    env:
      EXAMPLE_VAR: "123"
    watch:
      patterns:
        - "src/**/*.rs"
      debounce_ms: 1000
```

### Available Task Fields
- `description` (string): Brief help text.
- `command` (string): Shell command to run (executed using `sh -c`).
- `cwd` (string): Optional working directory.
- `env` (map): Environment variables for the task.
- `watch` (object): Configuration for file watching.
- `timeout` (string): Timeout duration (e.g., "30s", "1m"); enforced by the TimeoutPlugin.
- `pre_deps` and `post_deps` (arrays): Define tasks or commands to run before/after the task.
- `concurrently` (array): Defines a group of tasks/commands to run in parallel (handled by the ConcurrentPlugin).

## Listing Tasks

Use the following command to list all tasks:
```bash
bodo --list
```

This command triggers the PrintListPlugin, which displays a grouped list of tasks from all discovered YAML files.

## Concurrency

Tasks can run parts of their workflow concurrently. For example:

```yaml
default_task:
  description: "Run two commands in parallel."
  concurrently_options:
    fail_fast: true       # Stop all tasks if one fails.
    max_concurrent_tasks: 2
  concurrently:
    - task: test
    - command: "echo 'Hello Parallel World'"
```

- `fail_fast`: If any concurrent task fails, remaining tasks are terminated.
- `max_concurrent_tasks`: Limits the number of tasks that run at the same time.

## Watch Mode

Tasks can be configured to automatically re-run when specified files change. Example configuration:

```yaml
tasks:
  test:
    command: cargo test
    watch:
      patterns:
        - "src/**/*.rs"
        - "tests/**/*.rs"
      debounce_ms: 1000
      ignore_patterns:
        - "target/**"
```

Then run:
```bash
bodo --watch test
```

Bodo will:
1. Execute the test task.
2. Monitor files matching the specified patterns.
3. Re-run the task when changes are detected.

## Debug Logging

Enable debug logs by using the `--debug` flag or setting the environment variable:

```bash
bodo --debug test
```

This will output additional information about plugin execution, dependency resolution, and process management.

## Environment Variables
- Global environment variables can be set for Bodo (e.g., `BODO_LOG_LEVEL`, `BODO_TASK_PATH`).
- Tasks can define their own `env` map, which is merged with any global environment settings.

Example:
```bash
export BODO_LOG_LEVEL=debug
bodo test
```

## Exit Codes
- Bodo exits with a non-zero code if any task or command fails.
- In concurrency mode with `fail_fast` enabled, if one task fails, Bodo attempts to terminate all other tasks and exits non-zero.

## Future / Unimplemented Features
- **Interactive TUI**: The `--interactive` mode for selecting tasks is not yet implemented.
- **ASCII Graph Visualization**: Although referenced in the design documentation, a `--graph` flag is not available.
- **Sandbox Mode**: No sandboxing features are currently implemented.
- **Failing Plugin**: A stub exists but has no functionality.

## Practical Examples

1. Run the default task:
   ```bash
   bodo
   ```

2. List all tasks:
   ```bash
   bodo --list
   ```

3. Run a build task in watch mode:
   ```bash
   bodo --watch build
   ```

4. Deploy using a subtask:
   ```bash
   bodo deploy prod
   ```

5. Run tests with custom arguments:
   ```bash
   bodo test -- --test-threads=1
   ```

6. Enable debug logging:
   ```bash
   bodo --debug test
   ```

This usage document covers the current capabilities of Bodo. For additional features or changes, refer to the design documentation and plugin interface for further customization.

