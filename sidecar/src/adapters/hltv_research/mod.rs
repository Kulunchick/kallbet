//! Ticket #36: research parsers and guarded HLTV historical match collector.
//!
//! Never wired into the live sidecar scheduler or the server crate.
//! The iOS-impersonating `wreq` transport is behind the optional Cargo feature
//! `hltv-impersonate` (`default = []`).

pub mod archive;
pub mod cli;
pub mod collector;
#[cfg(feature = "hltv-impersonate")]
pub mod impersonate;
pub mod liquipedia_maps;
pub mod parse;
pub mod rank_badges;
