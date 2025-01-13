
# BODO

A task runner with intuitive organization and powerful features. The name “BODO” comes from Farsi, meaning “run,” and it’s fast to type on a QWERTY keyboard.

## Who is it for?

BODO is for developers who want to run tasks in a structured way, but don't want to deal with the complexity of a full-fledged build system. It is designed to be simple, fast, and easy to use.

BODO is optimized for large monorepos where each team might own a few dozen scripts. BODO is not a build system, but it can be used to build things. It does not handle build artifacts, and it does not have a concept of a build pipeline. It's meant to replace `npm run` and `yarn run` for most use cases as well as having many one-off scripts that don't fit into a build pipeline.

BODO is written in Rust and is designed to be fast and efficient.

## Installation

```bash
npm install -g bodo
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
  By default, watch mode monitors the entire `scripts/<subdirectory>` directory for changes. You can configure different folders or file types in your `bodo.yaml` if needed. Any file changes trigger a re-run of the selected task or subtask.

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
    custom_script.ts
  deploy/
    script.yaml
    custom_plugin.ts
.env
.env.local
```

### Example `script.yaml`

```yaml
# scripts/script.yaml
name: My Script
description: All the things I want to do

# Add paths to the script’s execution context (like adding to PATH)
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

  # Example of concurrency with multiple commands
  lint:
    concurrently:
      - command: "eslint src"
      - command: "prettier --check ."

  test:
    # Watch only certain files if desired
    watch:
      patterns:
        - "**/*.test.ts"
    pre_deps:
      - defaultTask
    command: "jest"

  # Run a custom TypeScript script via tsx
  validate-package:
    command:
      ts: ./validate-package.ts

  # Demonstration of dependencies and arguments
  deploy:
    pre_deps:
      - defaultTask
      - lint
    post_deps:
      - command:
          sh: "echo 'Deployed!'"

    args:
      - name: target
        help: "The target to deploy to"
        type: enum
        enums:
          - development
          - production
        required: true
        interactive: false
      - name: force
        help: "Force the deployment"
        type: boolean
        default: false

    commands:
      - command: "echo 'Deploying to {{ ARGS.target }}'"
      - command: "vercel deploy --target {{ ARGS.target }}"
```

## Environment Variables

BODO automatically loads environment variables from `.env` in the project root. For instance:

```
# .env
MY_GLOBAL_ENV=123
```

```yaml
# scripts/env-example.yaml
envFiles:
  - .env
  - .env.local

env:
  MY_GLOBAL_ENV: "overridden-value"
defaultTask:
  command: "echo 'MY_GLOBAL_ENV = {{ ENV.MY_GLOBAL_ENV }}'"
```
If `.env` had `MY_GLOBAL_ENV=123`, this script overrides it with `overridden-value`.

## Templating

BODO uses [EJS](https://ejs.co/) for templating. You can insert variables with `{{ }}`. Here’s a quick loop/conditional example:

```yaml
# scripts/ejs-example.yaml
defaultTask:
  vars:
    items:
      - alpha
      - beta
      - gamma
  command: |
    <% if (VARS.items && VARS.items.length) { %>
      <% for (let i = 0; i < VARS.items.length; i++) { %>
        echo "Item <%= i %>: <%= VARS.items[i] %>"
      <% } %>
    <% } else { %>
      echo "No items found."
    <% } %>
```

## Extending the Command DSL

By default, you can specify shell commands with `command: "some-command"` or TypeScript/JavaScript scripts with:
```yaml
command:
  ts: ./script.ts
```
You can add new keywords (`go:`, `python:`, etc.) by editing BODO’s internal command resolvers or writing a plugin that interprets your custom field.

## Plugins

Create custom plugins in `plugins/`. Each plugin can hook into tasks before and after execution.

```ts
// plugins/my-plugin.ts
import { Plugin, OnBeforeRunOptions, OnAfterRunOptions } from 'bodo'

export default class MyPlugin extends Plugin {
  onBeforeRun(options: OnBeforeRunOptions) {
    console.log("Hello from the plugin!")
  }
  onAfterRun(options: OnAfterRunOptions) {
    console.log("Goodbye from the plugin!")
  }
}
```

Apply it globally in `bodo.yaml`:
```yaml
# bodo.yaml
globalPlugins:
  - ./plugins/my-plugin.ts
```

Or apply it per script/task:
```yaml
# scripts/with-plugin.yaml
plugins:
  - ./plugins/my-plugin.ts

defaultTask:
  command: "echo 'Hello, world!'"
```

## Watch Mode

Watch mode re-runs tasks on file changes. By default, it monitors the entire subdirectory. Specify custom watch paths in your script or global config:
```yaml
# scripts/watch-override.yaml
watch:
  patterns:
    - "src/**/*.ts"
    - "tests/**/*.ts"
```
Usage:
```bash
bodo watch <subdirectory>
```

## Concurrency

Use `concurrently` to run multiple commands in parallel within a single task:
```yaml
lint:
  concurrently:
    - command: "eslint src"
    - command: "prettier --check ."
```
Or define concurrency on multiple tasks by setting a global `maxConcurrency` in `bodo.yaml`. BODO executes tasks in parallel if no dependencies force a strict sequence.

## `bodo.yaml`

`bodo.yaml` (or `bodo.yml`, `bodo.json`, etc.) is optional and configures BODO globally. Example:

```yaml
# bodo.yaml
globalPlugins:
  - ./plugins/log-plugin.ts
maxConcurrency: 10
tidyUp: true
executableMap:
  - extensions:
      - ts
      - tsx
    executable: npx tsx
  - extensions:
      - sh
      - bash
    executable: bash
  - extensions:
      - js
      - mjs
      - cjs
    executable: node
  - extensions:
      - py
    executable: python
```

- **globalPlugins**: Loaded for every task.
- **maxConcurrency**: Limits the number of tasks or commands running in parallel.
- **tidyUp**: Cleans up and reformats your task definitions before runtime.
- **executableMap**: Maps file extensions to executable names.
## FAQ / Gotchas

1. **Quoting on Windows**  
   If you’re on Windows, remember that some shells handle quotes differently. Ensure your command strings match your local shell.

2. **Path Expansion**  
   Relative paths may differ if the working directory is not set as you expect. Use absolute or carefully relative paths in your commands.

3. **Env Overrides**  
   If you define variables in multiple places, the last definition takes precedence. Keep track of your merges if you use `.env`, `.env.local`, or `env:` blocks.

4. **Multiple Shell Commands**  
   If you chain commands, watch for `&&` vs. separate lines. BODO can split them or run them concurrently; check your concurrency setup.

5. **Templating Logic**  
   EJS uses `<% %>` for logic and `<%= %>` for printing. Don’t forget the difference when looping or conditionally inserting text.

## Features Recap

- Built-in CLI argument parsing
- Automatic environment variable loading
- Multiple subtasks and dependencies
- Concurrency and watch mode
- Shell, JavaScript, or TypeScript commands
- Plugin system for custom functionality
- EJS templating for complex tasks

## Contributing

Fork, branch, commit, and open a pull request.

## License

MIT
