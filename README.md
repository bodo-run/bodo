<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./docs/logo/logo-black-bg.svg">
    <source media="(prefers-color-scheme: light)" srcset="./docs/logo/logo-white-bg.svg">
    <img alt="bodo logo" src="./docs/logo/logo-white-bg.svg" width="200">
  </picture>
</div>

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
curl -fsSL https://bodo.run | bash
```

## Quick Start

Create a scripts/ directory in your project root and make a script.yaml file in it:

```yaml
# scripts/script.yaml
default_task:
  command: "echo 'Hello, World!'"
```

Then run:

```bash
bodo
```

## Design Principles

- Fast: bodo is designed to be fast and efficient
- Plugins architecture: Most things are implemented as a Rust plugin if possible
- User friendly: bodo is designed to be user friendly and easy to understand
- Forgiving: bodo is designed to be forgiving and not strict
- Helpful errors: bodo provides helpful errors and suggestions
- Highly customizable: bodo is designed to be highly customizable and flexible

## Usage

Create a scripts/ directory in your project root. Each subdirectory in scripts/ represents a task group. Inside each subdirectory, add a script.yaml defining commands, arguments, environment variables, dependencies, and more.

### Basic Commands

- Default task:

```bash
bodo <subdirectory>
```

- Subtask:

```bash
bodo <subdirectory> <subtask>
```

- Watch mode:

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

### Example script.yaml

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
default_task:
  command: "tsc -p tsconfig.json"

# tasks
tasks:
  clean:
    command: "rm -rf dist"
  lint:
    command: "cargo clippy"
  test:
    pre_deps:
      - default_task
    command: "cargo test"
```

### Run tasks in parallel

```yaml
# scripts/test/script.yaml
default_task:
  concurrently:
    - task: test
    - task: lint
    - command: "cargo fmt"
    - task: code-quality/spellcheck

tasks:
  test:
    command: "cargo test"
  lint:
    command: "cargo clippy"
```
