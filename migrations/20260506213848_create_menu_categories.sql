CREATE TABLE IF NOT EXISTS menu_categories (
    id            TEXT     PRIMARY KEY NOT NULL,
    name          TEXT     NOT NULL UNIQUE,
    description   TEXT,
    sort_order    INTEGER  NOT NULL DEFAULT 0,  -- controls display order in the menu
    is_active     INTEGER  NOT NULL DEFAULT 1 CHECK(is_active IN (0, 1)),
    created_by_id TEXT     NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);