# `bodo`

A task runner with intuitive organization and powerful features.

> The name "bodo" comes from Farsi, meaning "run," and it's fast to type on a QWERTY keyboard.

## Who is this for?

- Bodo is made for large repos with a lot of scripts
- You have a huge `Makefile`/`package.json`/other script runner and you want to organize it
- You have lots of scripts in various languages
- You want each team to own their own scripts and enforce standards
- You want to enforce `CODEOWNERS` for scripts

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

#### Run tasks in parallel

bodo supports running tasks in parallel similar to [`concurrently`](https://github.com/open-cli-tools/concurrently).

```yaml
# scripts/test/script.yaml
defaultTask:
  concurrently:
    # mix tasks and commands
    - task: test
    - task: lint
    - command: "cargo fmt"

    # bring tasks from other scripts
    - task: code-quality/spellcheck

subtasks:
  test:
    command: "cargo test"

  lint:
    command: "cargo clippy"
```

## Plugin System

bodo provides a powerful plugin system that allows you to hook into every stage of the task runner lifecycle:

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

- `on_bodo_init`: Run during bodo initialization
- `on_bodo_exit`: Run before bodo exits

### Creating a Plugin

Plugins can be written in Rust or TypeScript. Here are examples in both languages:

#### Rust Plugin

```rust
use bodo::plugin::BodoPlugin;

struct MyPlugin;

impl BodoPlugin for MyPlugin {
    fn on_before_task_run(&mut self, task_name: &str) {
        println!("Starting task: {}", task_name);
    }

    fn on_after_task_run(&mut self, task_name: &str, status_code: i32) {
        println!("Task {} finished with status {}", task_name, status_code);
    }

    fn on_error(&mut self, task_name: &str, err: &dyn std::error::Error) {
        eprintln!("Task {} failed: {}", task_name, err);
    }

    fn on_resolve_command(&mut self, task: &mut TaskConfig) {
        // Transform commands or inject environment variables
        if let Some(env) = &mut task.env {
            env.insert("DEBUG".to_string(), "1".to_string());
        }
    }

    fn on_command_ready(&mut self, command: &str, task_name: &str) {
        println!("Executing command '{}' for task '{}'", command, task_name);
    }

    fn on_bodo_exit(&mut self, exit_code: i32) {
        println!("Bodo exiting with code {}", exit_code);
    }
}
```

#### TypeScript Plugin

```typescript
// plugins/my-plugin.ts
import { Plugin, TaskConfig } from "bodo";

export class MyPlugin implements Plugin {
  onBeforeTaskRun(opts: { taskName: string; cwd: string }): void {
    console.log(`Starting task: ${opts.taskName} in ${opts.cwd}`);
  }

  onAfterTaskRun(opts: { taskName: string; status: number }): void {
    console.log(`Task ${opts.taskName} finished with status ${opts.status}`);
  }

  onError(opts: { taskName: string; error: string }): void {
    console.error(`Task ${opts.taskName} failed: ${opts.error}`);
  }

  onResolveCommand(opts: { task: TaskConfig }): void {
    // Transform commands or inject environment variables
    if (opts.task.command.startsWith("ts-node")) {
      opts.task.command = `npx ${opts.task.command}`;
    }
  }

  onCommandReady(opts: { command: string; taskName: string }): void {
    console.log(`Command ready for ${opts.taskName}:`, opts.command);
  }

  onBodoExit(opts: { exitCode: number }): void {
    console.log("Bodo exiting with code:", opts.exitCode);
  }
}
```

#### Python Plugin

```python
# plugins/my_plugin.py
from typing import Dict, Any, Optional

def on_before_task_run(opts: Dict[str, Any]) -> None:
    print(f"Starting task: {opts['taskName']} in {opts['cwd']}")

def on_after_task_run(opts: Dict[str, Any]) -> None:
    print(f"Task {opts['taskName']} finished with status {opts['status']}")

def on_error(opts: Dict[str, Any]) -> None:
    print(f"Task {opts['taskName']} failed: {opts['error']}")

def on_resolve_command(opts: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    task = opts['task']
    if task['command'].startswith('python'):
        task['env'] = task.get('env', {})
        task['env']['PYTHONPATH'] = './src'
    return task

def on_command_ready(opts: Dict[str, Any]) -> None:
    print(f"Command ready for {opts['taskName']}: {opts['command']}")

def on_bodo_exit(opts: Dict[str, Any]) -> None:
    print(f"Bodo exiting with code: {opts['exitCode']}")
```

#### Ruby Plugin

```ruby
module BodoPlugin
  class << self
    def on_before_task_run(opts)
      puts "Starting task: #{opts['taskName']} in #{opts['cwd']}"
    end

    def on_after_task_run(opts)
      puts "Task #{opts['taskName']} finished with status #{opts['status']}"
    end

    def on_error(opts)
      puts "Task #{opts['taskName']} failed: #{opts['error']}"
    end

    def on_resolve_command(opts)
      task = opts['task']
      if task['command'].start_with?('bundle')
        task['env'] ||= {}
        task['env']['BUNDLE_PATH'] = 'vendor/bundle'
      end
      task
    end

    def on_command_ready(opts)
      puts "Command ready for #{opts['taskName']}: #{opts['command']}"
    end

    def on_bodo_exit(opts)
      puts "Bodo exiting with code: #{opts['exitCode']}"
    end
  end
end
```

### Registering Plugins

Register plugins in your `bodo.yaml`:

```yaml
plugins:
  # Rust plugin
  - path: "./plugins/my_plugin.rs"
  # TypeScript plugin
  - path: "./plugins/my-plugin.ts"
  # Python plugin
  - path: "./plugins/my_plugin.py"
  # Ruby plugin
  - path: "./plugins/my_plugin.rb"
```

Or register them programmatically:

```rust
// Rust plugin
let mut plugin_manager = PluginManager::new();
plugin_manager.register_plugin(PathBuf::from("./plugins/my_plugin.rs"));
```

```typescript
// TypeScript plugin
const plugin_manager = new PluginManager();
plugin_manager.registerPlugin(new MyPlugin());
```

## Environment Variables

bodo automatically loads environment variables from `.env` in the project root:

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

`bodo.yaml` (or `bodo.yml`, `bodo.json`) configures bodo globally:

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
