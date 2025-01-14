# Bodo Design Document

## 1. Core Architecture

The core of Bodo is a Graph Manager that builds and manages a graph representing:

- Script files
- Tasks within scripts
- Commands (simpler than tasks, can be bash scripts or executable files)

The Graph Manager handles:

- Task dependencies (pre/post)
- Task references
- Concurrent tasks
- Circular dependency detection
- Debugging tools (graph visualization etc)

## 2. Plugin-Based Architecture

Everything beyond core graph management is implemented as plugins:

### 2.1 Environment Variables Plugin

- Manages environment variables
- Tracks final env var values on graph nodes

### 2.2 Command Prefix Plugin

- Handles command output prefixing (e.g. `[build] building...`)
- Configures prefixes on task/command nodes

### 2.3 Execution Plugin

- Uses Tokio for process management
- Handles script and task execution

### 2.4 Watch Plugin

- Uses Tokio for file watching
- Manages watched files and task triggers

### 2.5 Path Plugin

- Computes final PATH for each node
- Adds exec_paths to node environment

### 2.6 List Plugin

- Prints tasks and commands from graph
- Handles task documentation

## 3. Requirements

### 3.1 Core Graph Management

- Parse script files into node structure
- Handle dependencies and prevent cycles
- Provide debugging interface

### 3.2 Plugin Lifecycle

- Initialization phase
- Graph transformation phase
- Execution preparation phase
- Execution phase

### 3.3 Data and Metadata

- Allow plugins to modify node metadata
- Define conflict resolution
- Support structured data on nodes

### 3.4 Concurrency Support

- Handle parallel task execution
- Support fail-fast behavior
- Track task status

### 3.5 Watch Mode

- Monitor file changes
- Re-run affected tasks
- Integrate with concurrency

### 3.6 Environment Management

- Gather env vars from config
- Handle inheritance
- Merge PATH correctly

### 3.7 Command Execution

- Async process management
- Output logging with prefixes
- Environment integration

### 3.8 Documentation

- Generate task listings
- Support multiple formats
- Include descriptions

### 3.9 Error Handling

- Consistent error types
- Error bubbling
- User-friendly logging

### 3.10 Testing

- Unit tests per plugin
- Integration tests
- Graph validation tests
- Execution tests

## 4. Task File Format and Structure

Tasks are defined in a YAML file. Here are some examples:

### 4.1 Simple Task File

```yaml
default_task:
  test:
    command: "cargo test"
```

### 4.2 Multiple Tasks

```yaml
tasks:
  test:
    command: "cargo test"
  lint:
    command: "cargo clippy"
```

### 4.3 Combined Format

```yaml
default_task:
  test:
    command: "cargo test"
tasks:
  lint:
    command: "cargo clippy"
```

### 4.4 Configuration

Default configuration in `bodo.yaml`:

```toml
root_task_file_path = "scripts/scripts.yaml"
tasks_paths = ["scripts"]
```

Custom configuration:

```toml
root_task_file_path = "my_tasks.yaml"
tasks_paths = ["packages/*/tasks.yaml"]
```

## 5. Task Properties

- `command`: The command to run
- `pre_deps`: The tasks that must be run before this task
- `post_deps`: The tasks that will be run after this task
- `concurrently`: The tasks that will be run concurrently with this task
- `description`: The description of the task
- `env`: The environment variables to set for the task
- `exec_paths`: The paths to add to the PATH environment variable
- `args`: The arguments options
- `cwd`: The current working directory for the task
- `prefix_color`: The color of the prefix. Colors are from `colored` crate

## 6. Task Configuration

### 6.1 Task References

Basic task reference:

```yaml
default_task:
  pre_deps:
    - task: test
tasks:
  test:
    command: "cargo test"
```

Cross-file reference:

```yaml
default_task:
  pre_deps:
    - task: ../other_tasks.yaml # default_task will be used
    - task: ./other_tasks.yaml/some_task # some_task from other_tasks.yaml will be used
```

### 6.2 Task Name Restrictions

- max length: 100
- min length: 1
- Disallow special characters:
  - `/` (because it is used for relative paths)
  - `.` (because it is used for current directory)
  - `..` (because it is used for parent directory)
