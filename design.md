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
- Before run phase
- Execution phase
- After run phase

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

### 3.11 Plugin-Specific Tests

#### 3.11.1 Plugin Lifecycle Tests

- Test `on_init` phase:

  - Verify plugin configuration loading
  - Test initialization error handling
  - Confirm plugin state after initialization

- Test `on_graph_build` phase:

  - Verify node metadata modifications
  - Test graph transformation operations
  - Validate error handling during graph building

- Test `on_execution_prepare` phase:

  - Verify environment variable setup
  - Test command prefix configuration
  - Validate path modifications

- Test `before_run` phase:

  - Verify pre-execution setup
  - Test resource allocation
  - Validate state preparation
  - Test cancellation handling

- Test `on_execution` phase:

  - Verify process management
  - Test output handling
  - Validate error propagation

- Test `after_run` phase:

  - Verify resource cleanup
  - Test state restoration
  - Validate post-execution tasks
  - Test error handling during cleanup

#### 3.11.2 Plugin Metadata Tests

- Test metadata conflict resolution:

  - Multiple plugins modifying same metadata
  - Priority handling between plugins
  - Metadata inheritance rules

- Test metadata validation:
  - Type checking for metadata values
  - Required vs optional metadata fields
  - Invalid metadata handling

#### 3.11.3 Plugin Error Tests

- Test error types:

  - Plugin initialization errors
  - Graph transformation errors
  - Execution preparation errors
  - Runtime execution errors

- Test error handling:
  - Error propagation through plugin chain
  - Graceful plugin disabling on errors
  - Error recovery mechanisms

#### 3.11.4 Plugin Integration Tests

- Test plugin interactions:

  - Environment + Path plugin cooperation
  - Watch + Execution plugin integration
  - Command Prefix + List plugin coordination

- Test plugin ordering:
  - Verify correct execution order
  - Test dependency resolution between plugins
  - Validate plugin priority system

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

## Testing plan for core functionality

Below is a categorized list of tests that would be valuable for ensuring the correctness of the core Bodo code (graph construction, config loading, script loading) in its stripped-down, no-plugin state. Many of these can be unit tests (testing small pieces in isolation) or integration tests (verifying multiple parts in tandem).

1. Graph Tests

1.1 Node Creation
• Test: Create an empty Graph. Confirm nodes and edges are both empty.
• Test: Add a single Task node. Confirm the node’s ID is 0, nodes.len() is 1, and the node has correct data (TaskData).
• Test: Add multiple Command nodes. Confirm each node ID increments, and nodes.len() matches the count.
• Test: Confirm that node metadata is empty by default when you add new nodes.

1.2 Edge Creation
• Test: Add an edge between two valid node IDs (0 -> 1). Confirm edges.len() is 1 and that the stored edge is correct.
• Test: Add multiple edges. Confirm the final edges.len() is as expected and edges are stored in the order they were added.
• (Optional): Attempt to add an edge with an invalid node ID (e.g., from 999 to 1000). Confirm it either panics or that your system has a safe check for invalid IDs (depending on your design choice).

1.3 Graph Debug Print
• Test: With a small number of nodes/edges, call print_debug() and capture its stdout output. Confirm it contains the correct node count and edge references.

2. Script Loader Tests

2.1 Loading a Single YAML File
• Test: Minimal YAML with only a default_task (simple command). Confirm the graph has exactly 1 node, which is a Command node, and the raw_command is as expected.
• Test: YAML with tasks map containing multiple tasks. Confirm the graph node count matches the number of tasks + (optional) default task. Verify the correct TaskData details are stored.
• Test: YAML in which default_task is a “complex task” (with command, description, maybe a concurrently placeholder). Confirm the resulting node is still recognized as a Command, and the right fields appear in metadata if you store them.

2.2 Loading Multiple YAML Files in One Directory
• Test: Directory with scriptA.yaml and scriptB.yaml. Confirm both files load into the graph (e.g., 2 default tasks from each file, multiple named tasks).
• Test: Nested directories: place .yaml in subfolders. Ensure WalkDir picks them up if your config says "scripts/".

2.3 Using a Glob
• Test: Provide a glob pattern in bodo.toml like "scripts/**/\*.yaml" or "**/script.yaml". Confirm it recursively loads all matching .yaml files.
• Test: Provide a broken glob or an empty matching set. Confirm no panic occurs, either zero files loaded or an error is returned (depending on design).

2.4 Invalid Files
• Test: File not found or no .yaml in scripts/. Confirm the graph ends up with zero nodes, or the loader returns an error if that’s desired.
• Test: Malformed YAML syntax. Confirm it returns a PluginError::GenericError with a “YAML parse error:” message.
• Test: Unexpected data structure (like top-level keys that don’t match ScriptFile definitions). Confirm it returns a parse error or gracefully ignores unknown fields if you’ve set #[serde(default)] on those fields.

2.5 Edge Cases in ScriptFile.to_graph()
• Test: default_task and tasks both empty. Confirm no nodes are created, but no panic occurs.
• Test: default_task is present but has an empty command string. Ensure the node still becomes a Command node (with possibly an empty command) or that you handle it as a no-op.
• Test: Named tasks that have an empty command. Confirm a Task node is created with some default or empty metadata.

3. BodoConfig Loading Tests

3.1 Default Config
• Test: No bodo.toml file in the current directory. Confirm BodoConfig::default() is used, and script_paths is None.

3.2 Valid bodo.toml
• Test: A minimal TOML specifying script_paths = ["custom-scripts/"]. Confirm the config is loaded and script_paths is Some(vec!["custom-scripts/"]).
• Test: A more complex TOML with extra fields (which your struct might ignore if not declared). Confirm no parse error if the extra fields are harmless.

