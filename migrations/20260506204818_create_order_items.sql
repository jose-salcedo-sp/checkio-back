CREATE TABLE IF NOT EXISTS order_items (
    id             TEXT    PRIMARY KEY NOT NULL,  -- UUID stored as TEXT
    order_id       TEXT    NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    menu_item_id   TEXT    NOT NULL,              -- will reference menu_items once created
    quantity       INTEGER NOT NULL CHECK(quantity > 0),
    unit_price     REAL    NOT NULL CHECK(unit_price >= 0),  -- snapshot price at time of order
    created_by_id TEXT NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_order_items_order_id ON order_items(order_id);