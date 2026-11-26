//! The Candidate Signal registry and the point-in-time view a Candidate Signal
//! reads. A new Candidate Signal is one module here: implement
//! `CandidateSignal` (fit on the training Matches the protocol hands it) and
//! `FittedSignal` (predict one Match from its view), then add it to `registry`.
use crate::adapters::debate;
use crate::adapters::backtest::feeds::MarketSource;
use crate::*;
use std::collections::BTreeMap;

pub mod bo1_veto_sim;
pub mod bo3_veto_sim;
pub mod bo5_veto_sim;
pub mod calibration;
pub mod favourite_longshot;
pub mod match_context;
pub mod roster;
pub mod head_to_head;
pub mod learned_weights;
pub mod line_movement;
pub mod market_sources;
pub mod lineup_strength;
pub mod own_rating;
pub mod rank_gap;
pub mod per_map_rating;
pub mod player_form;
pub mod recent_form;
pub mod round_diff_rating;
pub mod veto_sim;

/// What a Candidate Signal may read of one Match: the Match as the pipeline
/// sees it before its cutoff, and the anchor's inputs read through the replay
/// ports at that cutoff (each already checked with `AsOf::at`). No outcome.
#[derive(Clone)]
pub struct MatchView {
    pub game: Match,
    pub context: crate::adapters::backtest::match_context::MatchContext,
    /// The Match cutoff (`backtest::match_cutoff`): every datum must be
    /// observed strictly before it.
    pub cutoff: Instant,
    /// Valve's rating as the Signal Estimator port gives it.
    pub signal: Option<AsOf<Signal>>,
    /// Team A's fair series probability as the Odds Source port gives it.
    pub market: Option<AsOf<f64>>,
    /// All archive Matches finished strictly before `cutoff`, oldest first.
    pub history_before: HistorySlice,
    /// Team A's archive Matches finished strictly before `cutoff`, oldest first.
    pub team_a_history: HistorySlice,
    /// Team B's archive Matches finished strictly before `cutoff`, oldest first.
    pub team_b_history: HistorySlice,
    pub map_history: crate::adapters::backtest::map_history::MapHistory,
    pub veto_history: crate::adapters::backtest::veto_history::VetoHistory,
    /// Every price point of each market source observed before the cutoff
    /// (team A's series win, oldest first), for the sources that have any.
    pub market_history: BTreeMap<MarketSource, Vec<AsOf<f64>>>,
    /// Team A's Liquipedia roster revision strictly before `cutoff`, if known.
    pub team_a_roster: Option<AsOf<TeamRosterFact>>,
    /// Team B's Liquipedia roster revision strictly before `cutoff`, if known.
    pub team_b_roster: Option<AsOf<TeamRosterFact>>,
    /// HLTV appearances strictly before `cutoff`, keyed by `player_key(nickname)`.
    pub player_stats: std::collections::BTreeMap<String, PlayerStatsSlice>,
    /// Both active fives and the roster observations available at this cutoff.
    pub roster: Option<AsOf<roster::RosterFeatures>>,
    /// Both teams' VRS and HLTV ranks known at this cutoff (`backtest::rank_history`).
    pub ranks: crate::adapters::backtest::rank_history::Ranks,
}
impl MatchView {
    /// The view a live bot has (ticket #47): the anchor's inputs from the live
    /// ports and nothing else. The live sidecar has no history, roster, map or
    /// context port, so those are empty; only the `promotion::LIVE` signals,
    /// which read the signal and the market alone, run on it.
    pub fn live(
        game: Match,
        cutoff: Instant,
        signal: Option<AsOf<Signal>>,
        market: Option<AsOf<f64>>,
    ) -> Self {
        Self {
            game,
            context: crate::adapters::backtest::match_context::MatchContext::unknown(),
            cutoff,
            signal,
            market,
            history_before: Default::default(),
            team_a_history: Default::default(),
            team_b_history: Default::default(),
            map_history: Default::default(),
            veto_history: Default::default(),
            market_history: Default::default(),
            team_a_roster: None,
            team_b_roster: None,
            player_stats: Default::default(),
            roster: None,
            ranks: Default::default(),
        }
    }
    /// The price points of `source`, each rechecked against the cutoff.
    pub fn points(&self, source: MarketSource) -> Result<Vec<(Instant, f64)>, TemporalLeakError> {
        self.market_history
            .get(&source)
            .into_iter()
            .flatten()
            .map(|p| Ok((p.observed_at(), *p.at(self.cutoff)?)))
            .collect()
    }
    pub fn signal(&self) -> Result<Option<&Signal>, TemporalLeakError> {
        self.signal.as_ref().map(|s| s.at(self.cutoff)).transpose()
    }
    pub fn market(&self) -> Result<Option<f64>, TemporalLeakError> {
        self.market
            .as_ref()
            .map(|m| m.at(self.cutoff).copied())
            .transpose()
    }
    pub fn history_before(&self) -> Result<Vec<&HistoricalMatch>, TemporalLeakError> {
        self.history_before.at(self.cutoff)
    }
    pub fn team_a_history(&self) -> Result<Vec<&HistoricalMatch>, TemporalLeakError> {
        self.team_a_history.at(self.cutoff)
    }
    pub fn team_b_history(&self) -> Result<Vec<&HistoricalMatch>, TemporalLeakError> {
        self.team_b_history.at(self.cutoff)
    }
    pub fn team_a_roster(&self) -> Result<Option<&TeamRosterFact>, TemporalLeakError> {
        self.team_a_roster
            .as_ref()
            .map(|r| r.at(self.cutoff))
            .transpose()
    }
    pub fn team_b_roster(&self) -> Result<Option<&TeamRosterFact>, TemporalLeakError> {
        self.team_b_roster
            .as_ref()
            .map(|r| r.at(self.cutoff))
            .transpose()
    }
    pub fn player_matches(&self, nickname: &str) -> Result<Vec<&PlayerAppearance>, TemporalLeakError> {
        let Some(slice) = self.player_stats.get(&player_key(nickname)) else {
            return Ok(vec![]);
        };
        slice.at(self.cutoff)
    }
    /// The latest observation among the inputs, to date a prediction made from them.
    pub fn observed_at(&self) -> Option<Instant> {
        let s = self.signal.as_ref().map(AsOf::observed_at);
        let m = self.market.as_ref().map(AsOf::observed_at);
        s.max(m)
    }
    /// The anchor (`debate::base`) with `stat` in place of the Valve rating.
    pub fn anchor_with(&self, stat: Option<Signal>) -> Result<Option<f64>> {
        let dossier = Dossier {
            game: self.game.clone(),
            signal: stat,
            market: self.market()?,
            context: vec![],
        };
        Ok(debate::base(&dossier))
    }
    /// The anchor (`debate::base`) with `market` in place of the quote the Odds Source gave.
    pub fn anchor_with_market(&self, market: Option<f64>) -> Result<Option<f64>> {
        let dossier = Dossier {
            game: self.game.clone(),
            signal: self.signal()?.cloned(),
            market,
            context: vec![],
        };
        Ok(debate::base(&dossier))
    }
    /// The current anchor: the baseline every Candidate Signal is compared with.
    pub fn anchor(&self) -> Result<Option<f64>> {
        self.anchor_with(self.signal()?.cloned())
    }
}

