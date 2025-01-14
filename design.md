# Bodo Design Document

## Core Architecture

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

## Plugin-Based Architecture

Everything beyond core graph management is implemented as plugins:

### Environment Variables Plugin

- Manages environment variables
- Tracks final env var values on graph nodes

### Command Prefix Plugin

- Handles command output prefixing (e.g. `[build] building...`)
- Configures prefixes on task/command nodes

### Execution Plugin

- Uses Tokio for process management
- Handles script and task execution

### Watch Plugin

- Uses Tokio for file watching
- Manages watched files and task triggers

### Path Plugin

- Computes final PATH for each node
- Adds exec_paths to node environment

### List Plugin

- Prints tasks and commands from graph
- Handles task documentation

## Requirements

### 1. Core Graph Management

- Parse script files into node structure
- Handle dependencies and prevent cycles
- Provide debugging interface

### 2. Plugin Lifecycle

- Initialization phase
- Graph transformation phase
- Execution preparation phase
- Execution phase

### 3. Data and Metadata

- Allow plugins to modify node metadata
- Define conflict resolution
- Support structured data on nodes

### 4. Concurrency Support

- Handle parallel task execution
- Support fail-fast behavior
- Track task status

### 5. Watch Mode

- Monitor file changes
- Re-run affected tasks
- Integrate with concurrency

### 6. Environment Management

- Gather env vars from config
- Handle inheritance
- Merge PATH correctly

### 7. Command Execution

- Async process management
- Output logging with prefixes
- Environment integration

### 8. Documentation

- Generate task listings
- Support multiple formats
- Include descriptions

### 9. Error Handling

- Consistent error types
- Error bubbling
- User-friendly logging

### 10. Testing

- Unit tests per plugin
- Integration tests
- Graph validation tests
- Execution tests
