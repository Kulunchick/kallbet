//! Keeping secrets out of logs and `Debug` output.
//!
//! Two layers. Types that hold a credential wrap it in [`Secret`] (or print
//! nothing of it in their own `Debug`). Everything written to the log then
//! passes through a [`Redactor`], which removes the values of the secrets the
//! process knows about and the shapes credentials take in URLs and headers, so
//! that a library error that quotes a request URL cannot leak a key either.
use regex::Regex;
use std::{
    io::Write,
    sync::{Arc, Mutex},
};
use tracing_subscriber::fmt::MakeWriter;

pub const REDACTED: &str = "[REDACTED]";

/// A credential. Its `Debug` shows nothing of the value.
#[derive(Clone)]
pub struct Secret(String);
impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
    pub fn expose(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret([REDACTED])")
    }
}

/// Values shorter than this are never treated as secrets: replacing "1" or "on"
/// everywhere would wreck the log without protecting anything.
const MIN_SECRET_LEN: usize = 8;

/// Environment variables whose *name* says they hold a credential.
fn looks_secret(name: &str) -> bool {
    let name = name.to_ascii_uppercase();
    ["SECRET", "TOKEN", "KEY", "PASSWORD", "PASSWD", "CREDENTIAL"]
        .iter()
        .any(|word| name.contains(word))
}

#[derive(Clone)]
pub struct Redactor {
    values: Arc<Mutex<Vec<String>>>,
    shapes: Arc<Vec<(Regex, &'static str)>>,
}
impl Default for Redactor {
    fn default() -> Self {
        let shape = |pattern: &str, replacement: &'static str| {
            (Regex::new(pattern).expect("a valid pattern"), replacement)
        };
        Self {
            values: Arc::default(),
            shapes: Arc::new(vec![
                // Authorization: Bearer <token>
                shape(r"(?i)(bearer\s+)[A-Za-z0-9._~+/=\-]{6,}", "${1}[REDACTED]"),
                // ?apiKey=..., &token=..., key=...
                shape(
                    r#"(?i)\b((?:[a-z_\-]*(?:api[_\-]?key|token|secret|password)[a-z_\-]*|key)=)[^&\s"'<>]+"#,
                    "${1}[REDACTED]",
                ),
                // x-api-key: ..., "x-api-key":"..."
                shape(
                    r#"(?i)(x-api-key["']?\s*[:=]\s*["']?)[^\s"',}]+"#,
                    "${1}[REDACTED]",
                ),
                // An Invite Link grants League membership.
                shape(r"/join/[A-Za-z0-9_\-]+", "/join/[REDACTED]"),
                // Provider API keys by their well-known prefix.
                shape(r"\bsk-[A-Za-z0-9_\-]{16,}", "[REDACTED]"),
            ]),
        }
    }
}
impl Redactor {
    /// Remove this exact value wherever it appears from now on.
    pub fn learn(&self, value: &str) {
        if value.len() < MIN_SECRET_LEN {
            return;
        }
        let mut values = self.values.lock().unwrap();
        if !values.iter().any(|known| known == value) {
            values.push(value.to_owned());
            // Longest first, so a secret that contains another is removed whole.
            values.sort_by_key(|v| std::cmp::Reverse(v.len()));
        }
    }
    /// Learn the Invite Link codes of a configuration and every credential in
    /// the environment: variables whose name says secret, token, key or password.
    pub fn learn_environment(&self, env: impl IntoIterator<Item = (String, String)>) {
        for (name, value) in env {
            if looks_secret(&name) {
                self.learn(&value);
            }
        }
    }
    pub fn learn_invites<'a>(&self, links: impl IntoIterator<Item = &'a String>) {
        for link in links {
            if let Some(code) = link.rsplit('/').next() {
                self.learn(code);
            }
        }
    }
    pub fn redact(&self, text: &str) -> String {
        let mut out = text.to_owned();
        for value in self.values.lock().unwrap().iter() {
            if out.contains(value.as_str()) {
                out = out.replace(value.as_str(), REDACTED);
            }
        }
        for (pattern, replacement) in self.shapes.iter() {
            out = pattern.replace_all(&out, *replacement).into_owned();
        }
        out
    }
}

/// Where redacted log lines go.
pub type Sink = Arc<Mutex<dyn Write + Send>>;
pub fn stderr_sink() -> Sink {
    Arc::new(Mutex::new(std::io::stderr()))
}

/// A `MakeWriter` for `tracing_subscriber`: every event is buffered whole,
/// redacted, then written to the sink.
#[derive(Clone)]
pub struct RedactingWriter {
    pub redactor: Redactor,
    pub sink: Sink,
}
pub struct EventWriter {
    redactor: Redactor,
    sink: Sink,
    buffer: Vec<u8>,
}
impl Write for EventWriter {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        let text = String::from_utf8_lossy(&self.buffer).into_owned();
        self.buffer.clear();
        let mut sink = self.sink.lock().unwrap();
        sink.write_all(self.redactor.redact(&text).as_bytes())?;
        sink.flush()
    }
}
impl Drop for EventWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
impl<'a> MakeWriter<'a> for RedactingWriter {
    type Writer = EventWriter;
    fn make_writer(&'a self) -> EventWriter {
        EventWriter {
            redactor: self.redactor.clone(),
            sink: self.sink.clone(),
            buffer: Vec::new(),
        }
    }
}
