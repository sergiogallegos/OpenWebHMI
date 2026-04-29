# OpenWebHMI Scripting

`openwebhmi-scripting` runs project Python scripts in CPython worker subprocesses. The gateway starts workers with `python3` by default, or with the command in `OPENWEBHMI_PYTHON`.

## Requirements

- Python 3.11+ available on `PATH`.
- Optional Python packages such as `numpy` and `pandas` can be installed into the active environment; v1 does not create a per-project virtualenv.

Workers communicate with the gateway using newline-delimited JSON over stdin/stdout. Scripts import `system` and use the v1 surface:

```python
import system

@system.on_tag_change("rockwell-1/Pressure")
def pressure_changed(tag):
    value = tag["value"]["value"]
    if value > 100:
        system.tag.write("rockwell-1/Setpoint", value * 0.5)
```

`system.tag.write` writes directly through the trusted gateway `TagStore` and emits a structured INFO audit log with `script_id`, tag path, and value.

## Limitations

- Only `on_tag_change` is active in v1. Timer, alarm, and button-click triggers are schema stubs.
- Each trigger has a timeout, defaulting to 5 seconds. A timed-out worker is killed and restarted with exponential backoff.
- POSIX memory/CPU rlimits are not enforced yet; this crate currently relies on the subprocess crash boundary and per-trigger timeout. Windows process resource limits are also not enforced in v1.