/// A training Match: its view and how it ended.
#[derive(Clone)]
pub struct Observed {
    pub view: MatchView,
    pub team_a_won: bool,
}

/// A named criterion the lab evaluates. Pure: everything it learns comes from
/// `fit`'s training Matches, everything it predicts from one `MatchView`.
pub trait CandidateSignal: Send + Sync {
    /// The registry name (snake case), used by `--signal` and in the report.
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    /// The data it needs, for the `not testable` reason when no Match had it.
    fn needs(&self) -> &'static str;
    fn uses_rosters(&self) -> bool {
        false
    }
    /// Whether this Candidate Signal was added after spec #33's candidate list
    /// was fixed (marked next to its verdict in the report).
    fn added_after_list_fixed(&self) -> bool {
        false
    }
    /// Why the archive cannot test it at all, if it cannot (reported `not testable`).
    fn missing_data(&self, _archive: &crate::adapters::backtest::ReplayArchive) -> Option<String> {
        None
    }
    /// Fits on `training`: every Match of the Events that ended before the
    /// evaluated Event began, and nothing else.
    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>>;
    /// The names of the features it gives the combiner (`FittedSignal::features`),
    /// by default one: `shift`.
    fn feature_names(&self) -> Vec<&'static str> {
        vec!["shift"]
    }
}

