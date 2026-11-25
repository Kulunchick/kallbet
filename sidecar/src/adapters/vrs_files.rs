//! The VRS Signal Estimator over the configured `vrs_files`, re-scanned while
//! the sidecar runs: a Valve snapshot dropped into a configured directory (by
//! `deploy/sidecar/fetch-vrs.sh` on a schedule) is used without a restart.
use crate::*;
use chrono::Duration;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;

/// How often the configured files and directories are listed again: cheap,
/// and not on every Match.
pub const RESCAN_EVERY: Duration = Duration::minutes(10);

struct State {
    standings: vrs::Standings,
    /// Files already read (or found unreadable): each is read once.
    seen: BTreeSet<PathBuf>,
    next_scan: Instant,
}

pub struct WatchedStandings {
    entries: Vec<PathBuf>,
    audit: Audit,
    clock: Arc<dyn Clock>,
    state: Mutex<State>,
}

impl WatchedStandings {
    /// Reads every configured file now; a file that cannot be read or parsed
    /// fails start-up. Each snapshot is also kept in the audit's
    /// `vrs_snapshots` (where the roster-diff enricher reads it).
    pub async fn load(entries: &[PathBuf], audit: &Audit, clock: Arc<dyn Clock>) -> Result<Self> {
        let watched = Self {
            entries: entries.to_vec(),
            audit: Audit {
                db: audit.db.clone(),
            },
            state: Mutex::new(State {
                standings: vrs::Standings::default(),
                seen: BTreeSet::new(),
                next_scan: clock.now() + RESCAN_EVERY,
            }),
            clock,
        };
        {
            let mut state = watched.state.lock().await;
            for path in vrs_paths(&watched.entries)? {
                watched.read(&mut state, &path).await?;
            }
        }
        Ok(watched)
    }

    async fn read(&self, state: &mut State, path: &Path) -> Result<()> {
        state.seen.insert(path.to_owned());
        let markdown = std::fs::read_to_string(path)?;
        let observed_at = state.standings.add(&markdown)?;
        self.audit.record_vrs(&markdown, observed_at).await
    }

    /// Reads the files that appeared since the last scan. A new file that
    /// cannot be read is logged once and skipped; the Match goes on.
    async fn rescan(&self, state: &mut State) {
        let paths = match vrs_paths(&self.entries) {
            Ok(paths) => paths,
            Err(e) => {
                tracing::warn!(error = %e, "vrs: listing the configured files failed");
                return;
            }
        };
        for path in paths {
            if state.seen.contains(&path) {
                continue;
            }
            match self.read(state, &path).await {
                Ok(()) => tracing::info!(file = %path.display(), "vrs: new snapshot loaded"),
                Err(e) => {
                    tracing::warn!(file = %path.display(), error = %e, "vrs: new file skipped")
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl SignalEstimator for WatchedStandings {
    async fn signal(&self, m: &Match, cutoff: Instant) -> Result<Option<AsOf<Signal>>> {
        let mut state = self.state.lock().await;
        let now = self.clock.now();
        if now >= state.next_scan {
            self.rescan(&mut state).await;
            state.next_scan = now + RESCAN_EVERY;
        }
        state
            .standings
            .estimate(&m.team_a.name, &m.team_b.name, m.format, cutoff)
    }
}

/// `vrs_files` entries are Markdown files or directories of them: a directory
/// contributes its Valve global files (`standings_global_*.md`; not the
/// `.partial` files `fetch-vrs.sh` writes first), so a new Valve snapshot is
/// one more file in a volume, not a config edit.
pub fn vrs_paths(entries: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in entries {
        if entry.is_dir() {
            let mut files = std::fs::read_dir(entry)?
                .map(|f| f.map(|f| f.path()))
                .collect::<std::io::Result<Vec<_>>>()?;
            files.retain(|f| {
                f.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("standings_global_") && n.ends_with(".md"))
            });
            files.sort();
            paths.extend(files);
        } else {
            paths.push(entry.clone());
        }
    }
    Ok(paths)
}
