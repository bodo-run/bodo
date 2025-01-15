# Bodo Design Document (Expanded)

## 1. Core Architecture

The core of Bodo is a **Graph Manager** that builds and manages a graph representing:

- **Script files**
- **Tasks** within scripts
- **Commands** (simpler than tasks, can be bash scripts or executable files)

The **Graph Manager** handles:

- **Task dependencies** (pre/post)
- **Task references**
- **Concurrent tasks**
- **Circular dependency detection**
- **Debugging tools** (graph visualization etc)

## 2. Plugin-Based Architecture

Everything beyond core graph management is implemented as plugins. Each plugin can modify or enhance the graph (e.g., adding environment variables, PATH, concurrency wrappers, watchers, etc.) according to its responsibilities.

### 2.1 Environment Variables Plugin

- Manages environment variables.
- Tracks final env var values on graph nodes, enabling each task to have consolidated environment properties.

### 2.2 Command Prefix Plugin

- Handles command output prefixing (e.g. `[build] building...`).
- Configures prefixes on task/command nodes, integrating color or other formatting settings.

### 2.3 Execution Plugin

- Uses **Tokio** for process management.
- Handles script and task execution (launches the actual processes according to the graph).

### 2.4 Watch Plugin

- Uses **Tokio** for file watching.
- Manages watched files and triggers tasks when these files change.

### 2.5 Path Plugin

- Computes the final `PATH` for each node.
- Adds `exec_paths` to node environment.

### 2.6 List Plugin

- Prints tasks and commands from the graph.
- Handles task documentation functionality (retrieving and displaying).

## 3. Requirements

### 3.1 Core Graph Management

- Parse script files into a node structure.
- Handle dependencies and prevent cycles.
- Provide debugging interface (e.g., ASCII graph visualization).

### 3.2 Plugin Lifecycle

1. **Initialization phase**
2. **Graph transformation phase**
3. **Execution preparation phase**
4. **Before run phase**
5. **Execution phase**
6. **After run phase**

### 3.3 Data and Metadata

- Allow plugins to modify node metadata.
- Define conflict resolution among plugins.
- Support structured data (e.g., JSON-compatible objects) on nodes.

### 3.4 Concurrency Support

- Handle parallel task execution.
- Support fail-fast behavior (if one task fails, optional immediate stop).
- Track task status (in-progress, succeeded, failed, canceled).

### 3.5 Watch Mode

- Monitor file changes.
- Re-run affected tasks on changes.
- Integrate with concurrency (watch triggers can also run concurrently with other tasks).

### 3.6 Environment Management

- Gather env vars from config files (`bodo.yaml` or other).
- Handle inheritance from global environment.
- Merge `PATH` variables correctly for each node.

### 3.7 Command Execution

- Asynchronous process management.
- Output logging with prefixes (if enabled).
- Environment integration (passing the final `env` to the child process).

### 3.8 Documentation

- Generate task listings in multiple formats.
- Include descriptions, arguments, environment variables, etc.

### 3.9 Error Handling

- Consistent error types.
- Error bubbling (surfacing the original error cause).
- User-friendly logging (with hints on how to fix issues).

### 3.10 Testing

- **Unit tests** per plugin.
- **Integration tests** (end-to-end scenario testing).
- **Graph validation tests** (ensuring correct dependency resolution).
- **Execution tests** (verifying the actual processes run as intended).

### 3.11 Plugin-Specific Tests

#### 3.11.1 Plugin Lifecycle Tests

- **`on_init` phase**:

  - Verify plugin configuration loading.
  - Test initialization error handling.
  - Confirm plugin state after initialization.

- **`on_graph_build` phase**:

  - Verify node metadata modifications.
  - Test graph transformation operations.
  - Validate error handling during graph building.

- **`on_execution_prepare` phase**:

  - Verify environment variable setup.
  - Test command prefix configuration.
  - Validate path modifications.

- **`before_run` phase**:

  - Verify pre-execution setup.
  - Test resource allocation.
  - Validate state preparation.
  - Test cancellation handling.

- **`on_execution` phase**:

  - Verify process management.
  - Test output handling.
  - Validate error propagation.

- **`after_run` phase**:
  - Verify resource cleanup.
  - Test state restoration.
  - Validate post-execution tasks.
  - Test error handling during cleanup.

#### 3.11.2 Plugin Metadata Tests