/// A fitted Candidate Signal.
pub trait FittedSignal: Send + Sync {
    /// Team A's series probability for `m`, dated by the latest datum it read
    /// (the protocol checks it against the cutoff), or `None` when it has no
    /// prediction for this Match.
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>>;
    /// The fit at full precision, for a live bot to load (ticket #47); `None`
    /// for a Candidate Signal that cannot run live (`promotion::LIVE`).
    fn deployment(&self) -> Result<Option<serde_json::Value>> {
        Ok(None)
    }
    /// What it fitted, for the report's row of the evaluated Event.
    fn parameters(&self) -> Option<serde_json::Value> {
        None
    }
    /// Its features for the combiner, one per `CandidateSignal::feature_names`
    /// (`None`: not available for this Match), each dated by what it read. By
    /// default the shift of its probability from the anchor's in logit space.
    fn features(&self, m: &MatchView) -> Result<Vec<Option<AsOf<f64>>>> {
        use crate::adapters::lab::combiner::logit;
        let (Some(p), Some(anchor)) = (self.predict(m)?, m.anchor()?) else {
            return Ok(vec![None]);
        };
        let shift = logit(*p.at(m.cutoff)?) - logit(anchor);
        Ok(vec![Some(AsOf::new(shift, p.observed_at(), m.cutoff)?)])
    }
    /// Optional unblended series rating probability for direct comparison
    /// against the Valve rating (`m.signal()?.series`), without the market line.
    fn predict_rating(&self, _m: &MatchView) -> Result<Option<AsOf<f64>>> {
        Ok(None)
    }
}

/// Every registered Candidate Signal, in report order.
pub fn registry() -> Vec<std::sync::Arc<dyn CandidateSignal>> {
    let mut signals: Vec<std::sync::Arc<dyn CandidateSignal>> = vec![
        std::sync::Arc::new(calibration::RatingCalibration),
        std::sync::Arc::new(learned_weights::LearnedWeights),
        std::sync::Arc::new(market_sources::pinnacle()),
        std::sync::Arc::new(market_sources::polymarket()),
        std::sync::Arc::new(market_sources::kalshi()),
        std::sync::Arc::new(market_sources::average()),
        std::sync::Arc::new(line_movement::LineMovement),
        std::sync::Arc::new(favourite_longshot::FavouriteLongshot),
        std::sync::Arc::new(own_rating::OwnRating),
        std::sync::Arc::new(recent_form::RecentForm),
        std::sync::Arc::new(head_to_head::HeadToHead),
        std::sync::Arc::new(super::llm::LlmAnswers::unrun()),
        std::sync::Arc::new(lineup_strength::LineupStrength),
        std::sync::Arc::new(player_form::PlayerForm),
        std::sync::Arc::new(per_map_rating::PerMapRating),
        std::sync::Arc::new(bo1_veto_sim::Bo1VetoSim),
        std::sync::Arc::new(bo3_veto_sim::Bo3VetoSim),
        std::sync::Arc::new(bo5_veto_sim::Bo5VetoSim),
        std::sync::Arc::new(veto_sim::VetoSim),
        std::sync::Arc::new(rank_gap::RankGap),
        std::sync::Arc::new(round_diff_rating::RoundDiffRating),
    ];
    signals.extend(roster::candidates());
    signals.extend(match_context::registry());
    signals
}

pub fn by_name(name: &str) -> Option<std::sync::Arc<dyn CandidateSignal>> {
    registry().into_iter().find(|s| s.name() == name)
}
