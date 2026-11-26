//! Candidate Signal `own_rating` (ticket #40, spec #33 story 25): a Glicko-2
//! team rating computed from our own archive of series results, updating after
//! every Match finished strictly before the evaluated Match's cutoff (teams
//! joined by `team_names::team_key`). Its map-level logit scale is calibrated
//! walk-forward per format (falling back to the natural Glicko-2 scale 1.0
//! before `MIN_FIT` rated training Matches) and blended with the market as the
//! anchor does; `predict_rating` also exposes the unblended series probability
//! for direct comparison against the Valve rating.
use super::{CandidateSignal, FittedSignal, MatchView, Observed, calibration::fit_scale};
use crate::team_names::team_key;
use crate::*;
use std::{collections::BTreeMap, f64::consts::PI};

pub const NAME: &str = "own_rating";
/// Fewest rated training Matches before fitting a calibration scale.
pub const MIN_FIT: usize = 30;

/// Standard Glicko-2 defaults.
pub const INITIAL_RATING: f64 = 1500.0;
pub const INITIAL_RD: f64 = 350.0;
pub const INITIAL_VOLATILITY: f64 = 0.06;
pub const TAU: f64 = 0.5;
pub const GLICKO_SCALE: f64 = 173.7178;
/// Continuous rating period length for inactivity RD inflation (days).
pub const PERIOD_DAYS: f64 = 30.0;
const MAX_PHI: f64 = INITIAL_RD / GLICKO_SCALE;
const EPSILON: f64 = 1e-6;

pub fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

pub fn logit(p: f64) -> f64 {
    let p = p.clamp(1e-9, 1.0 - 1e-9);
    (p / (1.0 - p)).ln()
}

/// Information weight of a series format when updating ratings and form.
pub fn format_weight(format: Format) -> f64 {
    match format {
        Format::Bo1 => 1.0,
        Format::Bo3 => 1.5,
        Format::Bo5 => 2.0,
    }
}

fn slot(format: Format) -> usize {
    match format {
        Format::Bo1 => 0,
        Format::Bo3 => 1,
        Format::Bo5 => 2,
    }
}

fn g(phi: f64) -> f64 {
    (1.0 + 3.0 * phi * phi / (PI * PI)).sqrt().recip()
}

fn expected(mu: f64, opp_mu: f64, opp_phi: f64) -> f64 {
    sigmoid(g(opp_phi) * (mu - opp_mu))
}

/// One team's Glicko-2 state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TeamRating {
    /// Skill mean on the Glicko-2 internal scale (`(rating - 1500) / 173.7178`).
    pub mu: f64,
    /// Rating deviation on the Glicko-2 internal scale (`rd / 173.7178`).
    pub phi: f64,
    /// Rating volatility.
    pub sigma: f64,
    /// Number of completed Matches incorporated into this rating.
    pub games: usize,
    /// Finish time of the last Match incorporated.
    pub last_played: Option<Instant>,
}

impl Default for TeamRating {
    fn default() -> Self {
        Self {
            mu: 0.0,
            phi: MAX_PHI,
            sigma: INITIAL_VOLATILITY,
            games: 0,
            last_played: None,
        }
    }
}

impl TeamRating {
    /// Rating on the familiar 1500-centered Glicko scale.
    pub fn rating(&self) -> f64 {
        INITIAL_RATING + self.mu * GLICKO_SCALE
    }

    /// Rating deviation on the Glicko scale.
    pub fn rd(&self) -> f64 {
        self.phi * GLICKO_SCALE
    }

    /// Rating deviation inflated for elapsed inactivity up to `at`.
    pub fn phi_at(&self, at: Instant) -> f64 {
        let Some(last) = self.last_played else {
            return self.phi;
        };
        let days = (at - last).num_seconds().max(0) as f64 / 86_400.0;
        let periods = days / PERIOD_DAYS;
        (self.phi * self.phi + periods * self.sigma * self.sigma)
            .sqrt()
            .min(MAX_PHI)
    }

    fn updated_against(&self, opp: &TeamRating, score: f64, weight: f64, at: Instant) -> Self {
        let phi_i = self.phi_at(at);
        let phi_j = opp.phi_at(at);
        let g_j = g(phi_j);
        let e_ij = expected(self.mu, opp.mu, phi_j);
        let v = (weight * g_j * g_j * e_ij * (1.0 - e_ij))
            .max(1e-12)
            .recip();
        let delta = v * weight * g_j * (score - e_ij);
        let sigma_prime = update_volatility(phi_i, self.sigma, v, delta);
        let phi_star = (phi_i * phi_i + sigma_prime * sigma_prime)
            .sqrt()
            .min(MAX_PHI);
        let phi_prime = (1.0 / (phi_star * phi_star) + 1.0 / v).sqrt().recip();
        let mu_prime = self.mu + phi_prime * phi_prime * weight * g_j * (score - e_ij);
        Self {
            mu: mu_prime,
            phi: phi_prime,
            sigma: sigma_prime,
            games: self.games + 1,
            last_played: Some(at),
        }
    }
}

