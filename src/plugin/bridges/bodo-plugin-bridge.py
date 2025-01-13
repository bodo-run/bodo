#!/usr/bin/env python3
import os
import json
import sys
import importlib.util
from typing import Any, Dict

def load_plugin(plugin_file: str) -> Any:
    spec = importlib.util.spec_from_file_location("bodo_plugin", plugin_file)
    if not spec or not spec.loader:
        raise ImportError(f"Could not load plugin from {plugin_file}")
    
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def main() -> None:
    # Get environment variables
    opts_json = os.getenv("BODO_OPTS")
    plugin_file = os.getenv("BODO_PLUGIN_FILE")

    if not opts_json or not plugin_file:
        print("Missing required environment variables", file=sys.stderr)
        sys.exit(1)

    # Parse options
    try:
        opts: Dict[str, Any] = json.loads(opts_json)
    except json.JSONDecodeError as e:
        print(f"Failed to parse BODO_OPTS JSON: {e}", file=sys.stderr)
        sys.exit(1)

    # Get hook name
    hook_name = opts.get("hook")
    if not hook_name:
        print("No hook specified in opts", file=sys.stderr)
        sys.exit(1)

    # Load plugin
    try:
        plugin = load_plugin(plugin_file)
    except Exception as e:
        print(f"Failed to load plugin {plugin_file}: {e}", file=sys.stderr)
        sys.exit(1)

    # Get hook function
    hook_fn = getattr(plugin, hook_name, None)
    if not hook_fn or not callable(hook_fn):
        print(f"Plugin does not export a '{hook_name}' function", file=sys.stderr)
        sys.exit(1)

    # Execute hook
    try:
        result = hook_fn(opts)
        if result:
            print(json.dumps(result))
        sys.exit(0)
    except Exception as e:
        print(f"Plugin error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main() 