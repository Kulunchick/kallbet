//! `HybridAnchored`: the market line and the statistical estimate blended in
//! logit space, moved by an LLM adjustment only as far as the rules below allow.
//! Everything here is pure, and nothing a model writes is trusted: the clamp, the
//! factor and evidence check, the tie-break and `override_market` are decided in
//! this file whatever the prompt said.
use crate::*;
use serde_json::{Value, json};

/// Weights of the two logits in `p_base`.
pub const MARKET_WEIGHT: f64 = 0.80;
pub const STAT_WEIGHT: f64 = 0.20;
/// Largest shift, in percentage points, a substantiated adjustment may make.
pub const MAX_SHIFT_PP: f64 = 8.0;
/// Largest shift without a factor, allowed only inside `TIE_BREAK_BAND`.
pub const TIE_BREAK_PP: f64 = 2.0;
/// A Match is a toss-up when `|p_base - 0.5|` is below this.
pub const TIE_BREAK_BAND: f64 = 0.03;
/// The closed list of adjustment factors (the server's `FactorKind`).
pub const FACTOR_KINDS: [&str; 6] = [
    "stand_in",
    "roster_change_30d",
    "igl_or_coach_change",
    "visa_or_travel",
    "illness_or_emergency",
    "technical_or_dq",
];
/// The server's bounds on a Rationale (`src/domain/rationale.rs`).
const SUMMARY_MAX: usize = 1000;
const FACTORS_MAX: usize = 8;
const EVIDENCE_URL_MAX: usize = 512;

fn logit(p: f64) -> f64 {
    let p = p.clamp(1e-9, 1.0 - 1e-9);
    (p / (1.0 - p)).ln()
}

/// The anchored probability that team A wins the series, before any LLM:
/// `0.80` market and `0.20` statistical logit; whichever exists when the other
/// is missing; `None` with neither.
pub fn base(dossier: &Dossier) -> Option<f64> {
    let stat = dossier.signal.as_ref().map(|s| s.series);
    match (dossier.market, stat) {
        (Some(market), Some(stat)) => {
            let blended = MARKET_WEIGHT * logit(market) + STAT_WEIGHT * logit(stat);
            Some(1.0 / (1.0 + (-blended).exp()))
        }
        (Some(p), None) | (None, Some(p)) => Some(p),
        (None, None) => None,
    }
}

/// What is left of an LLM adjustment after validation.
#[derive(Debug, Clone, PartialEq)]
pub struct Accepted {
    /// The shift that is applied, in percentage points for team A.
    pub delta_pp: f64,
    /// The factors that passed, as the server's `{kind, evidence_url}`.
    pub factors: Vec<Value>,
    /// The model asked for a shift and none of it was allowed.
    pub rejected: bool,
}

/// An absolute http(s) URL with a host, without whitespace, of sane length:
/// the server's rule for a Rationale's `evidence_url`.
fn is_evidence_url(url: &str) -> bool {
    url.len() <= EVIDENCE_URL_MAX
        && ["https://", "http://"].iter().any(|scheme| {
            url.strip_prefix(scheme)
                .is_some_and(|rest| !rest.is_empty() && !rest.starts_with('/'))
        })
        && !url.chars().any(|c| c.is_whitespace() || c.is_control())
}

/// Applies the guardrails to `adjustment` for a Match anchored at `p_base`:
/// a factor counts only with a kind from the closed list and an evidence URL;
/// with one, the shift is clamped to ±8 pp; without, to ±2 pp inside the
/// toss-up band and to zero outside it.
pub fn accept(p_base: f64, adjustment: &LlmAdjustment) -> Accepted {
    let mut factors: Vec<Value> = vec![];
    for factor in &adjustment.factors {
        let (Some(kind), Some(url)) = (factor["kind"].as_str(), factor["evidence_url"].as_str())
        else {
            continue;
        };
        if !FACTOR_KINDS.contains(&kind) || !is_evidence_url(url) {
            continue;
        }
        let factor = json!({"kind": kind, "evidence_url": url});
        if !factors.contains(&factor) && factors.len() < FACTORS_MAX {
            factors.push(factor);
        }
    }
    let asked = if adjustment.delta.is_finite() {
        adjustment.delta
    } else {
        0.0
    };
    let limit = if !factors.is_empty() {
        MAX_SHIFT_PP
    } else if (p_base - 0.5).abs() < TIE_BREAK_BAND {
        TIE_BREAK_PP
    } else {
        0.0
    };
    let delta_pp = asked.clamp(-limit, limit);
    Accepted {
        delta_pp,
        factors,
        rejected: asked != 0.0 && delta_pp == 0.0,
    }
}

