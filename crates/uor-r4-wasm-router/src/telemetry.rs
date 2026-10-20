use std::fmt::{self, Write as _};
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Level, Metadata, Subscriber};

/// Install the process-wide, dependency-light tracing subscriber.
///
/// `verbosity` maps to warn, info, debug, and trace for zero, one, two, and
/// three-or-more `-v` flags. Formatting intentionally stays compact and ANSI
/// free so tracing adds no regex, registry, or terminal-color dependencies.
pub fn init(verbosity: u8) {
    let max_level = match verbosity {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };
    let _ = tracing::subscriber::set_global_default(LightSubscriber {
        max_level,
        next_span: AtomicU64::new(1),
    });
}

struct LightSubscriber {
    max_level: Level,
    next_span: AtomicU64,
}

impl Subscriber for LightSubscriber {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        level_rank(metadata.level()) <= level_rank(&self.max_level)
    }

    fn new_span(&self, _attributes: &Attributes<'_>) -> Id {
        Id::from_u64(self.next_span.fetch_add(1, Ordering::Relaxed))
    }

    fn record(&self, _span: &Id, _values: &Record<'_>) {}

    fn record_follows_from(&self, _span: &Id, _follows: &Id) {}

    fn event(&self, event: &Event<'_>) {
        if !self.enabled(event.metadata()) {
            return;
        }
        let metadata = event.metadata();
        let mut fields = FieldVisitor::default();
        event.record(&mut fields);
        eprintln!(
            "{} {}{}",
            metadata.level(),
            metadata.target(),
            fields.output
        );
    }

    fn enter(&self, _span: &Id) {}

    fn exit(&self, _span: &Id) {}
}

fn level_rank(level: &Level) -> u8 {
    match *level {
        Level::ERROR => 1,
        Level::WARN => 2,
        Level::INFO => 3,
        Level::DEBUG => 4,
        Level::TRACE => 5,
    }
}

#[derive(Default)]
struct FieldVisitor {
    output: String,
}

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        let separator = if self.output.is_empty() { ": " } else { ", " };
        let _ = write!(self.output, "{separator}{}={value:?}", field.name());
    }
}
