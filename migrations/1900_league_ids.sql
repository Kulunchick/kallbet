-- Review fix A4: League ids are never reused.
-- `leagues.id` (0400) is a plain INTEGER PRIMARY KEY, so SQLite hands out
-- MAX(id) + 1 and a deleted newest League's id goes to the next League.
-- Rebuilding the table with AUTOINCREMENT would cascade-delete every child
-- row, so instead the highest id ever used is kept here (one row) and
-- `store::leagues::create` takes the next id from it.

CREATE TABLE league_id_high_water (
    last INTEGER NOT NULL
);

INSERT INTO league_id_high_water (last) SELECT COALESCE(MAX(id), 0) FROM leagues;

CREATE TRIGGER league_id_high_water_on_insert AFTER INSERT ON leagues
BEGIN
    UPDATE league_id_high_water SET last = MAX(last, NEW.id);
END;
