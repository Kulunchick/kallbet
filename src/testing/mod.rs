//! Test support: the fakes for the three ports and the Seam 1 harness.
//!
//! Compiled into the library (integration tests in `tests/` are separate
//! crates and cannot see `#[cfg(test)]` items), but production code never
//! uses it. How to write a Seam 1 test: `docs/agents/codebase.md`.

mod app;
mod clock;
mod fixtures;
mod notifier;
mod source;

pub use app::{TestApp, TestAppBuilder, TestClient, TestResponse};
pub use clock::FakeClock;
pub use fixtures::EventSnapshotBuilder;
pub use notifier::RecordingNotifier;
pub use source::ScriptedSource;

/// The bot token of the test configuration: sign Telegram login payloads
/// with it to exercise the real HMAC verification.
pub const TEST_BOT_TOKEN: &str = "123456:TEST-BOT-TOKEN";
