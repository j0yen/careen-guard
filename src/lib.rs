//! careen-guard library — public API for integration tests.
//!
//! Watches disk SLO and triggers careen-sweep against live target dirs
//! when usage breaches thresholds. Emits ballast-guard-compatible JSON events.

pub mod config;
pub mod disk;
pub mod event;
pub mod guard;
pub mod survey;
pub mod sweep;