/// Glickman (2012) Step 5: Illinois root-finder for the updated log-variance.
fn update_volatility(phi: f64, sigma: f64, v: f64, delta: f64) -> f64 {
    let a = (sigma * sigma).ln();
    let phi_sq = phi * phi;
    let delta_sq = delta * delta;
    let tau_sq = TAU * TAU;
    let f = |x: f64| -> f64 {
        let ex = x.exp();
        let denom = phi_sq + v + ex;
        ex * (delta_sq - phi_sq - v - ex) / (2.0 * denom * denom) - (x - a) / tau_sq
    };

    let mut big_a = a;
    let mut big_b = if delta_sq > phi_sq + v {
        (delta_sq - phi_sq - v).ln()
    } else {
        let mut k = 1.0;
        while k <= 100.0 && f(a - k * TAU) < 0.0 {
            k += 1.0;
        }
        a - k * TAU
    };

    let mut fa = f(big_a);
    let mut fb = f(big_b);
    for _ in 0..100 {
        if (big_b - big_a).abs() <= EPSILON || (fb - fa).abs() <= 1e-15 {
            break;
        }
        let c = big_a + (big_a - big_b) * fa / (fb - fa);
        let fc = f(c);
        if fc * fb <= 0.0 {
            big_a = big_b;
            fa = fb;
        } else {
            fa /= 2.0;
        }
        big_b = c;
        fb = fc;
    }
    (big_a / 2.0).exp().clamp(0.01, 0.5)
}

/// Point-in-time Glicko-2 ratings table keyed by `team_names::team_key`.
#[derive(Clone, Debug, Default)]
pub struct Glicko2Table {
    teams: BTreeMap<String, TeamRating>,
}

impl Glicko2Table {
    /// Builds a table by replaying `matches` in the order given (oldest first).
    pub fn from_matches(matches: &[&HistoricalMatch]) -> Self {
        let mut table = Self::default();
        for m in matches {
            table.update_match(m);
        }
        table
    }

    /// Simultaneously updates both teams from one completed Match.
    pub fn update_match(&mut self, m: &HistoricalMatch) {
        let a = self.teams.get(&m.team_a_key).copied().unwrap_or_default();
        let b = self.teams.get(&m.team_b_key).copied().unwrap_or_default();
        let weight = format_weight(m.format);
        let score_a = if m.winner_key == m.team_a_key { 1.0 } else { 0.0 };
        let next_a = a.updated_against(&b, score_a, weight, m.finished_at);
        let next_b = b.updated_against(&a, 1.0 - score_a, weight, m.finished_at);
        self.teams.insert(m.team_a_key.clone(), next_a);
        self.teams.insert(m.team_b_key.clone(), next_b);
    }

    pub fn get(&self, key: &str) -> Option<&TeamRating> {
        self.teams.get(key)
    }

    /// Opponent strength in `(0, 1)` (`0.5` for an unrated team).
    pub fn strength(&self, key: &str) -> f64 {
        self.teams.get(key).map_or(0.5, |r| sigmoid(r.mu))
    }

    /// Raw Glicko-2 skill logit difference `g(sqrt(phi_a^2 + phi_b^2)) * (mu_a - mu_b)`
    /// at `cutoff`. `None` when either team has no prior Match in the table.
    pub fn logit_diff(&self, key_a: &str, key_b: &str, cutoff: Instant) -> Option<f64> {
        let a = self.teams.get(key_a).filter(|r| r.games > 0)?;
        let b = self.teams.get(key_b).filter(|r| r.games > 0)?;
        let phi_a = a.phi_at(cutoff);
        let phi_b = b.phi_at(cutoff);
        let phi_ab = (phi_a * phi_a + phi_b * phi_b).sqrt();
        Some(g(phi_ab) * (a.mu - b.mu))
    }
}

/// Immutable prefix cache of a `Glicko2Table` verified by a checksum over
/// `(match_id, finished_at, winner_key)` so sequential evaluations within an
/// Event reuse earlier work without ever reading stale state.
#[derive(Clone)]
pub(crate) struct PrefixTable {
    table: Glicko2Table,
    len: usize,
    checksum: u64,
}

