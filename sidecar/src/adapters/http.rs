use crate::*;
use reqwest::{Client, RequestBuilder, Response};
use serde::Deserialize;
use serde_json::json;

pub struct HttpPlatform {
    base: String,
    secret: String,
    client: Client,
}
impl HttpPlatform {
    pub fn new(base: &str, secret: String) -> Result<Self> {
        let _ = rustls::crypto::ring::default_provider().install_default();
        Ok(Self {
            base: base.trim_end_matches('/').into(),
            secret,
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()?,
        })
    }
    fn url(&self, path: &str) -> String {
        format!("{}/api/{path}", self.base)
    }
}
// Invite URLs contain a credential. Strip URLs before propagating transport errors.
async fn send(req: RequestBuilder) -> Result<Response> {
    Ok(req
        .send()
        .await
        .and_then(Response::error_for_status)
        .map_err(reqwest::Error::without_url)?)
}
#[derive(Deserialize)]
struct Auth {
    token: String,
}
#[derive(Deserialize)]
struct League {
    id: i64,
    archived_at: Option<Instant>,
}
#[derive(Deserialize)]
struct Event {
    id: i64,
}
#[derive(Deserialize)]
struct View {
    event: Event,
    stages: Vec<Stage>,
}
#[derive(Deserialize)]
struct Stage {
    groups: Vec<Group>,
}
#[derive(Deserialize)]
struct Group {
    matches: Vec<Card>,
}
#[derive(Deserialize)]
struct Card {
    id: i64,
    team_a: Option<Team>,
    team_b: Option<Team>,
    format: Format,
    scheduled_start: Option<Instant>,
    lock: Lock,
    my_prediction: Option<i64>,
}
#[derive(Deserialize)]
struct Lock {
    can_predict: bool,
}
#[async_trait::async_trait]
impl LeaguePlatform for HttpPlatform {
    async fn login(&self, b: &BotProfile) -> Result<String> {
        let profile = json!({
            "bot_key": b.bot_key,
            "nickname": b.nickname,
            "avatar_url": b.avatar_url,
            "language": b.language.as_deref().unwrap_or("en"),
        });
        let req = self
            .client
            .post(self.url("auth/bot"))
            .bearer_auth(&self.secret)
            .json(&profile);
        Ok(send(req).await?.json::<Auth>().await?.token)
    }
    async fn join(&self, token: &str, invite: &str) -> Result<()> {
        let url = reqwest::Url::parse(invite).map_err(|_| "invalid Invite Link")?;
        let code = url
            .path()
            .strip_prefix("/join/")
            .filter(|s| {
                !s.is_empty()
                    && s.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            })
            .ok_or("invalid Invite Link path")?;
        let res = self
            .client
            .post(self.url(&format!("invites/{code}/join")))
            .bearer_auth(token)
            .send()
            .await
            .map_err(reqwest::Error::without_url)?;
        if res.status() == reqwest::StatusCode::CONFLICT {
            let body = res
                .json::<serde_json::Value>()
                .await
                .map_err(reqwest::Error::without_url)?;
            if body["error"]["code"] == "league_archived" {
                return Ok(());
            }
            return Err("invite conflict".into());
        }
        res.error_for_status()
            .map_err(reqwest::Error::without_url)?;
        Ok(())
    }
    async fn leagues(&self, token: &str) -> Result<Vec<i64>> {
        let leagues = send(self.client.get(self.url("leagues")).bearer_auth(token))
            .await?
            .json::<Vec<League>>()
            .await?;
        Ok(leagues
            .into_iter()
            .filter(|l| l.archived_at.is_none())
            .map(|l| l.id)
            .collect())
    }
    async fn matches(&self, token: &str, league: i64) -> Result<Vec<(Match, bool)>> {
        let view = send(
            self.client
                .get(self.url(&format!("leagues/{league}/matches")))
                .bearer_auth(token),
        )
        .await?
        .json::<View>()
        .await?;
        Ok(view
            .stages
            .into_iter()
            .flat_map(|s| s.groups)
            .flat_map(|g| g.matches)
            .filter(|m| m.lock.can_predict)
            .filter_map(|m| {
                Some((
                    Match {
                        id: m.id,
                        event_id: view.event.id,
                        team_a: m.team_a?,
                        team_b: m.team_b?,
                        format: m.format,
                        scheduled_start: m.scheduled_start,
                    },
                    m.my_prediction.is_some(),
                ))
            })
            .collect())
    }
    async fn submit(&self, token: &str, league: i64, game: i64, pick: &Prediction) -> Result<()> {
        send(
            self.client
                .put(self.url(&format!("leagues/{league}/matches/{game}/prediction")))
                .bearer_auth(token)
                .json(pick),
        )
        .await?;
        Ok(())
    }
}
