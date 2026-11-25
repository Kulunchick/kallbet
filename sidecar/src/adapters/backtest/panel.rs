//! Frozen prompts and provider answers keyed by the exact rendered prompt.
use super::*;
use crate::adapters::{
    debate::{self, HybridAnchored},
    llm::*,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::{
    Arc,
    atomic::{AtomicI64, Ordering},
};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use tokio::sync::Mutex;

struct Archive {
    file: PathBuf,
    answers: BTreeMap<String, Value>,
    live: bool,
    pub status: Option<String>,
}
impl Archive {
    pub fn load(dir: &Path, live: bool) -> Result<Self> {
        let file = dir.join("panel.json");
        let answers = if file.exists() {
            serde_json::from_slice(&fs::read(&file)?)?
        } else {
            BTreeMap::new()
        };
        Ok(Self {
            file,
            answers,
            live,
            status: None,
        })
    }
    async fn ask(&mut self, cli: Cli, text: String, role: &RolePrompt) -> Result<Option<Value>> {
        if self.status.is_some() {
            return Ok(None);
        }
        let provider = match cli {
            Cli::Claude => "claude",
            Cli::Codex => "codex",
            Cli::Antigravity => "agy",
        };
        let digest = hex::encode(Sha256::digest(format!("{provider}\n{text}").as_bytes()));
        let record = if let Some(prior) = self.answers.get(&digest) {
            prior.clone()
        } else if !self.live
            || self
                .answers
                .values()
                .any(|v| v["error"] == "provider quota exceeded")
        {
            return Ok(None);
        } else {
            let config: CliConfig = serde_json::from_value(json!({"timeout_seconds":180}))?;
            let mut invocation = cli.command(&config, &text, &role.schema, ToolAccess::None);
            if matches!(cli, Cli::Codex) {
                let stdin = invocation
                    .args
                    .pop()
                    .ok_or("missing Codex stdin argument")?;
                for feature in ["shell_tool", "unified_exec", "multi_agent", "apps"] {
                    invocation.args.extend(["--disable".into(), feature.into()]);
                }
                invocation.args.push(stdin);
            }
            let result = match TokioRunner.run(&invocation).await {
                Ok(out) => cli.parse(&out, &role.schema),
                Err(RunError::TimedOut) => Err(LlmError::Timeout),
                Err(RunError::NotFound) => Err(LlmError::NotConfigured),
                Err(_) => Err(LlmError::Transient),
            };
            let record = match result {
                Ok(answer) => json!({"provider":provider,"role":role.role,"answer":answer}),
                Err(e) => json!({"provider":provider,"role":role.role,"error":e.to_string()}),
            };
            self.answers.insert(digest, record.clone());
            fs::write(&self.file, serde_json::to_vec_pretty(&self.answers)?)?;
            record
        };
        if let Some(error) = record["error"].as_str() {
            self.status = Some(format!("{provider} {}: {error}", role.role));
            return Ok(None);
        }
        let answer = &record["answer"];
        if !schema::valid(answer, &role.schema) {
            return Err("invalid recorded answer".into());
        }
        Ok(Some(answer.clone()))
    }
}

struct Leaf {
    archive: Arc<Mutex<Archive>>,
    cli: Cli,
}
#[async_trait::async_trait]
impl LlmRuntime for Leaf {
    async fn evaluate(
        &self,
        _: &BotProfile,
        d: &Dossier,
        tier: Tier,
        role: &RolePrompt,
    ) -> Result<Option<Value>, LlmError> {
        let text = prompt::render(d, tier, role, ToolAccess::None);
        self.archive
            .lock()
            .await
            .ask(self.cli, text, role)
            .await
            .map_err(|_| LlmError::Transient)
    }
}

struct ReplayClock(AtomicI64);
impl Clock for ReplayClock {
    fn now(&self) -> Instant {
        Instant::from_timestamp_millis(self.0.load(Ordering::Relaxed)).expect("replay timestamp")
    }
}

pub(super) struct Panel {
    archive: Arc<Mutex<Archive>>,
    runtime: Arc<dyn LlmRuntime>,
    clock: Arc<ReplayClock>,
}
impl Panel {
    pub async fn load(dir: &Path, live: bool, audit: &Audit) -> Result<Self> {
        let archive = Arc::new(Mutex::new(Archive::load(dir, live)?));
        let clock = Arc::new(ReplayClock(AtomicI64::new(0)));
        let leaves = [
            ("claude", Cli::Claude),
            ("codex", Cli::Codex),
            ("agy", Cli::Antigravity),
        ]
        .map(|(name, cli)| {
            let leaf: Arc<dyn LlmRuntime> = Arc::new(Leaf {
                archive: archive.clone(),
                cli,
            });
            (name.to_string(), leaf)
        });
        let providers = Arc::new(Providers::from_runtimes(
            leaves,
            chrono::Duration::zero(),
            clock.clone(),
        ));
        let config = serde_json::from_value(json!({"bots":{
            "single":{"mode":"never","fast":["claude"]},
            "debate":{"mode":"always","advocate":["claude"],"devil":["codex"],"arbiter":["agy"]}
        }}))?;
        Ok(Self {
            runtime: debate::build(&config, providers, audit)?,
            archive,
            clock,
        })
    }
    pub async fn status(&self) -> Option<String> {
        self.archive.lock().await.status.clone()
    }
    pub async fn predict(
        &self,
        bot: &BotProfile,
        original: &Dossier,
        blind: bool,
        cutoff: Instant,
    ) -> Result<Vec<(&'static str, Option<Prediction>)>> {
        self.clock
            .0
            .store(cutoff.timestamp_millis(), Ordering::Relaxed);
        let masked = blind.then(|| BlindAnonymizer::mask(original));
        let d = masked.as_ref().map(|pair| &pair.0).unwrap_or(original);
        let mut picks = vec![];
        for (key, label) in [("single", "SingleLlm"), ("debate", "AdversarialDebateLlm")] {
            if self.status().await.is_some() {
                break;
            }
            let mut bot = bot.clone();
            bot.bot_key = key.into();
            let result = self
                .runtime
                .evaluate(&bot, d, Tier::PreLock, &RolePrompt::adjustment())
                .await?;
            if let Some(v) = result {
                let mut pick = HybridAnchored.decide(
                    &bot,
                    d,
                    Tier::PreLock,
                    Some(serde_json::from_value(v)?),
                )?;
                if let (Some((_, mapping)), Some(p)) = (&masked, &mut pick) {
                    p.team_id = mapping
                        .winner(p.team_id)
                        .ok_or("unknown anonymous winner")?;
                }
                picks.push((label, pick));
            }
        }
        Ok(picks)
    }
}

impl Panel {
    /// Compare the two subscription strategies on Singapore's Swiss Pick'em
    /// before the first Match. Only the guarded opening view reaches this method.
    pub async fn swiss(&self, opening: &Value) -> Result<Vec<(&'static str, Value)>> {
        let ids: Vec<i64> = opening["teams"]
            .as_array()
            .ok_or("missing Stage teams")?
            .iter()
            .map(|t| t["id"].as_i64().ok_or("invalid Stage team"))
            .collect::<std::result::Result<_, _>>()?;
        let bucket = |n| json!({"type":"array","minItems":n,"maxItems":n,"items":{"type":"integer","enum":ids}});
        let schema = json!({"type":"object","properties":{"three_nil":bucket(2),"zero_three":bucket(2),"advance":bucket(6)},
            "required":["three_nil","zero_three","advance"],"additionalProperties":false});
        let brief_schema = json!({"type":"object","properties":{"brief":{"type":"string"}},"required":["brief"],"additionalProperties":false});
        let mut output = vec![];
        let mut archive = self.archive.lock().await;
        let mut briefs = vec![];
        for (seat, cli, task, brief) in [
            ("single", Cli::Claude, "Choose the ten picks.", false),
            (
                "advocate",
                Cli::Claude,
                "Argue for the solver's picks.",
                true,
            ),
            (
                "devil",
                Cli::Codex,
                "Challenge its most uncertain picks.",
                true,
            ),
            (
                "arbiter",
                Cli::Antigravity,
                "Weigh both briefs and choose the ten picks.",
                false,
            ),
        ] {
            let role = RolePrompt {
                role: format!("swiss-{seat}"),
                instructions: task.into(),
                schema: if brief {
                    brief_schema.clone()
                } else {
                    schema.clone()
                },
                attachments: vec![],
            };
            let text = format!(
                "You are the {seat} for a Swiss Stage Pick'em. {task} Use only this pre-Stage snapshot. No tools or web access. Select distinct teams for 3-0, 0-3 and 3-1/3-2.\n{opening}\nBriefs: {}\nSchema: {}",
                Value::Array(briefs.clone()),
                role.schema
            );
            let Some(answer) = archive.ask(cli, text, &role).await? else {
                break;
            };
            if brief {
                briefs.push(answer);
                continue;
            }
            let mut unique = std::collections::BTreeSet::new();
            for (key, count) in [("three_nil", 2), ("zero_three", 2), ("advance", 6)] {
                let items = answer[key].as_array().ok_or("invalid Swiss pick")?;
                if items.len() != count
                    || !items.iter().all(|id| {
                        id.as_i64()
                            .is_some_and(|id| ids.contains(&id) && unique.insert(id))
                    })
                {
                    return Err("invalid or duplicate Swiss pick".into());
                }
            }
            output.push((
                if seat == "single" {
                    "SingleLlm"
                } else {
                    "AdversarialDebateLlm"
                },
                answer,
            ));
        }
        Ok(output)
    }
}