- **Test metadata conflict resolution**:

  - Multiple plugins modifying the same metadata.
  - Priority handling between plugins.
  - Metadata inheritance rules.

- **Test metadata validation**:
  - Type checking for metadata values.
  - Required vs optional metadata fields.
  - Invalid metadata handling.

#### 3.11.3 Plugin Error Tests

- **Test error types**:

  - Plugin initialization errors.
  - Graph transformation errors.
  - Execution preparation errors.
  - Runtime execution errors.

- **Test error handling**:
  - Error propagation through the plugin chain.
  - Graceful plugin disabling on errors.
  - Error recovery mechanisms.

#### 3.11.4 Plugin Integration Tests

- **Test plugin interactions**:

  - Environment + Path plugin cooperation.
  - Watch + Execution plugin integration.
  - Command Prefix + List plugin coordination.

- **Test plugin ordering**:
  - Verify correct execution order.
  - Test dependency resolution between plugins.
  - Validate plugin priority system.

## 4. Task File Format and Structure

Tasks are defined in a **YAML file**. Below are examples.

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

**Default configuration** in `bodo.toml`:

```toml
root_task_file_path = "scripts/tasks.yaml"
tasks_paths = ["scripts"]
```

**Custom configuration**:

```toml
root_task_file_path = "tasks/tasks.yaml"
tasks_paths = ["packages/*/tasks.yaml"]
```

## 5. Task Properties

- **`command`**: The command to run.
- **`pre_deps`**: The tasks that must be run before this task.
- **`post_deps`**: The tasks that will be run after this task.
- **`concurrently`**: The tasks that will be run concurrently with this task.
- **`description`**: The description of the task.
- **`env`**: The environment variables to set for the task.
- **`exec_paths`**: The paths to add to the PATH environment variable.
- **`args`**: The arguments options.
- **`cwd`**: The current working directory for the task.
- **`prefix_color`**: The color of the prefix (from the `colored` crate).

## 6. Task Configuration

### 6.1 Task References

**Basic task reference**:

```yaml
default_task:
  pre_deps:
    - task: test
tasks:
  test:
    command: "cargo test"
```

**Cross-file reference**:

```yaml
default_task:
  pre_deps:
    - task: ../other_tasks.yaml # default_task will be used
    - task: ./other_tasks.yaml/some_task # some_task from other_tasks.yaml will be used
```

### 6.2 Task Name Restrictions

- **max length**: 100
- **min length**: 1
- **Disallow special characters**:
  - `/` (used for relative paths)
  - `.` (used for current directory)
  - `..` (used for parent directory)
- **Task name must not be a reserved word**:
  - `watch`
  - `default_task`
  - `pre_deps`
  - `post_deps`
  - `concurrently`

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
bodo # default_task will be used
bodo <task_name> # test task will be used
bodo <task_name_dir_path> # runs that task from the given directory path
bodo <task_name_dir_path> <task_name> # runs <task_name> from <task_name_dir_path>
```

- `/` can refer to the root directory: `bodo scripts/test`
- Relative paths also work: `bodo ./scripts/test`

### 6.4 Custom Directory Structure

Configuration in `bodo.toml`:

```toml
root_task_file_path = "./tasks.yaml"
tasks_paths = ["./packages/*/tasks.yaml"]
```

## 7. Command Configuration

### 7.1 Basic Command Forms

**Simple command**:

```yaml
command: "cargo test"
```

**Shell command**:

```yaml
command:
  sh: "echo 'Hello, World!'"
```

### 7.2 Script Files

**Shell script**:

```yaml
command: ./path/to/script.sh
```

**Other script types**:

```yaml
command: ./path/to/script.ts
```

**Language-specific**:

```yaml
command:
  python: ./path/to/script.py
```

```yaml
command:
  js: ./path/to/script.js
```

### 7.3 Command Options

- **`name`**: The name of the task
- **`args`**: The arguments options
- **`silent`**: Whether to run the command silently. Will not print the command content to the console first
- **`cwd`**: The current working directory for the task
- **`env`**: The environment variables to set for the task
- **`exec_paths`**: The paths to add to the PATH environment variable
- **`description`**: The description of the task

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

**Command dependencies**:

```yaml
default_task:
  pre_deps:
    - command: "cargo test"
    - command: "cargo clippy"
  command: "cargo build"
