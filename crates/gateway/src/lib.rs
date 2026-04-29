//! OpenWebHMI gateway library surface used by the binary and integration tests.

#![deny(missing_docs)]

/// Minimal Phase 1 project-file loader and driver runner.
pub mod project;
/// Script-driven tag write routing.
pub mod script_writes;
/// WebSocket server and connection handlers.
pub mod server;
/// Phase 0 simulated tag provider.
pub mod sim_provider;
