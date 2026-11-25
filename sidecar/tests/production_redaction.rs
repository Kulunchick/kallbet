//! No credential reaches the log or a `Debug` print: not the shared Bot secret,
//! not a Bearer token, not an API key in a URL, not an Invite Link.
use kallbet_ai::{
    BotProfile, Config, Policy,
    adapters::{
        http::HttpPlatform,
        production::{
            Redactor, Secret, logging,
            redact::Sink,
            settings::{LogFormat, Settings},
        },
    },
};
use std::sync::{Arc, Mutex};

const BOT_SECRET: &str = "s3cr3t-bot-secret-0123456789abcdef";
const API_KEY: &str = "oddspapi-key-ABCDEF123456";
const TOKEN: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f00f1e2d3c4b5a69788796a5b4c3d2e1f0";
const INVITE: &str = "https://kallbet.example.com/join/INVITEcode_0123456789";

#[derive(Clone, Default)]
struct Buffer(Arc<Mutex<Vec<u8>>>);
impl std::io::Write for Buffer {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl Buffer {
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
}
fn logged(redactor: Redactor, format: LogFormat, emit: impl FnOnce()) -> String {
    let buffer = Buffer::default();
    let sink: Sink = Arc::new(Mutex::new(buffer.clone()));
    let subscriber = logging::subscriber(redactor, sink, "info", format).unwrap();
    tracing::subscriber::with_default(subscriber, emit);
    buffer.text()
}
fn configured() -> Redactor {
    let redactor = Redactor::default();
    redactor.learn(BOT_SECRET);
    redactor.learn_environment([
        ("ODDSPAPI_API_KEY".into(), API_KEY.into()),
        ("HOME".into(), "/home/someone-long-enough".into()),
    ]);
    redactor.learn_invites([&INVITE.to_string()]);
    redactor
}

#[test]
fn known_secrets_are_removed_wherever_they_appear_in_a_line() {
    let text = logged(configured(), LogFormat::Text, || {
        tracing::error!(error = %format!("login failed with {BOT_SECRET} for key {API_KEY}"), "tick failed");
    });
    assert!(text.contains("tick failed"), "{text}");
    for secret in [BOT_SECRET, API_KEY] {
        assert!(!text.contains(secret), "{secret} leaked: {text}");
    }
    assert!(text.contains("[REDACTED]"));
}

#[test]
fn only_variables_named_like_credentials_are_learned() {
    let text = configured().redact("running in /home/someone-long-enough");
    assert_eq!(text, "running in /home/someone-long-enough");
}

#[test]
fn credentials_in_urls_and_headers_are_removed_even_when_never_learned() {
    let redactor = Redactor::default();
    let cases = [
        (
            "error sending request for url (https://api.oddspapi.io/v4/fixtures?apiKey=abc123XYZ&tournamentId=400014)",
            "abc123XYZ",
        ),
        ("sent Authorization: Bearer abcDEF123456.tail", "abcDEF123456"),
        ("headers {\"x-api-key\": \"sk-ant-1234567890abcdefghij\"}", "1234567890abcdef"),
        ("GET https://kallbet.example.com/join/INVITEcode_0123456789", "INVITEcode_0123456789"),
        ("token=deadbeefdeadbeef&x=1", "deadbeefdeadbeef"),
    ];
    for (line, secret) in cases {
        let out = redactor.redact(line);
        assert!(!out.contains(secret), "{line} -> {out}");
        assert!(out.contains("[REDACTED]"), "{line} -> {out}");
    }
    // Ordinary text is left alone.
    let plain = "tick ok submitted=2 elapsed_ms=40 sort_key=3 monkey=1";
    assert_eq!(redactor.redact(plain), plain);
}

#[test]
fn json_logs_are_redacted_too() {
    let text = logged(configured(), LogFormat::Json, || {
        tracing::warn!(url = %format!("{INVITE}?apiKey={API_KEY}"), "join failed");
    });
    assert!(text.trim_start().starts_with('{'), "{text}");
    assert!(text.contains("join failed"));
    for secret in [API_KEY, "INVITEcode_0123456789"] {
        assert!(!text.contains(secret), "{secret} leaked: {text}");
    }
}

#[test]
fn a_secret_that_contains_another_is_removed_whole() {
    let redactor = Redactor::default();
    redactor.learn("abcdefgh-short");
    redactor.learn("abcdefgh-short-and-longer-secret");
    assert_eq!(redactor.redact("x abcdefgh-short-and-longer-secret y"), "x [REDACTED] y");
}

fn profile() -> BotProfile {
    BotProfile {
        bot_key: "oracle".into(),
        nickname: "Oracle".into(),
        language: None,
        avatar_url: None,
        invite_links: vec![INVITE.into()],
        policy: Policy::HybridAnchored,
        provider_chain: vec!["claude".into()],
    }
}

#[test]
fn debug_output_shows_no_secret_and_no_invite_link() {
    let platform = HttpPlatform::new("https://kallbet.example.com", BOT_SECRET.into()).unwrap();
    let mut shown = format!("{platform:?}");
    shown += &format!("{:?}", Secret::new(TOKEN));
    shown += &format!("{:?}", profile());
    let settings = Settings::from_lookup(&|name| {
        match name {
            "SIDECAR_CONFIG" => Some("sidecar.json".to_string()),
            "KALLBET_BOT_SECRET" => Some(BOT_SECRET.to_string()),
            _ => None,
        }
    })
    .unwrap();
    shown += &format!("{settings:?}");
    let config = Config {
        platform_url: "https://kallbet.example.com".into(),
        audit_file: "sidecar.db".into(),
        vrs_files: vec![],
        bots: vec![profile()],
        context: Default::default(),
        llm: Default::default(),
        debate: Default::default(),
    };
    shown += &format!("{config:?}");
    for secret in [BOT_SECRET, TOKEN, "INVITEcode_0123456789"] {
        assert!(!shown.contains(secret), "{secret} leaked: {shown}");
    }
    // What is useful stays.
    assert!(shown.contains("oracle") && shown.contains("kallbet.example.com"));
}
