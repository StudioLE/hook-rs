//! Tracing subscriber setup.

use crate::prelude::*;
use std::io::stderr;
use std::time::Instant;
use tracing::Level;
use tracing::dispatcher::DefaultGuard;
use tracing::level_filters::LevelFilter;
use tracing::subscriber::set_default;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::layer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry};

const DEFAULT_LOG_LEVEL: Level = Level::INFO;

#[cfg(test)]
const TEST_LOG_LEVEL: Level = Level::TRACE;

/// Initialize the tracing subscriber for production use, writing to stderr.
///
/// Returns a guard that must be held for the lifetime of the program.
#[must_use]
pub fn init_logger(log_level: Option<Level>) -> DefaultGuard {
    let log_level = log_level.unwrap_or(DEFAULT_LOG_LEVEL);
    let targets = get_targets().with_default(LevelFilter::from_level(log_level));
    let layer = layer()
        .compact()
        .with_writer(stderr)
        .with_target(false)
        .with_timer(ElapsedTime::default())
        .with_filter(targets);
    let registry = Registry::default().with(layer);
    set_default(registry)
}

/// Initialize the tracing subscriber for tests at TRACE level.
///
/// Returns a guard that must be held for the lifetime of the test.
#[must_use]
#[cfg(test)]
pub fn init_test_logger() -> DefaultGuard {
    let targets = get_targets().with_default(LevelFilter::from_level(TEST_LOG_LEVEL));
    let layer = layer()
        .compact()
        .with_test_writer()
        .with_target(false)
        .with_timer(ElapsedTime::default())
        .with_filter(targets);
    let registry = Registry::default().with(layer);
    set_default(registry)
}

/// Per-crate log level overrides.
///
/// `brush_parser` internals (`expansion`, `parse`) are noisy at DEBUG/TRACE,
/// so cap them at INFO.
#[must_use]
fn get_targets() -> Targets {
    Targets::new()
        .with_target("expansion", LevelFilter::INFO)
        .with_target("parse", LevelFilter::INFO)
}

struct ElapsedTime {
    start: Instant,
}

impl Default for ElapsedTime {
    fn default() -> Self {
        ElapsedTime {
            start: Instant::now(),
        }
    }
}

impl FormatTime for ElapsedTime {
    fn format_time(&self, w: &mut Writer<'_>) -> FmtResult {
        let elapsed = self.start.elapsed();
        write!(w, "{:.3}", elapsed.as_secs_f64())
    }
}
