# SQLite as the only database

KallBet serves a few groups of friends from one small VPS, so we use a single SQLite file (via sqlx) rather than Postgres. It needs no extra service, the backup is one `.backup` call, and the load is tiny. If KallBet ever needs several app instances, this has to be revisited.
