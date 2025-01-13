#!/bin/bash

# Exit on any error
set -e

# Only show debug output if BODO_VERBOSE is true
if [ "$BODO_VERBOSE" = "true" ]; then
    echo "[DEBUG] Starting bash bridge script" >&2
    echo "[DEBUG] BODO_OPTS=$BODO_OPTS" >&2
    echo "[DEBUG] BODO_PLUGIN_FILE=$BODO_PLUGIN_FILE" >&2
fi

# Check if BODO_OPTS is provided
if [ -z "$BODO_OPTS" ]; then
    echo "No BODO_OPTS provided" >&2
    exit 1
fi

# Check if BODO_PLUGIN_FILE is provided
if [ -z "$BODO_PLUGIN_FILE" ]; then
    echo "No BODO_PLUGIN_FILE provided" >&2
    exit 1
fi

# Parse JSON opts using jq
if ! opts=$(echo "$BODO_OPTS" | jq -r '.'); then
    echo "Failed to parse BODO_OPTS JSON" >&2
    exit 1
fi

# Get hook name and convert from camelCase to snake_case
hook=$(echo "$opts" | jq -r '.hook')
if [ -z "$hook" ]; then
    echo "No hook specified in opts" >&2
    exit 1
fi

if [ "$BODO_VERBOSE" = "true" ]; then
    echo "[DEBUG] Original hook name: $hook" >&2
fi

# Convert hook name using a lookup table
case "$hook" in
"onBeforeTaskRun") hook_fn="on_before_task_run" ;;
"onAfterTaskRun") hook_fn="on_after_task_run" ;;
"onError") hook_fn="on_error" ;;
"onResolveCommand") hook_fn="on_resolve_command" ;;
"onCommandReady") hook_fn="on_command_ready" ;;
"onBodoExit") hook_fn="on_bodo_exit" ;;
*)
    echo "Unknown hook: $hook" >&2
    exit 1
    ;;
esac

if [ "$BODO_VERBOSE" = "true" ]; then
    echo "[DEBUG] Converted hook name: $hook_fn" >&2
fi

# Source the plugin file to get access to its functions
if [ "$BODO_VERBOSE" = "true" ]; then
    echo "[DEBUG] Sourcing plugin file: $BODO_PLUGIN_FILE" >&2
fi
if ! source "$BODO_PLUGIN_FILE"; then
    echo "Failed to source plugin file $BODO_PLUGIN_FILE" >&2
    exit 1
fi

# List available functions if verbose
if [ "$BODO_VERBOSE" = "true" ]; then
    echo "[DEBUG] Available functions:" >&2
    declare -F | grep -v "^declare -f _" >&2
fi

# Check if the function exists
if ! declare -F "$hook_fn" >/dev/null; then
    echo "Plugin does not export a '$hook_fn' function (converted from $hook)" >&2
    exit 1
fi

if [ "$BODO_VERBOSE" = "true" ]; then
    echo "[DEBUG] Found function: $hook_fn" >&2
fi

# Extract arguments based on hook type
case "$hook_fn" in
on_before_task_run)
    task_name=$(echo "$opts" | jq -r '.taskName')
    cwd=$(echo "$opts" | jq -r '.cwd')
    if [ "$BODO_VERBOSE" = "true" ]; then
        echo "[DEBUG] Calling $hook_fn with task_name=$task_name cwd=$cwd" >&2
    fi
    "$hook_fn" "$task_name" "$cwd"
    ;;
on_after_task_run)
    task_name=$(echo "$opts" | jq -r '.taskName')
    status=$(echo "$opts" | jq -r '.status')
    if [ "$BODO_VERBOSE" = "true" ]; then
        echo "[DEBUG] Calling $hook_fn with task_name=$task_name status=$status" >&2
    fi
    "$hook_fn" "$task_name" "$status"
    ;;
on_error)
    task_name=$(echo "$opts" | jq -r '.taskName')
    error=$(echo "$opts" | jq -r '.error')
    if [ "$BODO_VERBOSE" = "true" ]; then
        echo "[DEBUG] Calling $hook_fn with task_name=$task_name error=$error" >&2
    fi
    "$hook_fn" "$task_name" "$error"
    ;;
on_resolve_command)
    task_json=$(echo "$opts" | jq -r '.task')
    if [ "$BODO_VERBOSE" = "true" ]; then
        echo "[DEBUG] Task JSON: $task_json" >&2
        echo "[DEBUG] Calling $hook_fn with task_json=$task_json" >&2
    fi
    result=$("$hook_fn" "$task_json")
    # Validate that the result is valid JSON
    if ! echo "$result" | jq '.' >/dev/null 2>&1; then
        echo "Invalid JSON returned from on_resolve_command: $result" >&2
        exit 5
    fi
    echo "$result"
    ;;
on_command_ready)
    command=$(echo "$opts" | jq -r '.command')
    task_name=$(echo "$opts" | jq -r '.taskName')
    if [ "$BODO_VERBOSE" = "true" ]; then
        echo "[DEBUG] Calling $hook_fn with command=$command task_name=$task_name" >&2
    fi
    "$hook_fn" "$command" "$task_name"
    ;;
on_bodo_exit)
    exit_code=$(echo "$opts" | jq -r '.exitCode')
    if [ "$BODO_VERBOSE" = "true" ]; then
        echo "[DEBUG] Calling $hook_fn with exit_code=$exit_code" >&2
    fi
    "$hook_fn" "$exit_code"
    ;;
*)
    echo "Unknown hook: $hook_fn" >&2
    exit 1
    ;;
esac
