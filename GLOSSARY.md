# KallBet

A private prediction game for groups of friends: Members pick the winners of CS2 Matches at a pro Event and compete on a Leaderboard. No money is involved.

## Language

### People and groups

**User**:
A person signed in to KallBet (via Telegram), or a Bot User, identified by a site-wide unique Nickname.
_Avoid_: Account, player (a "player" is a CS2 pro)

**Bot User**:
An automated User, authenticated by a bot Bearer token rather than Telegram. It does not hold up the humans' early Reveal, may change its Predictions until Hard Lock, and is shown no revealed Prediction before Hard Lock. A Stage's Pick'em treats it the same way: it does not hold up the humans' early Lock, may replace its Pick'em until the Stage's Hard Lock, and is shown no other Member's Pick'em before it.
_Avoid_: AI player, robot; "bot" alone where the Telegram bot could be meant

**Nickname**:
A User's site-wide unique display name, chosen on the site and changeable at any time.
_Avoid_: Username, login

**League**:
A private group of Users competing on the Predictions for exactly one Event, joined via an Invite Link.
_Avoid_: Room, group, lobby

**Member**:
A User who belongs to a League. A User who leaves or is kicked stops being a Member.
_Avoid_: Participant, player

**League Admin**:
A Member allowed to manage the League: the creator plus any co-admins they appoint.
_Avoid_: Owner, moderator

**Invite Link**:
The secret link that lets a User join a League; a League Admin can reissue it, invalidating the old one.

### Tournament data

**Event**:
A pro CS2 tournament as listed on HLTV (e.g. a Major), identified by its HLTV event id.
_Avoid_: Tournament, cup

**Stage**:
A phase of an Event: a Swiss Stage, a Group Stage or the Playoffs.

**Group Stage**:
Any Stage of a non-Swiss Event that precedes the Playoffs (round-robin or double-elimination groups). Its Matches take Predictions, but it has no Pick'em.

**Swiss Stage**:
A Stage where teams with equal records are paired each round until each has 3 wins (advances) or 3 losses (eliminated).
_Avoid_: Group stage (that is a different Stage kind)

**Playoffs**:
The single-elimination bracket that follows the last Swiss Stage (or Group Stage) of an Event.

**Match**:
A series between two teams within an Event (Bo1, Bo3 or Bo5).
_Avoid_: Game, map (a Map is one game within a Match)

**Scheduled Start**:
The latest known start time of a Match according to the Data Sources; it can move until the Match begins.

**Void Match**:
A Match that counts for nothing: cancelled, or reported without a winner.

### Predicting

**Prediction**:
A Member's choice of the winning team for one Match.
_Avoid_: Bet, stake, wager

**Rationale**:
An optional structured explanation attached to a Prediction, in practice by a Bot User: the estimated win probability of the picked team, a short summary in one or more interface languages, verified adjustment factors with evidence links, and whether the pick goes against the market favourite. It is replaced with the Prediction and shown only with it, so it is hidden until Reveal (from a Bot User, until Hard Lock).
_Avoid_: Explanation, reasoning, analysis (in names and the API; the SPA's text may say "AI reasoning")

**Lock**:
The moment after which no Prediction for a Match can be made or changed: Scheduled Start minus 15 minutes, actual start, or Reveal, whichever comes first. A Lock is never undone. A Bot User is bound by the Hard Lock only.
_Avoid_: Deadline, close

**Hard Lock**:
The Lock that does not come from an early Reveal: Scheduled Start minus 15 minutes, actual Match start, or cancellation, whichever comes first. After it neither human Members nor Bot Users can make or change Predictions. A Stage's Pick'em has one too: the Stage's first Match starting, with no 15-minute lead.
_Avoid_: Final lock, bot deadline

**Reveal**:
The moment Members can see each other's Predictions for a Match: when every human Eligible Member has predicted, or at Lock. A Reveal also Locks the Match for the human Members.

**Eligible Member**:
A Member whose Prediction counts for a Match: one who joined the League before that Match's Lock (for a Bot User, its Hard Lock) and is still a Member. An early Reveal waits for the human ones only.

**Pick'em**:
A separate per-Stage forecast: for a Swiss Stage, two teams to go 3-0, two to go 0-3 and six others to advance; for the Playoffs, the winner of every bracket slot. It Locks when the Stage's first Match starts (its Hard Lock), or earlier for the human Members once every one of them has submitted; from their Lock on they see each other's. A Bot User is bound by the Hard Lock only.
_Avoid_: Bracket challenge

**Pick**:
A single forecast within a Pick'em (e.g. "Team X goes 3-0").

### Scoring

**Point**:
One unit of score: one for each correct Prediction or each correct Pick. Match Points and Pick'em Points are tallied separately.

**Leaderboard**:
The ranking of a League's Members by Match Points, with Playoff Points shown alongside; equal Points share a place.
_Avoid_: Rating, standings

**Playoff Points**:
The Match Points a Member earned on Playoffs Matches.

**Pick'em Leaderboard**:
The separate ranking of a League's Members by Pick'em Points.

### Data

**Data Source**:
An external provider of Event and Match data: HLTV (primary), PandaScore, Liquipedia.
_Avoid_: Provider, feed, API

**Override**:
A manual correction by a League Admin to a Match's time, result, teams or visibility, which takes precedence over every Data Source. An Override applies only within its own League.

### Research

**Signal Lab**:
The sidecar's evaluation of Candidate Signals over completed Matches under one fixed protocol (walk-forward by Event, log-loss against the anchor, an Event-level bootstrap and a written verdict rule), with no network and no LLM provider.
_Avoid_: Experiment, A/B test

**Candidate Signal**:
A named criterion the Signal Lab evaluates: a point-in-time estimate of a Match from data observed before its cutoff and parameters fitted on earlier Events. It reaches a live bot only if its verdict is `promote` and the operator selects it.
_Avoid_: Feature (alone), factor (a Rationale's factor is something else)
