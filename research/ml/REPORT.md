# Can a learned model pick more CS2 Match winners than the anchor? (phase 2: HLTV features)

## Primary configuration (declared before its evaluation)

Written and committed before any model was evaluated on the HLTV features. Phase 1's baseline
table (`ml-notes/ml-core.md`) had been seen, on these same Matches: logreg was the most accurate
there, which informed the choice.

- **Model:** `logreg` (L2 logistic regression, C chosen by inner walk-forward on training Events
  only, both team orientations).
- **Features:** `full` = phase 1's `baseline` groups plus every HLTV group: `rank`, `opp_form`,
  `past_form`, `hltv_h2h`, `players`, `map_pool`, `sides`, `standin`, `roster`, `stakes`.
- **Data:** research archive snapshot 2; evaluated on the 2087 in-scope Matches with an anchor
  (tier S/A, no qualifiers), walk-forward by Event exactly as phase 1.
- **Question and rule:** the difference in accuracy (share of winners picked) model − anchor on the
  same Matches. The model "picks more winners" only if the 90% Event-bootstrap interval (2000
  resamples, seed 33) lies wholly above zero.

Everything else below (the other models, the ablations, log-loss, importance) is exploratory: many
comparisons are made, and some will look good by chance.
