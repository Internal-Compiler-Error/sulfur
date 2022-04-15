CREATE TABLE IF NOT EXISTS http_download
(
    id       INTEGER PRIMARY KEY AUTOINCREMENT,
    uri      TEXT,
    -- I might come to regret this, consider using text to prevent weird parsing errors
    progress REAL,
    path     TEXT
);

CREATE TABLE IF NOT EXISTS sub_http_download
(
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER FOREIGN KEY REFERENCES http_download (id),
    offset    INTEGER,
    uri       TEXT,
    progress  REAL
);
