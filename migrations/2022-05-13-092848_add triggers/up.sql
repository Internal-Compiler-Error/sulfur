PRAGMA foreign_keys = ON;

CREATE TABLE url
(
    full_text TEXT NOT NULL PRIMARY KEY
);


CREATE TABLE file_path
(
    path TEXT NOT NULL PRIMARY KEY
);

CREATE TABLE http_subdownload
(
    id        INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,

    url       TEXT    NOT NULL,

    "offset"  INTEGER NOT NULL,
    total     INTEGER NOT NULL,

    file_path TEXT    NOT NULL,

    FOREIGN KEY (file_path) REFERENCES file_path (path) ON DELETE CASCADE,
    FOREIGN KEY (url) REFERENCES url (full_text) ON DELETE CASCADE
);


-- delete url and file_path if they are not referenced by http_subdownload
CREATE TRIGGER delete_url_and_file_path_not_referenced_by_http_subdownload_trigger
    AFTER DELETE
    ON http_subdownload
BEGIN
    DELETE FROM url WHERE url.full_text NOT IN (SELECT url FROM http_subdownload);
    DELETE FROM file_path WHERE file_path.path NOT IN (SELECT file_path FROM http_subdownload);
END;