fn text(en: &str, ru: &str, uk: &str) -> [(&'static str, String); 3] {
    [("en", en.into()), ("ru", ru.into()), ("uk", uk.into())]
}

/// The model's summary, each language kept only when it is usable text.
fn model_summary(summary: &Value) -> Option<Value> {
    let mut kept = serde_json::Map::new();
    for language in ["en", "ru", "uk"] {
        if let Some(text) = summary[language].as_str()
            && !text.trim().is_empty()
        {
            let text: String = text.trim().chars().take(SUMMARY_MAX).collect();
            kept.insert(language.into(), text.into());
        }
    }
    (!kept.is_empty()).then_some(Value::Object(kept))
}

/// The fixed summary used when the model wrote none, or wrote one for a shift
/// that was thrown out (its text would argue for a pick the bot did not make).
#[derive(Clone, Copy, PartialEq)]
enum Note {
    None,
    /// The model asked for a shift without a verified factor; it was discarded.
    Rejected,
    /// No LLM answer was available (every chain failed).
    Unavailable,
}
fn anchored_summary(has_market: bool, note: Note) -> Value {
    let mut parts = if has_market {
        text(
            "Anchored to the de-vigged market line, blended with the Valve Regional Standings estimate.",
            "Оценка привязана к рыночной линии без маржи и смешана с оценкой по рейтингу Valve.",
            "Оцінку прив'язано до ринкової лінії без маржі та змішано з оцінкою за рейтингом Valve.",
        )
    } else {
        text(
            "No market line is available, so the estimate rests on the Valve Regional Standings.",
            "Рыночной линии нет, поэтому оценка основана на рейтинге Valve.",
            "Ринкової лінії немає, тому оцінка ґрунтується на рейтингу Valve.",
        )
    };
    let note = match note {
        Note::None => None,
        Note::Rejected => Some(text(
            " The model proposed an adjustment without a verified factor and evidence; it was discarded.",
            " Модель предложила поправку без подтверждённого фактора и источника; она отклонена.",
            " Модель запропонувала поправку без підтвердженого чинника та джерела; її відхилено.",
        )),
        Note::Unavailable => Some(text(
            " No LLM adjustment was available for this Prediction (the language models did not answer), so it is the anchor alone.",
            " Поправка языковой модели для этого прогноза недоступна (модели не ответили), поэтому это только опорная оценка.",
            " Поправка мовної моделі для цього прогнозу недоступна (моделі не відповіли), тому це лише опорна оцінка.",
        )),
    };
    if let Some(note) = note {
        for (part, (_, note)) in parts.iter_mut().zip(note) {
            part.1.push_str(&note);
        }
    }
    Value::Object(
        parts
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect(),
    )
}

/// The evidence a run could really have read: the URLs in the dossier that was
/// sent to the model and, when the run had live web access, any page on the
/// whitelisted sites. A factor citing anything else is a factor without
/// evidence, however well-formed its URL is.
#[derive(Debug, Clone, PartialEq)]
pub struct Evidence {
    urls: std::collections::BTreeSet<String>,
    web: bool,
}
impl Evidence {
    /// `sent` is the dossier as the model was given it (under an anonymizer, the
    /// anonymized one: its URLs may be placeholders and are compared as they
    /// are). `web`: the run could read `adapters::llm::WEB_DOMAINS`.
    pub fn of(sent: &Dossier, web: bool) -> Self {
        fn collect(value: &Value, urls: &mut std::collections::BTreeSet<String>) {
            match value {
                Value::Array(items) => items.iter().for_each(|v| collect(v, urls)),
                Value::Object(fields) => {
                    for (key, value) in fields {
                        // Where a fact says its source is: `url`, `source_url`, ...
                        // Not the licence of the source or the feed an item came in.
                        let names_source = (key == "url" || key.ends_with("_url"))
                            && key != "license_url"
                            && key != "feed_url";
                        match value.as_str() {
                            Some(url) if names_source && !url.trim().is_empty() => {
                                urls.insert(url.to_owned());
                            }
                            _ => collect(value, urls),
                        }
                    }
                }
                _ => {}
            }
        }
        let mut urls = std::collections::BTreeSet::new();
        for fact in &sent.context {
            collect(fact, &mut urls);
        }
        Self { urls, web }
    }
    /// Whether `url` is one this run could have read.
    pub fn supports(&self, url: &str) -> bool {
        self.urls.contains(url) || (self.web && on_web_whitelist(url))
    }
    /// `answer` (an `LlmAdjustment` as JSON) without the factors whose
    /// `evidence_url` this run could not have read. With none left, the policy
    /// treats the shift as unsubstantiated.
    pub fn filter(&self, mut answer: Value) -> Value {
        if let Some(factors) = answer.get_mut("factors").and_then(Value::as_array_mut) {
            factors.retain(|f| f["evidence_url"].as_str().is_some_and(|u| self.supports(u)));
        }
        answer
    }
}

/// An http(s) URL whose host is one of the whitelisted sites or a subdomain of one.
fn on_web_whitelist(url: &str) -> bool {
    if !is_evidence_url(url) {
        return false;
    }
    let rest = url.split_once("://").map_or("", |(_, rest)| rest);
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    // `https://hltv.org@elsewhere.example/` names another host.
    if authority.contains('@') {
        return false;
    }
    let host = authority
        .split(':')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    crate::adapters::llm::WEB_DOMAINS
        .iter()
        .any(|site| host == *site || host.ends_with(&format!(".{site}")))
}

pub struct HybridAnchored;
impl DecisionPolicy for HybridAnchored {
    /// The anchor alone, and the Rationale says that no LLM adjustment was available.
    fn decide_unadjusted(
        &self,
        bot: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
        _: LlmError,
    ) -> Result<Option<Prediction>> {
        let mut decided = self.decide(bot, dossier, tier, None)?;
        if let Some(p) = &mut decided {
            p.rationale["summary"] = anchored_summary(dossier.market.is_some(), Note::Unavailable);
        }
        Ok(decided)
    }
    fn wants_adjustment(&self, _: &BotProfile) -> bool {
        true
    }
    fn decide(
        &self,
        _: &BotProfile,
        dossier: &Dossier,
        tier: Tier,
        adjustment: Option<LlmAdjustment>,
    ) -> Result<Option<Prediction>> {
        let Some(p_base) = base(dossier) else {
            return Ok(None);
        };
        let accepted = adjustment.as_ref().map(|a| accept(p_base, a));
        let delta = accepted.as_ref().map_or(0.0, |a| a.delta_pp) / 100.0;
        // A shift never pushes the estimate to certainty, and never the wrong way.
        let p_final = if delta == 0.0 {
            p_base
        } else {
            (p_base + delta).clamp(p_base.min(0.01), p_base.max(0.99))
        };
        // Exactly even keeps the side the anchor had (team A when that is even too).
        let picks_a = if p_final == 0.5 {
            p_base >= 0.5
        } else {
            p_final > 0.5
        };
        let (team, p_win) = if picks_a {
            (&dossier.game.team_a, p_final)
        } else {
            (&dossier.game.team_b, 1.0 - p_final)
        };
        let rejected = accepted.as_ref().is_some_and(|a| a.rejected);
        let summary = adjustment
            .as_ref()
            .filter(|_| !rejected)
            .and_then(|a| model_summary(&a.summary))
            .unwrap_or_else(|| {
                let note = if rejected { Note::Rejected } else { Note::None };
                anchored_summary(dossier.market.is_some(), note)
            });
        Ok(Some(Prediction {
            team_id: team.id,
            rationale: json!({
                "p_win": p_win,
                // The market favourite is team A at 50/50, as for MarketOnly.
                "override_market": dossier.market.is_some_and(|m| (m >= 0.5) != picks_a),
                "preliminary": tier == Tier::Draft,
                "summary": summary,
                "factors": accepted.map(|a| a.factors).unwrap_or_default(),
            }),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str =
        "https://liquipedia.net/counterstrike/index.php?title=PGL/2026/Singapore&oldid=3473596";

    fn bot() -> BotProfile {
        BotProfile {
            bot_key: "oracle".into(),
            nickname: "Oracle".into(),
            language: None,
            avatar_url: None,
            invite_links: vec![],
            policy: Policy::HybridAnchored,
            provider_chain: vec![],
        }
    }
    fn dossier(market: Option<f64>, stat: Option<f64>) -> Dossier {
        let team = |id, name: &str| Team {
            id,
            name: name.into(),
        };
        Dossier {
            game: Match {
                event_id: 1,
                id: 7,
                team_a: team(1, "Vitality"),
                team_b: team(2, "MOUZ"),
                format: Format::Bo3,
                scheduled_start: None,
            },
            signal: stat.map(|series| Signal {
                map: series,
                series,
            }),
            market,
            context: vec![],
        }
    }
    fn adjustment(delta: f64, factors: Value) -> LlmAdjustment {
        LlmAdjustment {
            delta,
            factors: factors.as_array().unwrap().clone(),
            summary: json!({"en": "Model view.", "ru": "Мнение модели.", "uk": "Думка моделі."}),
        }
    }
    fn stand_in() -> Value {
        json!([{"kind": "stand_in", "evidence_url": URL}])
    }
    /// (picked team id, its probability, override_market) and the whole Rationale.
    fn decide(d: &Dossier, a: Option<LlmAdjustment>) -> (i64, f64, bool, Value) {
        let pick = HybridAnchored
            .decide(&bot(), d, Tier::PreLock, a)
            .unwrap()
            .unwrap();
        let r = pick.rationale;
        (
            pick.team_id,
            r["p_win"].as_f64().unwrap(),
            r["override_market"].as_bool().unwrap(),
            r,
        )
    }
    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn market_and_stat_blend_80_20_in_logit_space() {
        // logit(0.6) = 0.405465, logit(0.7) = 0.847298; 0.8 and 0.2 of them sum
        // to 0.493832, whose sigmoid is 0.62101.
        let (team, p, against, _) = decide(&dossier(Some(0.60), Some(0.70)), None);
        assert_eq!(team, 1);
        assert!(close(p, 0.62101), "{p}");
        assert!(!against);
        // The same Match seen from the other side picks team B with the same odds.
        let (team, p, _, _) = decide(&dossier(Some(0.40), Some(0.30)), None);
        assert_eq!(team, 2);
        assert!(close(p, 0.62101), "{p}");
    }

    #[test]
    fn without_a_market_line_the_statistical_estimate_stands_alone() {
        let (team, p, against, _) = decide(&dossier(None, Some(0.30)), None);
        assert_eq!((team, against), (2, false));
        assert!(close(p, 0.70));
        let (team, p, _, _) = decide(&dossier(Some(0.64), None), None);
        assert_eq!(team, 1);
        assert!(close(p, 0.64));
        assert!(
            HybridAnchored
                .decide(&bot(), &dossier(None, None), Tier::Draft, None)
                .unwrap()
                .is_none(),
            "nothing to anchor on: no Prediction"
        );
    }

    #[test]
    fn a_substantiated_shift_is_clamped_to_eight_points() {
        let d = dossier(Some(0.60), Some(0.60));
        let (_, p, _, r) = decide(&d, Some(adjustment(-5.0, stand_in())));
        assert!(
            close(p, 0.55),
            "a shift inside the cap applies in full: {p}"
        );
        assert_eq!(r["factors"], stand_in());
        assert_eq!(r["summary"]["en"], "Model view.");
        let (team, p, _, _) = decide(&d, Some(adjustment(20.0, stand_in())));
        assert_eq!(team, 1);
        assert!(close(p, 0.68), "+20 becomes +8: {p}");
        let (team, p, _, _) = decide(&d, Some(adjustment(-45.0, stand_in())));
        assert_eq!(team, 1, "-45 becomes -8, which cannot flip a 60% favourite");
        assert!(close(p, 0.52), "{p}");
    }

    #[test]
    fn a_shift_without_a_valid_factor_and_evidence_is_zeroed() {
        let d = dossier(Some(0.60), Some(0.60));
        for factors in [
            json!([]),
            json!([{"kind": "rumour", "evidence_url": URL}]),
            json!([{"kind": "stand_in", "evidence_url": ""}]),
            json!([{"kind": "stand_in", "evidence_url": "   "}]),
            json!([{"kind": "stand_in", "evidence_url": "see the team's post on X"}]),
            json!([{"kind": "stand_in", "evidence_url": "javascript:alert(1)"}]),
            json!([{"kind": "stand_in"}]),
            json!([{"evidence_url": URL}]),
            json!(["stand_in"]),
        ] {
            let (team, p, against, r) = decide(&d, Some(adjustment(-30.0, factors.clone())));
            assert_eq!(team, 1, "{factors}");
            assert!(close(p, 0.60), "{factors}: {p}");
            assert!(!against);
            assert_eq!(r["factors"], json!([]), "{factors}");
            assert_ne!(
                r["summary"]["en"], "Model view.",
                "a summary arguing for the discarded shift is not shown"
            );
            assert!(r["summary"]["en"].as_str().unwrap().contains("discarded"));
        }
    }

    #[test]
    fn only_valid_factors_reach_the_rationale() {
        let d = dossier(Some(0.60), Some(0.60));
        let mixed = json!([
            {"kind": "gut_feeling", "evidence_url": URL},
            {"kind": "visa_or_travel", "evidence_url": "https://www.hltv.org/news/1", "note": "x"},
            {"kind": "visa_or_travel", "evidence_url": "https://www.hltv.org/news/1"},
            {"kind": "illness_or_emergency", "evidence_url": "ftp://example.org/a"},
        ]);
        let (_, p, _, r) = decide(&d, Some(adjustment(-8.0, mixed)));
        assert!(close(p, 0.52), "one valid factor is enough for the shift");
        assert_eq!(
            r["factors"],
            json!([{"kind": "visa_or_travel", "evidence_url": "https://www.hltv.org/news/1"}])
        );
    }

    #[test]
    fn a_toss_up_may_move_two_points_without_a_factor() {
        // 51/49: inside the 3-point band, so -6 becomes -2 and the pick flips.
        let (team, p, against, r) = decide(
            &dossier(Some(0.51), Some(0.51)),
            Some(adjustment(-6.0, json!([]))),
        );
        assert_eq!(team, 2);
        assert!(close(p, 0.51), "{p}");
        assert!(against, "the market favourite was team A");
        assert_eq!(r["summary"]["en"], "Model view.");
        // 54/46 is outside the band: nothing moves.
        let (team, p, _, _) = decide(
            &dossier(Some(0.54), Some(0.54)),
            Some(adjustment(-6.0, json!([]))),
        );
        assert_eq!(team, 1);
        assert!(close(p, 0.54), "{p}");
        // The band is open at 3 points.
        let (_, p, _, _) = decide(
            &dossier(Some(0.53), Some(0.53)),
            Some(adjustment(-6.0, json!([]))),
        );
        assert!(close(p, 0.53), "{p}");
        let (_, p, _, _) = decide(
            &dossier(Some(0.471), Some(0.471)),
            Some(adjustment(1.5, json!([]))),
        );
        assert!(
            close(p, 0.514),
            "a tie-break inside ±2 applies as asked: {p}"
        );
    }

    #[test]
    fn override_market_is_set_exactly_when_the_pick_is_not_the_market_favourite() {
        // A verified stand-in overturns a 52/48 favourite.
        let (team, p, against, _) = decide(
            &dossier(Some(0.52), Some(0.52)),
            Some(adjustment(-8.0, stand_in())),
        );
        assert_eq!((team, against), (2, true));
        assert!(close(p, 0.56), "{p}");
        // The same shift on a 60% favourite keeps the favourite.
        let (team, _, against, _) = decide(
            &dossier(Some(0.60), Some(0.60)),
            Some(adjustment(-8.0, stand_in())),
        );
        assert_eq!((team, against), (1, false));
        // The statistical estimate alone can carry the pick across the market.
        let (team, _, against, _) = decide(&dossier(Some(0.51), Some(0.30)), None);
        assert_eq!((team, against), (2, true));
        // No market line: there is no favourite to go against.
        let (team, _, against, _) = decide(
            &dossier(None, Some(0.52)),
            Some(adjustment(-8.0, stand_in())),
        );
        assert_eq!((team, against), (2, false));
        // An even market counts team A as its favourite.
        let (team, _, against, _) = decide(&dossier(Some(0.50), Some(0.50)), None);
        assert_eq!((team, against), (1, false));
    }

    #[test]
    fn a_shift_never_reaches_certainty() {
        let (team, p, _, _) = decide(
            &dossier(Some(0.97), Some(0.97)),
            Some(adjustment(8.0, stand_in())),
        );
        assert_eq!(team, 1);
        assert!(close(p, 0.99), "{p}");
    }

    #[test]
    fn the_rationale_summary_falls_back_per_language() {
        let d = dossier(Some(0.60), Some(0.60));
        let mut a = adjustment(0.0, json!([]));
        a.summary = json!({"en": "  ", "ru": "Новостей нет.", "uk": 7});
        let (_, _, _, r) = decide(&d, Some(a));
        assert_eq!(r["summary"], json!({"ru": "Новостей нет."}));
        let mut a = adjustment(0.0, json!([]));
        a.summary = json!({"en": "x".repeat(1500)});
        let (_, _, _, r) = decide(&d, Some(a));
        assert_eq!(r["summary"]["en"].as_str().unwrap().len(), 1000);
        let mut a = adjustment(0.0, json!([]));
        a.summary = json!("no object");
        let (_, _, _, r) = decide(&d, Some(a));
        for language in ["en", "ru", "uk"] {
            assert!(!r["summary"][language].as_str().unwrap().is_empty());
        }
        let draft = HybridAnchored
            .decide(&bot(), &d, Tier::Draft, None)
            .unwrap()
            .unwrap();
        assert_eq!(draft.rationale["preliminary"], true);
        assert_eq!(decide(&d, None).3["preliminary"], false);
    }

    /// A dossier with the context facts ticket #26 emits (shapes as in its tests).
    fn sent() -> Dossier {
        let mut d = dossier(Some(0.60), Some(0.60));
        d.context = vec![
            json!({
                "kind": "liquipedia_roster", "team": "Team Vitality", "has_stand_in": true,
                "source_url": URL,
                "attribution": {"source": "Liquipedia", "license": "CC-BY-SA 3.0",
                    "license_url": "https://creativecommons.org/licenses/by-sa/3.0/"},
                "notes": [{"text": "renamed", "source_url": "https://www.hltv.org/news/1/renamed"}]
            }),
            json!({
                "kind": "news_item", "title": "TYLOO bench Moseyuh",
                "url": "https://www.hltv.org/news/46047/tyloo-bench-moseyuh",
                "feed_url": "https://www.hltv.org/rss/news"
            }),
        ];
        d
    }

    #[test]
    fn evidence_is_a_url_the_dossier_carries() {
        let evidence = Evidence::of(&sent(), false);
        assert!(evidence.supports(URL), "a roster fact's source");
        assert!(evidence.supports("https://www.hltv.org/news/46047/tyloo-bench-moseyuh"));
        assert!(
            evidence.supports("https://www.hltv.org/news/1/renamed"),
            "a note's source"
        );
        // Invented, however plausible, and on a whitelisted site: not without web access.
        assert!(!evidence.supports("https://www.hltv.org/news/99999/vitality-field-stand-in"));
        assert!(!evidence.supports("https://liquipedia.net/counterstrike/Team_Vitality"));
        assert!(
            !evidence.supports(&format!("{URL}0")),
            "a near miss is a miss"
        );
        assert!(!evidence.supports(""));
        // The licence and the feed are in the dossier but are no evidence of anything.
        assert!(!evidence.supports("https://creativecommons.org/licenses/by-sa/3.0/"));
        assert!(!evidence.supports("https://www.hltv.org/rss/news"));
    }

    #[test]
    fn with_live_web_access_a_page_on_a_whitelisted_site_is_evidence_too() {
        let evidence = Evidence::of(&sent(), true);
        for url in [
            "https://www.hltv.org/news/99999/vitality-field-stand-in",
            "https://hltv.org/matches/1",
            "http://liquipedia.net/counterstrike/Team_Vitality",
            "https://LIQUIPEDIA.net:443/counterstrike/Team_Vitality?action=history",
            URL,
        ] {
            assert!(evidence.supports(url), "{url}");
        }
        for url in [
            "https://x.com/TeamVitality/status/1",
            "https://nothltv.org/news/1",
            "https://hltv.org.example.com/news/1",
            "https://hltv.org@example.com/news/1",
            "https://example.com/?next=https://hltv.org/",
            "hltv.org/news/1",
            "ftp://hltv.org/news/1",
        ] {
            assert!(!evidence.supports(url), "{url}");
        }
    }

    #[test]
    fn placeholder_urls_of_an_anonymized_dossier_are_compared_as_sent() {
        let mut d = sent();
        d.context = vec![json!({"kind": "liquipedia_roster", "source_url": "SOURCE_01"})];
        let evidence = Evidence::of(&d, false);
        assert!(evidence.supports("SOURCE_01"));
        assert!(
            !evidence.supports(URL),
            "the real URL was never shown to the model"
        );
    }

    #[test]
    fn a_factor_with_untraceable_evidence_is_a_factor_without_evidence() {
        let d = sent();
        let invented = "https://www.hltv.org/news/99999/vitality-field-stand-in";
        let answer = json!({
            "delta": -8,
            "factors": [
                {"kind": "stand_in", "evidence_url": invented},
                {"kind": "visa_or_travel", "evidence_url": "https://x.com/rumour"}
            ],
            "summary": {"en": "Model view.", "ru": "Мнение модели.", "uk": "Думка моделі."}
        });
        // No web access: neither URL can be traced, so nothing moves and nothing is shown.
        let kept = Evidence::of(&d, false).filter(answer.clone());
        assert_eq!(kept["factors"], json!([]));
        let (team, p, _, r) = decide(&d, Some(serde_json::from_value(kept).unwrap()));
        assert_eq!(team, 1);
        assert!(close(p, 0.60), "zeroed exactly as for a missing URL: {p}");
        assert_eq!(r["factors"], json!([]));
        assert!(r["summary"]["en"].as_str().unwrap().contains("discarded"));
        // The same answer from a run that could read hltv.org keeps that one factor.
        let kept = Evidence::of(&d, true).filter(answer.clone());
        assert_eq!(
            kept["factors"],
            json!([{"kind": "stand_in", "evidence_url": invented}])
        );
        let (_, p, _, r) = decide(&d, Some(serde_json::from_value(kept).unwrap()));
        assert!(close(p, 0.52), "{p}");
        assert_eq!(
            r["factors"],
            json!([{"kind": "stand_in", "evidence_url": invented}])
        );
        // Evidence from the dossier needs no web access.
        let mut cited = answer;
        cited["factors"] = json!([{"kind": "stand_in", "evidence_url": URL}]);
        let kept = Evidence::of(&d, false).filter(cited.clone());
        assert_eq!(kept, cited);
        // In a toss-up the two-point tie-break is all that is left, as without a URL.
        let mut even = sent();
        even.market = Some(0.51);
        even.signal = Some(Signal {
            map: 0.51,
            series: 0.51,
        });
        let kept = Evidence::of(&even, false).filter(json!({
            "delta": -8, "factors": [{"kind": "stand_in", "evidence_url": invented}],
            "summary": {"en": "Model view."}
        }));
        let (team, p, _, _) = decide(&even, Some(serde_json::from_value(kept).unwrap()));
        assert_eq!(team, 2);
        assert!(close(p, 0.51), "{p}");
    }
}
