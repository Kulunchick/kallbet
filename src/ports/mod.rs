//! The injected ports: the testing seam between KallBet and the outside
//! world. Production adapters are wired in `crate::runtime`; the fakes live
//! in `crate::testing`.

pub mod clock;
pub mod match_source;
pub mod notifier;

pub use clock::{Clock, SystemClock};
pub use match_source::{MatchSource, SourceError, UnconfiguredSource};
pub use notifier::{LogNotifier, Notification, NotificationKind, Notifier, NotifyError};
