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
struct RawMatchesView {
    event: Event,
    stages: Vec<RawStageView>,
}
#[derive(Deserialize)]
struct RawStageView {
    ordinal: u8,
    kind: String,
    groups: Vec<serde_json::Value>,
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
    async fn pickem_stages(&self, token: &str, league: i64) -> Result<Vec<StagePickem>> {
        let view = send(
            self.client
                .get(self.url(&format!("leagues/{league}/matches")))
                .bearer_auth(token),
        )
        .await?
        .json::<RawMatchesView>()
        .await?;

        let mut out = Vec::new();
        for stage in view.stages {
            if !matches!(stage.kind.as_str(), "swiss" | "playoffs") {
                continue;
            }
            let res = self
                .client
                .get(self.url(&format!(
                    "leagues/{league}/stages/{}/pickem",
                    stage.ordinal
                )))
                .bearer_auth(token)
                .send()
                .await
                .map_err(reqwest::Error::without_url)?;
            if res.status() == reqwest::StatusCode::NOT_FOUND {
                continue;
            }
            let mut pv = res
                .error_for_status()
                .map_err(reqwest::Error::without_url)?
                .json::<serde_json::Value>()
                .await
                .map_err(reqwest::Error::without_url)?;

            let locks_at: Option<Instant> = serde_json::from_value(pv["lock"]["locks_at"].clone())?;
            let locked = pv["lock"]["locked"].as_bool().unwrap_or(false);
            let can_edit = pv["lock"]["can_edit"].as_bool().unwrap_or(false);
            let my_updated_at: Option<Instant> = if pv["my_pickem"].is_null() {
                None
            } else {
                serde_json::from_value(pv["my_pickem"]["updated_at"].clone())?
            };
            let teams: Vec<Team> = serde_json::from_value(pv["teams"].clone())?;

            let mut leaf_slots = std::collections::HashSet::new();
            if let Some(slots) = pv["bracket"]["slots"].as_array() {
                for s in slots {
                    if s["leaf"].as_bool() == Some(true)
                        && let (Some(r), Some(slot)) =
                            (s["rounds_before_final"].as_u64(), s["slot"].as_u64())
                    {
                        leaf_slots.insert((r as u8, slot as u8));
                    }
                }
            }

            let mut opening_matches = Vec::new();
            let parse_card = |m: &serde_json::Value| -> Option<Match> {
                let card: Card = serde_json::from_value(m.clone()).ok()?;
                Some(Match {
                    event_id: view.event.id,
                    id: card.id,
                    team_a: card.team_a?,
                    team_b: card.team_b?,
                    format: card.format,
                    scheduled_start: card.scheduled_start,
                })
            };

            if stage.kind == "swiss" {
                let r1_groups: Vec<&serde_json::Value> = stage
                    .groups
                    .iter()
                    .filter(|g| {
                        g["kind"].as_str() == Some("record")
                            && g["wins"].as_u64() == Some(0)
                            && g["losses"].as_u64() == Some(0)
                    })
                    .collect();
                let source_groups: Vec<&serde_json::Value> = if r1_groups.is_empty() {
                    stage.groups.iter().collect()
                } else {
                    r1_groups
                };
                for g in source_groups {
                    if let Some(ms) = g["matches"].as_array() {
                        opening_matches.extend(ms.iter().filter_map(parse_card));
                    }
                }
            } else {
                for g in &stage.groups {
                    if g["kind"].as_str() == Some("playoff_round")
                        && let Some(r) = g["rounds_before_final"].as_u64()
                        && let Some(ms) = g["matches"].as_array()
                    {
                        for m in ms {
                            if let Some(s) = m["bracket_slot"].as_u64()
                                && leaf_slots.contains(&(r as u8, s as u8))
                                && let Some(parsed) = parse_card(m)
                            {
                                opening_matches.push(parsed);
                            }
                        }
                    }
                }
            }

            if let Some(obj) = pv.as_object_mut() {
                obj.insert("groups".into(), serde_json::Value::Array(stage.groups));
            }

            out.push(StagePickem {
                event_id: view.event.id,
                ordinal: stage.ordinal,
                locks_at,
                locked,
                can_edit,
                my_updated_at,
                teams,
                opening_matches,
                stage: pv,
            });
        }
        Ok(out)
    }
    async fn submit_pickem(
        &self,
        token: &str,
        league: i64,
        stage: u8,
        payload: &serde_json::Value,
    ) -> Result<bool> {
        let res = self
            .client
            .put(self.url(&format!("leagues/{league}/stages/{stage}/pickem")))
            .bearer_auth(token)
            .json(payload)
            .send()
            .await
            .map_err(reqwest::Error::without_url)?;
        if res.status() == reqwest::StatusCode::CONFLICT {
            let body = res
                .json::<serde_json::Value>()
                .await
                .map_err(reqwest::Error::without_url)?;
            if matches!(
                body["error"]["code"].as_str(),
                Some("pickem_locked" | "league_archived")
            ) {
                return Ok(false);
            }
            return Err("pickem conflict".into());
        }
        res.error_for_status()
            .map_err(reqwest::Error::without_url)?;
        Ok(true)
    }
}
