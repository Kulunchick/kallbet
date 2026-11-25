//! The log subscriber: `tracing` to stderr (where `docker logs` reads), in
//! `key=value` text or one JSON object per line, through the redactor.
use super::{
    redact::{RedactingWriter, Redactor, Sink},
    settings::LogFormat,
};
use crate::*;
use tracing::Subscriber;
use tracing_subscriber::EnvFilter;

pub fn subscriber(
    redactor: Redactor,
    sink: Sink,
    filter: &str,
    format: LogFormat,
) -> Result<Box<dyn Subscriber + Send + Sync>> {
    let filter = EnvFilter::try_new(filter)
        .map_err(|e| format!("SIDECAR_LOG is not a valid filter ({e}): try info or debug"))?;
    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .with_writer(RedactingWriter { redactor, sink });
    Ok(match format {
        LogFormat::Text => Box::new(builder.finish()),
        LogFormat::Json => Box::new(builder.json().finish()),
    })
}
