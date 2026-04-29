"""OpenWebHMI CPython worker harness."""

import argparse
import json
import runpy
import sys
import traceback
from pathlib import Path

PYTHON_ROOT = Path(__file__).resolve().parent
if str(PYTHON_ROOT) not in sys.path:
    sys.path.insert(0, str(PYTHON_ROOT))

import system  # noqa: E402


def emit(frame):
    """Write one newline-delimited JSON frame to stdout."""

    sys.stdout.write(json.dumps(frame, separators=(",", ":")) + "\n")
    sys.stdout.flush()


def main():
    """Load the user script and process host frames."""

    parser = argparse.ArgumentParser()
    parser.add_argument("--script-id", required=True)
    parser.add_argument("script_path")
    args = parser.parse_args()

    script_path = Path(args.script_path)
    if str(script_path.parent) not in sys.path:
        sys.path.insert(0, str(script_path.parent))

    try:
        runpy.run_path(str(script_path), run_name="__openwebhmi_script__")
    except Exception:
        emit(
            {
                "kind": "script.error",
                "message": "script load failed",
                "traceback": traceback.format_exc(),
            }
        )
        raise

    emit({"kind": "ready", "script_id": args.script_id})

    while True:
        line = sys.stdin.readline()
        if line == "":
            return 0
        frame = json.loads(line)
        kind = frame.get("kind")
        if kind == "shutdown":
            return 0
        if kind != "trigger":
            continue
        trigger_id = frame["id"]
        trigger = frame.get("trigger")
        args_obj = frame.get("args", {})
        try:
            if trigger == "on_tag_change":
                handler = system._get_tag_change_handler(args_obj.get("tag_path"))
                if handler is not None:
                    handler(args_obj)
            emit({"kind": "trigger.done", "id": trigger_id})
        except Exception:
            emit(
                {
                    "kind": "trigger.error",
                    "id": trigger_id,
                    "error": traceback.format_exc(),
                }
            )


if __name__ == "__main__":
    raise SystemExit(main())
