//! Data Source adapters (ADR-0001). Each adapter implements
//! [`crate::ports::MatchSource`] and is split into a pure parser (Seam 2,
//! tested on saved fixtures) and a thin fetch layer.

pub mod fallback;
pub mod hltv;
pub mod liquipedia;
pub mod pandascore;
