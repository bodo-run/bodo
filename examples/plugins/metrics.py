import time
from typing import Dict, Any

_start_times: Dict[str, float] = {}

def onBeforeTaskRun(opts: Dict[str, Any]) -> None:
    task_name = opts['taskName']
    _start_times[task_name] = time.time()
    print(f"[Python Metrics] Starting to measure task: {task_name}")

def onAfterTaskRun(opts: Dict[str, Any]) -> Dict[str, Any]:
    task_name = opts['taskName']
    if task_name in _start_times:
        duration = time.time() - _start_times[task_name]
        del _start_times[task_name]
        print(f"[Python Metrics] Task {task_name} took {duration:.2f} seconds")
        return {
            "task": task_name,
            "duration": duration,
            "status": opts.get("status", 0)
        }
    return None

def onError(opts: Dict[str, Any]) -> None:
    task_name = opts['taskName']
    if task_name in _start_times:
        duration = time.time() - _start_times[task_name]
        del _start_times[task_name]
        print(f"[Python Metrics] Task {task_name} failed after {duration:.2f} seconds") 