CREATE TABLE IF NOT EXISTS menu_items (
    id            TEXT     PRIMARY KEY NOT NULL,
    name          TEXT     NOT NULL,
    description   TEXT,
    price         REAL     NOT NULL CHECK(price >= 0),
    category_id     TEXT     NOT NULL REFERENCES menu_categories(id) ON DELETE RESTRICT,
    created_by_id TEXT     NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    is_active     INTEGER  NOT NULL DEFAULT 1 CHECK(is_active IN (0, 1)),
    created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);