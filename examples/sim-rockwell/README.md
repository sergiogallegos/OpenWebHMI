# sim-rockwell

Minimal EtherNet/IP simulator for OpenWebHMI Phase 1.

This fixture adapts the `rust-ethernet-ip` 0.7.0 `plc_sim` responder and adds the tags needed by OpenWebHMI driver tests. It is not a full CompactLogix emulator.

## Run

```bash
cargo run -p sim-rockwell -- --bind 127.0.0.1 --port 44818 --config examples/sim-rockwell/Sim.toml
```

Port `44818` is the EtherNet/IP standard port. If your OS or sandbox blocks it, use a higher port:

```bash
cargo run -p sim-rockwell -- --port 44819
```

## Built-in tags

| Tag | Type | Behavior |
|---|---|---|
| `Counter` | `DINT` | increments every 100ms |
| `Setpoint` | `REAL` | read-write latch |
| `Pressure` | `REAL` | sine wave, period 60s, amplitude 100, offset 250 |
| `Heartbeat` | `BOOL` | toggles every 1s |

Additional tags can be loaded from `Sim.toml`.

## Supported protocol subset

- EtherNet/IP `RegisterSession`
- EtherNet/IP `SendRRData`
- CIP `Read Tag` (`0x4C`)
- CIP `Write Tag` (`0x4D`)
- CIP `Multiple Service Packet` (`0x0A`) for batch reads used by `subscribe_tag_group`
- Scalar values: `BOOL`, `DINT`, `REAL`, `STRING`

## Not supported

Routing/backplane behavior, connected messaging, UDT metadata, produced/consumed tags, safety I/O, motion, firmware quirks, and full CIP conformance are out of scope for this fixture.
