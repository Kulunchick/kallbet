//! Ticket #30: production wiring of the `kallbet-ai` binary.
//!
//! `registry.rs` composes the ports from a configuration file; this module
//! turns that into a long-running process: settings from the environment
//! (`settings`), logs that cannot leak a credential (`redact`, `logging`), the
//! periodic loop with graceful shutdown (`runner`), a heartbeat file for the
//! container health check (`health`), and the subcommands (`cli`).
pub mod cli;
pub mod health;
pub mod logging;
pub mod redact;
pub mod runner;
pub mod settings;

pub use health::{Health, Heartbeat};
pub use redact::{Redactor, Secret};
pub use settings::Settings;
