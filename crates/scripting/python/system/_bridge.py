"""Internal JSON-RPC bridge used by the OpenWebHMI worker harness."""

import itertools
import json
import sys
import threading

_counter = itertools.count(1)
_write_lock = threading.Lock()
_read_lock = threading.Lock()
_pending = {}


def call(method, args=None):
    """Call a gateway RPC method and return its result."""

    request_id = f"req-{next(_counter)}"
    frame = {
        "kind": "rpc",
        "id": request_id,
        "method": method,
        "args": args or {},
    }
    with _write_lock:
        sys.stdout.write(json.dumps(frame, separators=(",", ":")) + "\n")
        sys.stdout.flush()

    while True:
        with _read_lock:
            pending = _pending.pop(request_id, None)
            if pending is not None:
                return _result_from_response(pending)
            line = sys.stdin.readline()
            if line == "":
                raise RuntimeError("host closed stdin while waiting for RPC response")
            response = json.loads(line)
            if response.get("id") != request_id:
                _pending[response.get("id")] = response
                continue
            return _result_from_response(response)


def _result_from_response(response):
    if response.get("kind") == "rpc.result":
        return response.get("result")
    if response.get("kind") == "rpc.error":
        raise RuntimeError(response.get("error", "gateway RPC failed"))
    raise RuntimeError(f"unexpected RPC response: {response!r}")
