//! Test support: the fakes for the three ports and the Seam 1 harness.
//!
//! Compiled into the library (integration tests in `tests/` are separate
//! crates and cannot see `#[cfg(test)]` items), but production code never
//! uses it. How to write a Seam 1 test: `docs/agents/codebase.md`.

mod app;
mod clock;
mod external;
mod fixtures;
mod notifier;
mod source;
mod telegram;

pub use app::{TestApp, TestAppBuilder, TestClient, TestResponse};
pub use clock::FakeClock;
pub use external::ExternalFake;
pub use fixtures::EventSnapshotBuilder;
pub use notifier::RecordingNotifier;
pub use source::ScriptedSource;
pub use telegram::TelegramLogin;

/// The bot token of the test configuration: sign Telegram login payloads
/// with it to exercise the real HMAC verification.
pub const TEST_BOT_TOKEN: &str = "123456:TEST-BOT-TOKEN";

/// The shared bot secret (`KALLBET_BOT_SECRET`) of the test configuration:
/// what a sidecar presents to `POST /api/auth/bot`.
pub const TEST_BOT_SECRET: &str = "test-shared-bot-secret";