- Task name must not be a reserved word:
  - watch
  - default_task
  - pre_deps
  - post_deps
  - concurrently

### 6.3 Task Resolution

Example task file (`scripts/validate.yaml`):

```yaml
default_task:
  command: "echo 'Hello, World!'"
tasks:
  test:
    command: "echo 'Hello, Test!'"
  lint:
    command: "echo 'Hello, Lint!'"
```

Usage:

```bash
bodo validate # default_task will be used
bodo validate test # test task will be used
```

### 6.4 Custom Directory Structure

Configuration in `bodo.yaml`:

```toml
root_task_file_path = "./tasks.yaml"
scripts_paths = ["./packages/*/tasks.yaml"]
```

## 7. Command Configuration

### 7.1 Basic Command Forms

Simple command:

```yaml
command: "cargo test"
```

Shell command:

```yaml
command:
  sh: "echo 'Hello, World!'"
```

### 7.2 Script Files

Shell script:

```yaml
command: ./path/to/script.sh
```

Other script types:

```yaml
command: ./path/to/script.ts
```

Language-specific:

```yaml
command:
  python: ./path/to/script.py
```

```yaml
command:
  js: ./path/to/script.js
```

### 7.3 Command Options

- `name`: The name of the task
- `args`: The arguments options
- `silent`: Whether to run the command silently. Will not print the command content to the console first
- `cwd`: The current working directory for the task
- `env`: The environment variables to set for the task
- `exec_paths`: The paths to add to the PATH environment variable
- `description`: The description of the task

## 8. Task Dependencies

### 8.1 Pre-dependencies

```yaml
tasks:
  test:
    command: "cargo test"
  lint:
    command: "cargo clippy"
default_task:
  pre_deps:
    - task: test
    - task: lint
```

Command dependencies:

```yaml
default_task:
  pre_deps:
    - command: "cargo test"
    - command: "cargo clippy"
  command: "cargo build"
```

### 8.2 Post-dependencies

Works exactly like pre_deps.

### 8.3 Concurrent Tasks

```yaml
default_task:
  concurrently:
    - task: test
    - task: lint
```

When `concurrently` is used, no `command` is allowed. but a `command` can be used with `concurrently`:

```yaml
default_task:
  concurrently:
    - task: test
    - task: lint
    - command: echo "Hello, World!"
```

#### Concurrent Task Options

Set it under `concurrently_options` key.

```yaml
default_task:
  concurrently_options:
    max_concurrent_tasks: 2
    prefix_output: false
    fail_fast_on_error: true
  concurrently:
    - task: test
    - task: lint
    - task: build
```

- `max_concurrent_tasks`: The maximum number of tasks to run concurrently
- `prefix_output`: Whether to prefix the output of the tasks
- `fail_fast_on_error`: Whether to fail fast if one of the tasks fails
- `fail_fast_on_error_exit_code`: The exit code to fail fast on
- `fail_fast_on_error_exit_code_range`: The range of exit codes to fail fast on

## 9. Additional Properties

### 9.1 Description

Single line:

```yaml
description: "Build the project"
```

Multi-line:

```yaml
description: |
  Build the project
  This is a multiline description
```

### 9.2 Environment Variables

```yaml
env:
  RUST_LOG: "info"
```

### 9.3 Execution Paths

```yaml
exec_paths:
  - /usr/local/bin
  - /usr/bin
  - ./node_modules/.bin
```

### 9.4 Arguments

Basic string argument:

```yaml
args:
  - name: "name"
    description: "The name of the task"
    type: "string"
    default: "world"
```

Enum argument:

```yaml
args:
  - name: "name"
    description: "The name of the task"
    type: "enum"
    values: ["hello", "world"]
    default: "world"
```

Number argument:

```yaml
args:
  - name: "int"
    description: "The number of stuff"
    type: "number"
    default: 1
```

Prompt argument:

```yaml
args:
  - name: "name"
    description: "The name of user"
    type: "prompt"
    prompt: "What is your name?"
```

### 9.5 Working Directory

```yaml
cwd: "./scripts"
```

### 9.6 Prefix Color

```yaml
prefix_color: "red"
```

## 10. Plugins

Plugins are the core of Bodo. They are responsible for the core functionality of Bodo.

Plugins are implemented as traits.
