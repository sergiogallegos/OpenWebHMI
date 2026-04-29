"""OpenWebHMI Python scripting shim.

Scripts import this module to register handlers and call gateway-backed
``system.*`` functions. The shim communicates with the Rust host through the
worker runner using newline-delimited JSON over stdin/stdout.
"""

from . import tag, util

_TAG_CHANGE_HANDLERS = {}


def on_tag_change(path, fn=None):
    """Register a function to run when ``path`` changes.

    Usable as either ``@system.on_tag_change("provider/Tag")`` or
    ``system.on_tag_change("provider/Tag", handler)``. The handler receives one
    dictionary with ``tag_path``, ``value``, ``quality``, and ``ts_ms`` keys.
    """

    def decorator(handler):
        _TAG_CHANGE_HANDLERS[path] = handler
        return handler

    if fn is not None:
        return decorator(fn)
    return decorator


def _get_tag_change_handler(path):
    return _TAG_CHANGE_HANDLERS.get(path)
