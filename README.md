# BODO

A task runner with intuitive organization and powerful features. The name "BODO" comes from Farsi, meaning "run," and it's fast to type on a QWERTY keyboard.

## Features

- Task organization by directory
- Powerful plugin system with lifecycle hooks
- Watch mode for development
- Environment variable management
- Task dependencies and concurrency
- Custom command resolvers

## Installation

```bash
cargo install bodo
```

## Usage

Create a `scripts/` directory in your project root. Each subdirectory in `scripts/` represents a task group. Inside each subdirectory, add a `script.yaml` defining commands, arguments, environment variables, dependencies, and more.

### Basic Commands

- **Default task**:
  ```bash
  bodo <subdirectory>
  ```

- **Subtask**:
  ```bash
  bodo <subdirectory> <subtask>
  ```

- **Watch mode**:
  ```bash
  bodo watch <subdirectory> [<subtask>]
  ```

### Structure Example

```
project-root/
bodo.yaml (optional)
scripts/
  script.yaml            <-- root-level tasks
  build/
    script.yaml
  test/
    script.yaml
  deploy/
    script.yaml
.env
.env.local
```

### Example `script.yaml`

```yaml
name: My Script
description: All the things I want to do

# Add paths to the script's execution context (like adding to PATH)
exec_paths:
  - node_modules/.bin

# Set environment variables for this script
env:
  NODE_OPTIONS: --max-old-space-size=4096

# The default task is invoked by simply running `bodo`
defaultTask:
  command: "tsc -p tsconfig.json"

# Subtasks
subtasks:
  clean:
    command: "rm -rf dist"

  lint:
    command: "cargo clippy"

  test:
    pre_deps:
      - defaultTask
    command: "cargo test"
```

## Plugin System

BODO provides a powerful plugin system that allows you to hook into every stage of the task runner lifecycle:

### Graph Construction Hooks
- `on_task_graph_construct_start`: Modify tasks before graph construction
- `on_task_graph_construct_end`: Inspect or modify the final dependency graph

### Command Resolution Hooks
- `on_resolve_command`: Transform commands or inject environment variables
- `on_command_ready`: Inspect the final resolved command

### Execution Lifecycle Hooks
- `on_before_run`: Run before task execution
- `on_after_run`: Run after task execution
- `on_error`: Handle task errors

### Watch Hooks
- `on_before_watch`: Modify watch patterns
- `on_after_watch_event`: React to file changes

### Global Lifecycle Hooks
- `on_bodo_init`: Run during BODO initialization
- `on_bodo_exit`: Run before BODO exits

### Creating a Plugin

Plugins can be written in Rust or TypeScript. Here are examples in both languages:

#### Rust Plugin

```rust
use bodo::plugin::BodoPlugin;

struct MyPlugin;

impl BodoPlugin for MyPlugin {
    fn on_before_run(&mut self, task_name: &str) {
        println!("Starting task: {}", task_name);
    }

    fn on_after_run(&mut self, task_name: &str, status_code: i32) {
        println!("Task {} finished with status {}", task_name, status_code);
    }

    fn on_error(&mut self, task_name: &str, err: &dyn std::error::Error) {
        eprintln!("Task {} failed: {}", task_name, err);
    }
}
```

#### TypeScript Plugin

```typescript
// plugins/my-plugin.ts
import { Plugin, TaskConfig, TaskGraph } from 'bodo';

export class MyPlugin implements Plugin {
    onBodoInit(config: Record<string, any>): void {
        console.log('BODO initialized with config:', config);
    }

    onTaskGraphConstructStart(tasks: TaskConfig[]): void {
        console.log('Building task graph with tasks:', tasks);
    }

    onTaskGraphConstructEnd(graph: TaskGraph): void {
        console.log('Task graph constructed:', graph);
    }

    onResolveCommand(task: TaskConfig): void {
        // Transform commands or inject environment variables
        if (task.command.startsWith('ts-node')) {
            task.command = `npx ${task.command}`;
        }
    }

    onCommandReady(command: string, taskName: string): void {
        console.log(`Command ready for ${taskName}:`, command);
    }

    onBeforeRun(taskName: string): void {
        console.log(`Starting task: ${taskName}`);
    }

    onAfterRun(taskName: string, statusCode: number): void {
        console.log(`Task ${taskName} finished with status ${statusCode}`);
    }

    onError(taskName: string, error: Error): void {
        console.error(`Task ${taskName} failed:`, error);
    }

    onBeforeWatch(patterns: string[]): void {
        console.log('Starting watch mode with patterns:', patterns);
    }

    onAfterWatchEvent(changedFile: string): void {
        console.log('File changed:', changedFile);
    }

    onBodoExit(exitCode: number): void {
        console.log('BODO exiting with code:', exitCode);
    }
}
```

### Registering Plugins

Register plugins in your `bodo.yaml`:

```yaml
plugins:
  # Rust plugin
  - path: "./plugins/my_plugin.rs"
  # TypeScript plugin
  - path: "./plugins/my-plugin.ts"
```

Or register them programmatically:

```rust
// Rust plugin
let mut plugin_manager = PluginManager::new(config);
plugin_manager.register_plugin(Box::new(MyPlugin));
```

```typescript
// TypeScript plugin
const plugin_manager = new PluginManager(config);
plugin_manager.registerPlugin(new MyPlugin());
```

## Environment Variables

BODO automatically loads environment variables from `.env` in the project root:

```
# .env
MY_GLOBAL_ENV=123
```

```yaml
# scripts/env-example.yaml
env:
  MY_GLOBAL_ENV: "overridden-value"
defaultTask:
  command: "echo $MY_GLOBAL_ENV"
```

## Watch Mode

Watch mode re-runs tasks on file changes:

```yaml
# scripts/watch-override.yaml
watch:
  patterns:
    - "src/**/*.rs"
    - "tests/**/*.rs"
```

Usage:
```bash
bodo watch <subdirectory>
```

## Configuration

`bodo.yaml` (or `bodo.yml`, `bodo.json`) configures BODO globally:

```yaml
# Maximum number of concurrent tasks
max_concurrency: 4

# Global plugins
plugins:
  - path: "./plugins/logger.rs"
  - path: "./plugins/metrics.rs"

# Global environment files
env_files:
  - .env
  - .env.local
```