fn mix_u64(mut hash: u64, v: u64) -> u64 {
    for b in v.to_le_bytes() {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    hash
}

fn step_checksum(mut hash: u64, m: &HistoricalMatch) -> u64 {
    hash = mix_u64(hash, m.match_id as u64);
    hash = mix_u64(hash, m.finished_at.timestamp() as u64);
    hash = mix_u64(hash, u64::from(m.finished_at.timestamp_subsec_nanos()));
    for b in m.winner_key.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    hash
}

fn history_checksum(history: &[&HistoricalMatch]) -> u64 {
    history
        .iter()
        .fold(0xcbf2_9ce4_8422_2325u64, |h, m| step_checksum(h, m))
}

impl PrefixTable {
    pub(crate) fn empty() -> Self {
        Self {
            table: Glicko2Table::default(),
            len: 0,
            checksum: 0xcbf2_9ce4_8422_2325u64,
        }
    }

    /// Advances `self` to match `history`, reusing its prefix whenever
    /// `history[..self.len]` has the identical checksum; rebuilds otherwise.
    pub(crate) fn sync_to(&mut self, history: &[&HistoricalMatch]) {
        if history.len() >= self.len && history_checksum(&history[..self.len]) == self.checksum {
            for m in &history[self.len..] {
                self.table.update_match(m);
                self.checksum = step_checksum(self.checksum, m);
            }
            self.len = history.len();
        } else {
            *self = Self::empty();
            for m in history {
                self.table.update_match(m);
                self.checksum = step_checksum(self.checksum, m);
            }
            self.len = history.len();
        }
    }

    pub(crate) fn table(&self) -> &Glicko2Table {
        &self.table
    }

    /// Pure lookup returning a `Glicko2Table` for `history`, cloning `self`
    /// and replaying only the tail when `self` is a valid prefix.
    pub(crate) fn derive(&self, history: &[&HistoricalMatch]) -> Glicko2Table {
        if history.len() >= self.len && history_checksum(&history[..self.len]) == self.checksum {
            let mut table = self.table.clone();
            for m in &history[self.len..] {
                table.update_match(m);
            }
            table
        } else {
            Glicko2Table::from_matches(history)
        }
    }
}

pub struct OwnRating;

impl CandidateSignal for OwnRating {
    fn name(&self) -> &'static str {
        NAME
    }
    fn description(&self) -> &'static str {
        "Glicko-2 team rating updated after every Match finished before the cutoff (teams joined \
         by team_key), with format-calibrated map scale blended with the market as the anchor does"
    }
    fn needs(&self) -> &'static str {
        "earlier completed Matches for both teams in the results history"
    }
    fn missing_data(&self, archive: &crate::adapters::backtest::ReplayArchive) -> Option<String> {
        let total: usize = archive.stages.iter().map(|s| s.matches.len()).sum();
        (total < 2).then(|| {
            "the archive holds fewer than two completed Matches, so no results history can be built"
                .into()
        })
    }
    fn fit(&self, training: &[Observed]) -> Result<Box<dyn FittedSignal>> {
        let mut prefix = PrefixTable::empty();
        let mut data = Vec::new();
        for o in training {
            let history = o.view.history_before()?;
            prefix.sync_to(&history);
            let ka = team_key(&o.view.game.team_a.name);
            let kb = team_key(&o.view.game.team_b.name);
            if let Some(x) = prefix.table().logit_diff(&ka, &kb, o.view.cutoff) {
                data.push((x, o.view.game.format, o.team_a_won));
            }
        }
        let pooled = (data.len() >= MIN_FIT)
            .then(|| fit_scale(&data))
            .flatten()
            .unwrap_or(1.0);
        let mut scales = [pooled; 3];
        for format in [Format::Bo1, Format::Bo3, Format::Bo5] {
            let rows: Vec<_> = data
                .iter()
                .filter(|r| slot(r.1) == slot(format))
                .copied()
                .collect();
            if rows.len() >= MIN_FIT
                && let Some(s) = fit_scale(&rows)
            {
                scales[slot(format)] = s;
            }
        }
        Ok(Box::new(FittedOwnRating { scales, base: prefix }))
    }
}

pub struct FittedOwnRating {
    pub scales: [f64; 3],
    base: PrefixTable,
}

impl FittedOwnRating {
    fn raw_signal(&self, m: &MatchView) -> Result<Option<(Signal, Instant)>> {
        let history = m.history_before()?;
        let Some(hist_obs) = m.history_before.observed_at() else {
            return Ok(None);
        };
        let table = self.base.derive(&history);
        let ka = team_key(&m.game.team_a.name);
        let kb = team_key(&m.game.team_b.name);
        let Some(x) = table.logit_diff(&ka, &kb, m.cutoff) else {
            return Ok(None);
        };
        let scale = self.scales[slot(m.game.format)];
        let map = sigmoid(scale * x);
        let series = vrs::series_probability(map, m.game.format);
        Ok(Some((Signal { map, series }, hist_obs)))
    }
}

impl FittedSignal for FittedOwnRating {
    fn predict(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        let Some((sig, hist_obs)) = self.raw_signal(m)? else {
            return Ok(None);
        };
        let Some(p) = m.anchor_with(Some(sig))? else {
            return Ok(None);
        };
        let observed = m
            .market
            .as_ref()
            .map_or(hist_obs, |mk| hist_obs.max(mk.observed_at()));
        Ok(Some(AsOf::new(p, observed, m.cutoff)?))
    }

    fn predict_rating(&self, m: &MatchView) -> Result<Option<AsOf<f64>>> {
        let Some((sig, hist_obs)) = self.raw_signal(m)? else {
            return Ok(None);
        };
        Ok(Some(AsOf::new(sig.series, hist_obs, m.cutoff)?))
    }
}
