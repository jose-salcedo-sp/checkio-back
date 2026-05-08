CREATE TABLE IF NOT EXISTS users (
    id            TEXT     PRIMARY KEY NOT NULL,
    name          TEXT     NOT NULL,
    email         TEXT     NOT NULL UNIQUE,
    role          TEXT     NOT NULL DEFAULT 'waiter' CHECK(role IN ('waiter', 'manager', 'admin')),
    is_active     INTEGER  NOT NULL DEFAULT 1 CHECK(is_active IN (0, 1)),
    password_hash TEXT     NOT NULL DEFAULT '',
    created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);