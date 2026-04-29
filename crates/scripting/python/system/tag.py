"""Tag functions backed by the OpenWebHMI gateway."""

from . import _bridge


def read(path):
    """Read a tag snapshot from the gateway.

    Returns ``None`` when the tag has no current value. Otherwise returns a
    dictionary with ``path``, ``value``, ``quality``, and ``ts_ms`` keys.
    """

    return _bridge.call("tag.read", {"path": path})


def write(path, value):
    """Write a tag value through the trusted gateway process.

    ``value`` may be a full OpenWebHMI TagValue dictionary such as
    ``{"type": "real", "value": 12.5}``, or a Python bool/int/float/str.
    """

    _bridge.call("tag.write", {"path": path, "value": _tag_value(value)})


def _tag_value(value):
    if isinstance(value, dict) and "type" in value and "value" in value:
        return value
    if isinstance(value, bool):
        return {"type": "bool", "value": value}
    if isinstance(value, int):
        return {"type": "int", "value": value}
    if isinstance(value, float):
        return {"type": "real", "value": value}
    if isinstance(value, str):
        return {"type": "string", "value": value}
    raise TypeError(f"unsupported tag value type: {type(value).__name__}")
