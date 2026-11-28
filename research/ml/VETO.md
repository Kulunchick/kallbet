# How well do the veto simulators predict the actual map vetoes? (W6)

## Primary configuration (declared in a commit before any veto prediction was scored)

Written before any veto step was predicted or scored. Seen so far: only the shape of the data (veto
step patterns per format, record counts, map names) and the code of `features_signals.VetoEB` and the
Rust lab's `bo1_veto_sim`/`bo3_veto_sim`/`bo5_veto_sim`.

- **Data:** the extended archive with W5's recovered HLTV links (`ml-cache-w5/research-archive-ext`,
  9,305 Matches with a 7-step veto; 1,944 of them in scope). A veto is released to the predictor only
  when its Match's known finish is strictly before the evaluated Match's cutoff, so the evaluated
  Match's own veto is never seen. Scored set: the 2,087 in-scope anchor Matches that have a full
  7-step veto whose six acting teams resolve. Event bootstrap: 2000 resamples, seed 33, 90%.
- **What is predicted:** each veto step, given the acting team (known when the veto starts) and the
  realised earlier steps ("next-step" mode: the candidates are the detected active pool minus the maps
  already removed or picked). A second mode, "pre-veto", integrates over the unknown earlier steps
  (exact integration over the 5,040 ordered choices, in torch); it is reported but is not primary.
  The detected pool is the one `ActivePool` finds from released 7-step vetoes (top seven maps of the
  last 14 days, refreshed at most every 7 days). An actual choice outside the detected pool gets
  probability 1e-3 under every method.
- **Contender `eb`** (the empirical-Bayes simulator's choice model, as in `VetoEB`): a team removes
  (picks) map m with weight (number of the team's last 25 released vetoes, any format, in which it
  removed (picked) m + 0.3), normalised over the candidates.
- **Baseline `freq`:** the team's most frequent choice at that step type (format, ban/pick, the team's
  1st/2nd action of that kind) over its last N vetoes of the same format, with additive smoothing α for
  the log-loss. (N, α) is chosen per evaluated Event, walk-forward, from N ∈ {3, 5, 8, 12, 20, 40} ×
  α ∈ {0.25, 1}: the pair with the lowest next-step log-loss over the same-format vetoes of Events that
  ended before the evaluated Event began (all tiers in the extended archive). Other baselines: `uniform`
  (remaining active maps) and `global` (the most frequent choice at that step over the last G vetoes of
  the format, G walk-forward from {100, 300, 1000}).
- **Top-1 tie-break** (all methods): higher global frequency of that map at that step type, then map name.
- **Primary quantity:** top-1 hit rate on **team picks in Bo3** (veto steps 3 and 4), next-step mode,
  detected pool: `eb` minus `freq`, paired per pick step, Event-bootstrap 90% interval of the pooled
  difference.
- **Rule:** `eb` beats the frequency rule only if the interval lies wholly above 0. Otherwise: not shown.
  Everything else (other steps, pre-veto mode, sets, orders, calibration, breakdowns, the lab ports,
  oracle pool, extra variants) is exploratory and labelled so.
- **Expectation, written down so it is not found afterwards:** `eb` only counts a team's own picks over
  its last 25 vetoes (+0.3 smoothing), so at a Bo3 pick it is the frequency rule with N = 25 plus Bo5
  picks. A difference near zero is the expected outcome.
