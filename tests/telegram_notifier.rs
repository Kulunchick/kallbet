//! The production Telegram adapter, against a local stand-in for the Bot API.
//! No real Telegram message is ever sent: the adapter is pointed at a
//! listener on 127.0.0.1.

use kallbet::domain::{Language, TelegramId, UserId};
use kallbet::ports::{Delivery, Notification, NotificationKind, Notifier, TelegramNotifier};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn notification() -> Notification {
    Notification {
        user_id: UserId(1),
        telegram_id: TelegramId(555),
        language: Language::En,
        kind: NotificationKind::Reminder,
        text: "NAVI vs FaZe locks soon".into(),
    }
}

/// Serves one request with `status` and `body`; returns the base URL and a
/// handle yielding the raw request that arrived.
async fn serve_once(status: u16, body: &str) -> (String, tokio::task::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let body = body.to_string();
    let handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut received = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let n = socket.read(&mut chunk).await.unwrap();
            received.extend_from_slice(&chunk[..n]);
            let text = String::from_utf8_lossy(&received).to_string();
            if let Some((head, rest)) = text.split_once("\r\n\r\n") {
                let length = head
                    .lines()
                    .find_map(|l| {
                        l.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|v| v.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                if rest.len() >= length {
                    break;
                }
            }
            if n == 0 {
                break;
            }
        }
        let reply = format!(
            "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(reply.as_bytes()).await.unwrap();
        String::from_utf8_lossy(&received).to_string()
    });
    (url, handle)
}

#[tokio::test]
async fn sends_the_text_to_the_chat_of_the_user_through_send_message() {
    let (url, request) = serve_once(200, r#"{"ok":true}"#).await;
    let notifier = TelegramNotifier::with_base_url(Some("123:ABC".into()), &url);

    let delivery = notifier.send(notification()).await;

    assert_eq!(delivery, Ok(Delivery::Sent));
    let request = request.await.unwrap();
    assert!(
        request.starts_with("POST /bot123:ABC/sendMessage "),
        "{request}"
    );
    assert!(request.contains(r#""chat_id":555"#), "{request}");
    assert!(request.contains("NAVI vs FaZe locks soon"), "{request}");
}

#[tokio::test]
async fn a_user_who_has_not_started_the_bot_is_undeliverable_not_an_error() {
    let (url, _) = serve_once(
        403,
        r#"{"ok":false,"error_code":403,"description":"Forbidden: bot can't initiate conversation with a user"}"#,
    )
    .await;
    let notifier = TelegramNotifier::with_base_url(Some("123:ABC".into()), &url);

    assert_eq!(
        notifier.send(notification()).await,
        Ok(Delivery::Undeliverable)
    );
}

#[tokio::test]
async fn a_rate_limit_is_a_passing_error() {
    let (url, _) = serve_once(
        429,
        r#"{"ok":false,"error_code":429,"description":"Too Many Requests: retry after 3"}"#,
    )
    .await;
    let notifier = TelegramNotifier::with_base_url(Some("123:ABC".into()), &url);

    assert!(notifier.send(notification()).await.is_err());
}

#[tokio::test]
async fn without_a_token_nothing_is_sent_and_it_is_not_an_error() {
    // Nothing listens here: any attempt to connect would be an error.
    let notifier = TelegramNotifier::with_base_url(None, "http://127.0.0.1:1");

    assert_eq!(
        notifier.send(notification()).await,
        Ok(Delivery::Undeliverable)
    );
}

#[tokio::test]
async fn a_network_failure_does_not_leak_the_bot_token() {
    let notifier = TelegramNotifier::with_base_url(Some("123:SECRET".into()), "http://127.0.0.1:1");

    let error = notifier.send(notification()).await.unwrap_err();

    assert!(!error.to_string().contains("SECRET"), "{error}");
}
