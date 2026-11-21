//! Persistence: one file per aggregate, plain async functions that take a
//! `&SqlitePool` (or a transaction). Use the runtime `sqlx::query*` functions,
//! not the compile-time `query!` macros, so the build needs no database.

pub mod events;
pub mod external_ids;
pub mod leagues;
pub mod notification_log;
pub mod overrides;
pub mod pickems;
pub mod predictions;
pub mod sessions;
pub mod users;