```

### 8.2 Post-dependencies

Works exactly like `pre_deps`.

### 8.3 Concurrent Tasks

```yaml
default_task:
  concurrently:
    - task: test
    - task: lint
```

When `concurrently` is used, **no `command` is allowed.** However, the example below shows a `command` can also be part of `concurrently`:

```yaml
default_task:
  concurrently:
    - task: test
    - task: lint
    - command: echo "Hello, World!"
```

#### Concurrent Task Options

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

- **`max_concurrent_tasks`**: The maximum number of tasks to run concurrently
- **`prefix_output`**: Whether to prefix the output of the tasks
- **`fail_fast_on_error`**: Whether to fail fast if one of the tasks fails
- **`fail_fast_on_error_exit_code`**: The exit code to fail fast on
- **`fail_fast_on_error_exit_code_range`**: The range of exit codes to fail fast on

## 9. Additional Properties

### 9.1 Description

**Single line**:

```yaml
description: "Build the project"
```

**Multi-line**:

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

**Basic string argument**:

```yaml
args:
  - name: "name"
    description: "The name of the task"
    type: "string"
    default: "world"
```

**Enum argument**:

```yaml
args:
  - name: "name"
    description: "The name of the task"
    type: "enum"
    values: ["hello", "world"]
    default: "world"
```

**Number argument**:

```yaml
args:
  - name: "int"
    description: "The number of stuff"
    type: "number"
    default: 1
```

**Prompt argument**:

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

Plugins are the core of Bodo's extensibility. They are implemented as **traits** and can hook into each phase of the lifecycle or provide custom transformations.

### 11. CLI

The CLI is a thin layer on top of the manager and aims to be extremely user-friendly. Below are key patterns:

#### 11.1 CLI Commands

- **`bodo`** (no arguments)  
  Runs the default task from the root task file.
- **`bodo <task_name>`**  
  Runs `<task_name>` from the root task file.
- **`bodo <task_name_dir_path>`**  
  Runs that task from the given directory path.
- **`bodo <task_name_dir_path> <task_name>`**  
  Runs `<task_name>` from `<task_name_dir_path>`.

  - `/` can refer to the root directory: `bodo scripts/test`
  - Relative paths also work: `bodo ./scripts/test`

- **`bodo <task_name> -- <args>`**  
  Runs `<task_name>` with `<args>`.

- **`bodo watch`** or **`bodo --watch`**  
  Watches for file changes and reruns tasks.  
  `bodo --watch <task_name> <args>` similarly watches for changes and reruns `<task_name>` with `<args>`.

- **`bodo --list`**  
  Lists all tasks. If no root task file is specified, running `bodo` may list all tasks too.

- **`bodo --graph`**  
  Visualizes the graph using ASCII art.

- **`bodo --debug`**  
  Prints Bodo debug logs (must come after `bodo`).

- **`bodo --run`**  
  Runs a task. If no task is specified, runs the default task from the root file.

#### 11.2 CLI Flags

- **`-c, --config`**: The path to the configuration file
- **`-d, --debug`**: Debug the graph
- **`-g, --graph`**: Visualize the graph (ASCII)
- **`-l, --list`**: List all tasks
- **`-w, --watch`**: Watch for file changes and rerun tasks
- **`-r, --run`**: Run a task

## Internal Plugins

1. **resolver_plugin**  
   Resolves task references. It enhances the graph to add the references for each node, removing the `task: ...` references and replacing them with actual `command` or `concurrently` nodes.

2. **path_plugin**  
   Handles the `PATH` environment variable. It enhances the graph with a final PATH value for each node.

3. **env_plugin**  
   Handles environment variables. It enhances the graph with final env var values for each node.

4. **command_echo_plugin**  
   Prints the command before execution. It can be turned off by setting `silent: true`.

5. **command_prefix_plugin**  
   Handles output prefixing (e.g. `[build] `).

6. **execution_plugin**  
   Runs the commands. This is the main plugin that actually spawns child processes.

7. **watch_plugin**  
   Handles file watching by enhancing the graph with watch tasks.

8. **concurrent_plugin**  
   Adds concurrency tasks or wrappers. The executor plugin sees these tasks and runs them concurrently.

9. **timeout_plugin**  
   Adds a timeout to each node if specified (makes the command concurrent with a timeout process and fail-fast option).

Plugins transform the graph step by step, then **execution_plugin** runs the final graph.

### Example of Plugin Transformations

Given:

```yaml
env:
  FOO: global
  BAR: global
