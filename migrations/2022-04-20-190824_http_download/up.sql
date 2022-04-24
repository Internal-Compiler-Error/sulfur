CREATE TABLE IF NOT EXISTS http_download
(
    id       INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    uri      TEXT    NOT NULL,
    -- I might come to regret this, consider using text to prevent weird parsing errors
    progress DOUBLE    NOT NULL,
    path     TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS sub_http_download
(
    id        INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER NOT NULL,
    offset    INTEGER NOT NULL,
    uri       TEXT    NOT NULL,
    progress  DOUBLE    NOT NULL,
    FOREIGN KEY (parent_id) REFERENCES http_download (id) ON DELETE CASCADE
);
