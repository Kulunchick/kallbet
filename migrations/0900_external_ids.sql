-- Ticket #9: linking fallback Data Sources to canonical ids, and the data
-- quality flags League Admins see (#13).

-- (source, kind, external id) -> canonical id. Filled automatically by the
-- fallback chain ('auto': matched by normalised team name and start time;
-- 'source': the record carried a canonical id itself, e.g. Liquipedia's
-- hltv= field) or by a League Admin ('manual', #13). A manual row is never
-- overwritten by an automatic one. Rows with source 'hltv' and kind 'event'
-- are aliases: an HLTV Stage event id -> its Major's (parent) event id.
CREATE TABLE external_ids (
    source       TEXT    NOT NULL,
    kind         TEXT    NOT NULL CHECK (kind IN ('event', 'match', 'team')),
    external_id  TEXT    NOT NULL,
    canonical_id INTEGER NOT NULL,
    origin       TEXT    NOT NULL CHECK (origin IN ('auto', 'source', 'manual')),
    updated_at   TEXT    NOT NULL,
    PRIMARY KEY (source, kind, external_id)
);

CREATE INDEX external_ids_canonical ON external_ids (kind, canonical_id);

-- Counters for local ids: Matches and teams that no HLTV id is known for get
-- negative ids (-1, -2, ...), so they never collide with HLTV's positive ids.
-- `last` is the last id handed out (0 before the first).
CREATE TABLE local_ids (
    kind TEXT    PRIMARY KEY CHECK (kind IN ('match', 'team')),
    last INTEGER NOT NULL
);

-- Records of a fallback source the chain could not link to canonical data,
-- per Event. Replaced as a whole for (event, source) on every fetch from that
-- source, so a fixed mapping clears its flag on the next fetch.
CREATE TABLE unmatched_records (
    event_id    INTEGER NOT NULL,
    source      TEXT    NOT NULL,
    kind        TEXT    NOT NULL CHECK (kind IN ('event', 'match', 'team')),
    external_id TEXT    NOT NULL,
    description TEXT    NOT NULL,
    seen_at     TEXT    NOT NULL,
    PRIMARY KEY (event_id, source, kind, external_id)
);

-- Two Data Sources reported different winners for one Match. `kept_*` is the
-- higher-priority source, whose result stands; `other_*` the one overruled.
-- A winner of NULL means "finished without a winner" or cancelled (Void).
CREATE TABLE source_conflicts (
    event_id      INTEGER NOT NULL,
    match_id      INTEGER NOT NULL,
    kept_source   TEXT    NOT NULL,
    kept_winner   INTEGER,
    other_source  TEXT    NOT NULL,
    other_winner  INTEGER,
    detected_at   TEXT    NOT NULL,
    PRIMARY KEY (match_id, other_source)
);

CREATE INDEX source_conflicts_event ON source_conflicts (event_id);
