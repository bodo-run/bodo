<div align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./docs/logo/logo-black-bg.svg">
    <source media="(prefers-color-scheme: light)" srcset="./docs/logo/logo-white-bg.svg">
    <img alt="bodo logo" src="./docs/logo/logo-white-bg.svg" width="200">
  </picture>
</div>

# `bodo`

> [!WARNING]
> This project is currently in development and is not ready for production use. It's open source just so I can use free CI!

Read the [Design Document](./DESIGN.md) to see what am I cooking. 👨🏽‍🍳

If you are adventurous, read the [Usage Document](./USAGE.md) to see what you can do with Bodo today.



Bodo is a task runner with intuitive organization and powerful features.

> The name "bodo" بدو comes from Farsi, meaning "run" and it's fast to type on a QWERTY keyboard.

## Who is this for?

- Bodo is made for large repos with a lot of scripts and many people working on them
- You have a huge `Makefile`/`package.json`/other script runner and you want to organize it
- You have lots of scripts in various languages
- You want each team to own their own scripts and enforce standards
- You want to enforce `CODEOWNERS` for scripts

## Features

- Task organization by directory
- Concurrent task execution
- Watch mode for development
- Environment variable management
- Task dependencies
- Custom command resolvers
- Task timeouts
- Interactive prompts

## Nice things that will be added

- Custom plugins
- Automatic documentation generation
- Language Server Protocol (LSP) support
- Sandbox mode
- Documentation site
- Automatic migration scripts for migrating from `Makefile`/`package.json`/other script runners
