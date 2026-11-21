-- Review fix A12: the Event list as the Data Sources last listed it, so
-- `GET /api/events` is served from here instead of asking the source on
-- every request. Replaced as a whole by each successful list poll.

CREATE TABLE event_list (
    event_id INTEGER PRIMARY KEY REFERENCES events (id) ON DELETE CASCADE,
    position INTEGER NOT NULL
);
