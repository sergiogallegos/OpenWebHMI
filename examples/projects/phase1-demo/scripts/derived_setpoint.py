"""Phase 3 scripting smoke: derive Setpoint from Pressure."""

import system


@system.on_tag_change("rockwell-1/Pressure")
def pressure_changed(tag):
    """Write half-pressure setpoint when Pressure exceeds 100."""

    value = tag["value"]["value"]
    if value > 100:
        system.tag.write("rockwell-1/Setpoint", value * 0.5)