3.3 Invalid bodo.toml
• Test: Malformed TOML content. Confirm you get PluginError::GenericError("bodo.toml parse error: ...").
• Test: Missing read permissions on bodo.toml. Confirm you get an IoError or GenericError referencing the inability to read the file.

4. GraphManager Tests

4.1 GraphManager::new()
• Test: Confirm manager starts with an empty graph and a default BodoConfig.

4.2 load_bodo_config()
• Test: Provide a path to a valid bodo.toml. After calling load_bodo_config(Some("my-config.toml")), confirm the manager’s self.config matches what’s in the file.
• Test: Provide None. Confirm it tries bodo.toml in the current directory. If none found, confirm it remains default.

4.3 build_graph()
(Integration with script_loader::load_scripts_from_fs)
• Test: With a known scripts directory containing 2 YAML files. After build_graph(), confirm the graph has the correct nodes.
• Test: If no scripts exist, confirm you either get 0 nodes or an error (depending on design).
• Test: If one of the YAML files is invalid. Confirm build_graph() returns an error.
• Test: If you want to do some extra validation (like cycle detection or name checking), add a test that ensures invalid references are caught.

5. Integration / End-to-End Tests

5.1 Minimal Project Directory Setup 1. Create a temp directory. 2. Write a minimal bodo.toml specifying script_paths = ["scripts/"]. 3. Make a scripts/ folder with a script.yaml containing a default task. 4. Run a small “main” function or a test harness that calls GraphManager::new(), load_bodo_config(...), build_graph(). 5. Assert the manager’s graph.nodes.len() == 1.

5.2 Multiple YAML + Overlapping Paths
• If bodo.toml sets script_paths = ["scripts/", "other-scripts/"], place valid YAML in both. Confirm nodes from both directories appear in the final graph.

5.3 Edge Cases
• Scripts folder is huge but only has one .yaml. Confirm performance is reasonable or the code doesn’t blow up.
• A script references advanced fields you haven’t implemented yet (e.g. pre_deps:, post_deps:). Confirm they’re just ignored or stored as raw data if you do so.

6. Suggested Additional Structural or Sanity Tests

6.1 Task Name Validation
If in the future you want to enforce name constraints (no slashes, no .., etc.), write tests verifying that attempts to parse invalid task names produce an error or are sanitized.

6.2 Graph Consistency
If you implement a method that verifies no circular references or duplicated node IDs, write tests that feed in a contrived script with a cycle. Confirm the code flags it.

6.3 Performance or Memory
Not typically a big issue at early stage, but you could do basic tests loading 100+ scripts or tasks to confirm it doesn’t degrade badly.

Next Steps 1. Unit Tests: Place them in the same file under a #[cfg(test)] mod tests or in a separate tests/ folder. 2. Integration Tests: Typically live in the tests/ directory, pulling in your library as a normal crate. 3. Mock File Structures: For file-based tests (like bodo.toml, script.yaml), you can use tempfile or assert_fs crates to create ephemeral directories.

This covers a broad range of scenarios so that once you add plugins or advanced features later, you’ll have confidence the base graph-loading logic remains solid.

## Development Plan

Note:
We use tokio for all processes management.
Watch is a form of concurrency.

    1.	Add plugin-specific tests. Include tests for each plugin lifecycle method (on_init, on_graph_build, etc.). Verify that plugins correctly modify node metadata and respond to errors.
    2.	Expand concurrency tests:
    •	Confirm fail_fast behavior kills remaining tasks if one fails.
    •	Validate timeouts by forcibly causing tasks to sleep beyond the deadline.
    •	Check output prefixing in concurrent tasks and make sure logs are readable.
    3.	Implement watch mode tests:
    •	Use a temporary directory and a mocked file watch to trigger task reruns on file changes.
    •	Ensure that only relevant tasks rerun and that concurrency logic still holds.
    4.	Introduce environment and path plugin validations:
    •	Confirm PATH merges correctly (especially with multiple exec_paths).
    •	Test environment variable conflicts and overrides.
    5.	Improve error handling tests:
    •	Raise and catch custom plugin errors.
    •	Confirm graceful shutdown or fallback when one plugin fails.
    6.	Provide a CLI integration test:
    •	Build a small binary that runs Bodo commands.
    •	Validate arguments, flags, and subcommands (if any).
    •	Capture stdout/stderr to confirm correct usage and error messages.
    7.	Add advanced dependency tests:
    •	Check that cycles are detected or prevented.
    •	Validate complex pre/post dependencies across multiple scripts.
    8.	Ensure more coverage for task properties:
    •	Confirm working_dir (cwd) changes command execution directory.
    •	Check handling of silent tasks or tasks without any command.
    9.	Expand the documentation plugin (or “list plugin”) and test:
    •	Ensure tasks and commands are listed in expected formats.
    •	Test color/no-color output, and any config overrides.
    10.	Optimize performance tests:

    •	Push node count higher (e.g., 50k tasks) to find any bottlenecks.
    •	Confirm memory usage remains stable under large loads.

    11.	Consider integration with external shells or interpreters:

    •	Python scripts, Node.js scripts, etc.
    •	Test that PATH or environment changes apply correctly for each shell or interpreter type.

    12.	Start capturing code coverage metrics:

    •	Integrate coverage reporting to identify untested lines.
    •	Set thresholds to avoid regressions.

    13.	Release a minimal alpha version:

    •	Include essential features: graph building, concurrency, plugin basics.
    •	Gather feedback on CLI, performance, error messages.

    14.	Plan for future expansions:

    •	Plugin for auto-completion or interactive features.
    •	Reusable plugin architecture for external contributions.
