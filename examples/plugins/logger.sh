#!/bin/bash

# Bodo plugins can be written in any language.
# Bodo natively supports Rust, TypeScript, Python, Ruby. For other languages,
# you can write a plugin in bash and call your own code

#
# In bodo's bash plugins:
# 1. Arguments are passed as positional parameters ($1, $2, etc.)
# 2. JSON objects are passed as stringified JSON
# 3. Return values are handled through stdout
# 4. Logs/errors should be written to stderr using echo >&2
# 5. Each function must be exported using export -f for bodo to call it

# Hook: on_before_task_run
# Arguments:
#   $1 (task_name): String - Name of the task being run
#   $2 (cwd): String - Current working directory
# Return: None (void hook)
on_before_task_run() {
    local task_name="$1"
    local cwd="$2"
    # Use echo >&2 for logging since this hook doesn't return a value
    echo "[Bash Logger] Starting task: $task_name in $cwd" >&2
}

# Hook: on_after_task_run
# Arguments:
#   $1 (task_name): String - Name of the task that completed
#   $2 (status): Number - Exit status of the task
# Return: None (void hook)
on_after_task_run() {
    local task_name="$1"
    local status="$2"
    echo "[Bash Logger] Task $task_name finished with status $status" >&2
}

# Hook: on_error
# Arguments:
#   $1 (task_name): String - Name of the task that failed
#   $2 (error): String - Error message
# Return: None (void hook)
on_error() {
    local task_name="$1"
    local error="$2"
    echo "[Bash Logger] Task $task_name failed: $error" >&2
}

# Hook: on_resolve_command
# Arguments:
#   $1 (task_json): String - JSON string containing task configuration
# Return: Modified task configuration as JSON string
# Example input JSON:
#   {"command": "npm test", "env": {"NODE_ENV": "test"}}
on_resolve_command() {
    local task_json="$1"
    # For hooks that need to return values:
    # 1. Use echo (without >&2) to return the value
    # 2. The value must be in the format expected by bodo
    # Here we modify the task by adding DEBUG=1 to env and return the modified JSON
    echo "$task_json" | jq '.env.DEBUG = "1"'

    # Log to stderr (won't affect return value)
    echo "[Bash Logger] Modified task configuration with DEBUG=1" >&2
}

# Hook: on_command_ready
# Arguments:
#   $1 (command): String - The command to be executed
#   $2 (task_name): String - Name of the task
# Return: None (void hook)
on_command_ready() {
    local command="$1"
    local task_name="$2"
    echo "[Bash Logger] Executing command '$command' for task '$task_name'" >&2
}

# Hook: on_bodo_exit
# Arguments:
#   $1 (exit_code): Number - The exit code bodo will use
# Return: None (void hook)
on_bodo_exit() {
    local exit_code="$1"
    echo "[Bash Logger] Bodo exiting with code $exit_code" >&2
}

# Example of a hook that processes complex JSON input and output
# This is not a standard hook, just an example
# Arguments:
#   $1 (config_json): String - Complex JSON configuration
# Return: Modified JSON configuration
example_json_processing() {
    local config_json="$1"

    # Parse specific fields from JSON using jq
    local task_name=$(echo "$config_json" | jq -r '.taskName')
    local env_vars=$(echo "$config_json" | jq -r '.env')

    # Log parsed values (to stderr)
    echo "Processing task: $task_name with env: $env_vars" >&2

    # Modify the JSON:
    # 1. Add new environment variable
    # 2. Add timestamp
    # 3. Modify existing fields
    echo "$config_json" | jq '
        .env.TIMESTAMP = (now | tostring) |
        .env.LOGGER = "bash" |
        .modified = true
    '
}

# Export all functions so they can be called by bodo
export -f on_before_task_run
export -f on_after_task_run
export -f on_error
export -f on_resolve_command
export -f on_command_ready
export -f on_bodo_exit
export -f example_json_processing
