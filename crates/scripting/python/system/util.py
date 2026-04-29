"""Utility functions backed by the OpenWebHMI gateway."""

from . import _bridge


def now():
    """Return current gateway time in Unix epoch milliseconds."""

    return _bridge.call("util.now", {})


def log(message):
    """Write an INFO log entry through the gateway with the script id attached."""

    _bridge.call("util.log", {"message": str(message)})
