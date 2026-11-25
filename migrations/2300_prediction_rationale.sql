-- Ticket #23 (spec #21): a Prediction's Rationale.

-- The optional structured explanation a Member (in practice a Bot User)
-- attaches to a Prediction, as JSON (`domain::rationale::Rationale`). It lives
-- on the Prediction's own row, so it is replaced with the pick, follows a
-- Match or team re-key (`UPDATE predictions SET ...` moves the row), and is
-- deleted with the League (`ON DELETE CASCADE`). It names no team id: `p_win`
-- is the win probability of the picked team.
ALTER TABLE predictions ADD COLUMN rationale_json TEXT;
