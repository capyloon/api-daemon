
CREATE TABLE IF NOT EXISTS resources
(
    id       TEXT      PRIMARY KEY NOT NULL, -- a uuid to identify resources.
    parent   TEXT      KEY NOT NULL,
    kind     INTEGER   NOT NULL,
    name     TEXT      KEY NOT NULL,
    created  DATETIME  NOT NULL,
    modified DATETIME  NOT NULL,
    scorer   BLOB      NOT NULL, -- bincode encoded representation of the scorer.
-- Enforce unique names under a container.
    UNIQUE(parent , name)
);

CREATE INDEX IF NOT EXISTS idx_resource_modified ON resources(modified);

CREATE TABLE IF NOT EXISTS tags
(
    id  TEXT KEY NOT NULL,
    tag TEXT NOT NULL,
    FOREIGN KEY(id) REFERENCES resources(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS variants
(
    id       TEXT    KEY NOT NULL,
    name     TEXT    NOT NULL,
    mimeType TEXT    NOT NULL,
    size     INTEGER NOT NULL,
    FOREIGN KEY(id) REFERENCES resources(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_tag_name ON tags(tag);

CREATE VIRTUAL TABLE fts USING fts5(id UNINDEXED, variant UNINDEXED, content);