exec_paths:
  - ./node_modules/.bin
default_task:
  pre_deps:
    - task: c
  post_deps:
    - task: main
tasks:
  main:
    concurrently:
      - task: a
      - task: b
  a:
    timeout: 1000
    command: echo "A"
  b:
    silent: true
    command: echo "B"
  c:
    env:
      FOO: "bar"
    command: echo "$FOO"
```

**1. Timeout Plugin**  
Wraps task `a` in a concurrent structure with a timeout process:

```yaml
a:
  concurrently_options:
    fail_fast_on_error: true
  concurrently:
    - command: BODO_TIMEOUT_EXECUTOR $TIMEOUT_MS
  env:
    TIMEOUT_MS: 1000
  command: echo "A"
```

**2. command_echo_plugin**  
Sets an env var indicating whether to echo the command:

```yaml
b:
  env:
    BODO_ECHO_COMMAND: false
  command: echo "B"
```

**3. env_plugin**  
Merges global and local environment variables:

```yaml
c:
  env:
    FOO: "bar"
    BAR: "global"
  command: echo "$FOO"
```

**4. path_plugin**  
Sets the `PATH`:

```yaml
c:
  env:
    PATH: "/usr/local/bin:/usr/bin:/usr/local/bin/node_modules/.bin"
    FOO: "bar"
    BAR: "global"
  command: echo "$FOO"
```

**5. command_prefix_plugin**  
Adds a prefix to the output:

```yaml
c:
  env:
    PATH: "/usr/local/bin:/usr/bin:/usr/local/bin/node_modules/.bin"
    FOO: "bar"
    BAR: "global"
    BODO_PREFIX: "[build] "
  command: echo "$FOO"
```

**6. concurrent_plugin**  
Enhances the graph with concurrency logic.

**7. resolver_plugin**  
Resolves references:

```yaml
default_task:
  sequence_options:
    order: [1, 2]
  sequence:
    - command_id: 1
      env:
        FOO: "bar"
      command: echo "$FOO"
    - command_id: 2
      command: $BODO_RUN_CONCURRENTLY $BODO_CONCURRENTLY_COMMAND_IDS
      env:
        BODO_CONCURRENTLY_COMMAND_IDS: [5, 6, 7]
        BODO_CONCURRENTLY_OPTION_FAIL_FAST_ON_ERROR: true
    - command_id: 3
      command: $BODO_TIMEOUT_EXECUTOR $BODO_TIMEOUT_MS
      env:
        BODO_TIMEOUT_EXECUTOR: "timeout"
        BODO_TIMEOUT_MS: 1000
        BODO_KILL_COMMAND_IDS: [3]
    - command_id: 4
      env:
        BODO_ECHO_COMMAND: false
      command: echo "B"
```

**8. execution_plugin**  
Finally runs the commands in either sequence or concurrency according to the transformed graph.

### Custom Plugins

Custom plugins implement the `Plugin` trait and can hook into any phase to modify the graph or the runner. For example:

- **`on_init`**: Load custom configurations.
- **`on_graph_build`**: Add or remove tasks, transform metadata, etc.
- **`on_execution_prepare`**: Adjust environment or concurrency options.
- **`before_run`** / **`after_run`**: Setup or cleanup tasks.

## Implementation

### Graph Manager

First step is to build a solid graph amanager.

Responsibilities:

- Build a graph consisting of script files, tasks, commands, and their dependencies.
- Add configuration option for how to build the graph. where root task file is, where to find tasks, duplicate task names, duplicate script parent directories which can cause issues, etc.
- Add methods to find tasks and commands in the graph.
- Throw appropriate errors if there are any issues with the graph. E.g. not found, circular dependency, etc.
- Provide a way to visualize the graph but not implement it. Provide a nice API for plugins to visualize the graph.
- Cover with tests.

### CLI

- Build a CLI that is easy to use and understand.
- In this step add only one plugin, a dummy executor that just prints the command when `bodo --run` is used.
- Cover with tests.
- CLI should understand the graph manager and be able to use it.
- CLI should pass the right arguments to the graph manager.

### Task Resolver

Task resolver is responsible for resolving the tasks and commands in the graph.

First step it will convert `task: ...` references to actual `command` nodes.

TBD
