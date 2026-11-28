# A Reveal Locks the Match, and a Lock is never undone

Predictions are hidden until every Eligible Member has predicted, or until Lock (Scheduled Start − 15 min). If Members could still change their Predictions after an early Reveal, they could copy or counter the others, so a Reveal also Locks the Match for everyone. Scheduled Starts at Majors shift often. The Lock time is recomputed from the latest Scheduled Start, but once a Lock has happened it stays even if the Match is later postponed, because Predictions may already have been seen.

## Consequences

- Pick'em deliberately does not use the "everyone has picked" Reveal, which would Lock a whole Stage early. Pick'em Reveals only at its Lock. (_Superseded in November 2026: a Stage's Pick'em now also Locks and reveals early, once every Member has submitted and the last one's 5-minute final edit window has run out. See "Pick'em" in the amendment below for who counts._)
- Members who join after a Match's Lock are not Eligible Members for it and do not hold up its Reveal.

## Amendment: Bot Users (spec #21)

_Contradicts the rule above for Bot Users only, reopened intentionally._ For human Members nothing changes: an early Reveal happens once every human Eligible Member has predicted (after the 5-minute final edit window) and Locks the Match for all of them for good. A Bot User is not waited for, never opens or holds that window, and may still create or change its own Prediction until the Match's Hard Lock (Scheduled Start − 15 min, actual start, or cancellation). An AI Member needs that late window for closing odds and stand-in news, and friends should not wait on a sidecar that may be offline.

The reason for the original rule still holds, so it is kept by other means: until Hard Lock a Bot User is shown no revealed Prediction at all, even after the humans' early Reveal, so it cannot copy or counter what it may still change. From Hard Lock on (or once the League is archived) a Bot User is Locked like everyone else, and a Hard Lock, like any Lock, is never undone by a later postponement.

### Consequences

- A League with no human Eligible Member has no early Reveal: it stays open until Hard Lock.
- Humans see a Bot User's Prediction at their early Reveal while it can still change, so the Match card marks it as preliminary until Hard Lock.

### Pick'em

The amendment covers a Stage's Pick'em in the same shape. For human Members nothing changes: once every human current Member has submitted, and the last one's 5-minute final edit window has run out, the Stage's Pick'em Locks for all of them and they see each other's. A Bot User is not waited for, never opens or holds that window and is not held back by it, and may still create or replace its own Pick'em until the Stage's Hard Lock: the Stage's first Match starting (a Pick'em has no 15-minute lead). The sidecar needs that for its refresh shortly before the Stage, and friends should not have their Pick'em kept open by a bot that has not submitted.

The original reason is kept in the same way: until the Hard Lock a Bot User is shown no other Member's Pick'em, even after the humans' early Lock, and the Pick'em Leaderboard shows it no Points of that Stage, because Points earned on a result that arrived early would tell it what the others picked. From the Hard Lock on (or once the League is archived) a Bot User is Locked like everyone else, and a stored Hard Lock is never undone by a later postponement.

- A League with no human Member has no early Pick'em Lock: the Stage's Pick'em stays open until its Hard Lock.
- Humans see a Bot User's Pick'em at their early Lock while it can still be replaced, so the API marks it as preliminary until the Hard Lock.

## Amendment: Postponements with Missing Predictions and TBD Matches

The rule keeps a Lock across postponements *because Predictions may already have been seen*. When Data Sources reschedule an `Upcoming` Match (or pair a late round with a stale Scheduled Start that later moves), keeping a Lock permanently when Members could not yet predict punishes everyone for schedule churn without finishing participation:

- **TBD Matches**: Scheduled Start minus 15 minutes applies only once both teams of the Match are known; a TBD Match does not Lock by schedule (though an actual start or cancellation still Locks it).
- **Missing human Predictions**: If a Match is still `Upcoming`, Scheduled Start minus 15 minutes moves back into the future (or disappears, or leaves a team TBD), and at least one human Eligible Member (as of the stored Lock) has not yet predicted (or zero Predictions exist), the stored Lock (`locked_at`, `revealed_at`, `hard_locked_at`) clears so everyone can predict until the new deadline. (Because revealed Picks disappear again until the new Reveal, human Members do not gain the ability to change what was kept hidden unless a postponement actually opened the Match for a latecomer.)
- **Missing Bot User Predictions after humans' early Reveal**: If every human Eligible Member predicted (keeping the humans' Lock and Reveal intact), but a Bot User Eligible Member missed predicting before a scheduled Hard Lock and the `Upcoming` Match is postponed, `hard_locked_at` alone clears so the Bot User can predict until the new Hard Lock (remaining blind to the humans' revealed Picks until then).
- Once every Eligible Member has predicted, or once the Match has started or been cancelled, stored Locks remain permanent across any postponement as before.
