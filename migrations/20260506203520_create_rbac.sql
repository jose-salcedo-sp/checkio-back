-- RBAC: permission scopes, roles, fine-grained permissions, role grants.
-- Role IDs are fixed UUIDs for stable references (e.g. DEFAULT on users.role_id).

CREATE TABLE IF NOT EXISTS permission_resources (
    id          TEXT PRIMARY KEY NOT NULL,
    description TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS roles (
    id          TEXT PRIMARY KEY NOT NULL,
    slug        TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    description TEXT NOT NULL,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS permissions (
    id           TEXT PRIMARY KEY NOT NULL,
    resource_id  TEXT NOT NULL REFERENCES permission_resources(id) ON DELETE RESTRICT,
    action       TEXT NOT NULL CHECK(action IN ('create', 'read', 'update', 'delete')),
    description  TEXT NOT NULL,
    UNIQUE(resource_id, action)
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id        TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id  TEXT NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY(role_id, permission_id)
);

CREATE INDEX IF NOT EXISTS idx_permissions_resource_id ON permissions(resource_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_role_id ON role_permissions(role_id);

-- Permission scopes (what area of the API a grant applies to; not dining-table rows).
INSERT INTO permission_resources (id, description) VALUES
('users', 'Staff accounts: create and manage users under /api/users.'),
('orders', 'Guest checks and tab lifecycle under /api/orders.'),
('menu_items', 'Menu dishes and pricing under /api/menu_items.'),
('menu_categories', 'Menu sections and ordering under /api/menu_item_categories.'),
('tables', 'Floor-plan seating definitions under /api/tables.');

-- Roles
INSERT INTO roles (id, slug, name, description) VALUES
('a0000001-0001-4000-8000-000000000001', 'admin', 'Administrator', 'Full access to all resources; configure staff, menus, seating, and orders.'),
('a0000001-0001-4000-8000-000000000002', 'waiter', 'Waiter', 'Serves tables: manage orders; view menus and seating; read-only on staff list.'),
('a0000001-0001-4000-8000-000000000003', 'hostess', 'Hostess', 'Greets and seats guests: manage seating layout; create and update orders; view menus.'),
('a0000001-0001-4000-8000-000000000004', 'readonly', 'Read-only', 'View menus, orders, tables, and staff; cannot change data.');

-- Permissions: one row per resource × CRUD verb (id = resource.action for stable references).
INSERT INTO permissions (id, resource_id, action, description) VALUES
-- users
('users.create', 'users', 'create', 'Create new user accounts.'),
('users.read', 'users', 'read', 'View user profiles and directory.'),
('users.update', 'users', 'update', 'Edit user details, roles, or activation.'),
('users.delete', 'users', 'delete', 'Remove or deactivate users where applicable.'),
-- orders
('orders.create', 'orders', 'create', 'Open new checks and add orders.'),
('orders.read', 'orders', 'read', 'View open and historical orders.'),
('orders.update', 'orders', 'update', 'Modify order contents or status.'),
('orders.delete', 'orders', 'delete', 'Void or cancel orders.'),
-- menu_items
('menu_items.create', 'menu_items', 'create', 'Add new dishes or beverages.'),
('menu_items.read', 'menu_items', 'read', 'View menu items and prices.'),
('menu_items.update', 'menu_items', 'update', 'Edit items, prices, or availability.'),
('menu_items.delete', 'menu_items', 'delete', 'Remove or retire menu items.'),
-- menu_categories
('menu_categories.create', 'menu_categories', 'create', 'Create menu sections.'),
('menu_categories.read', 'menu_categories', 'read', 'View menu categories.'),
('menu_categories.update', 'menu_categories', 'update', 'Rename or reorder categories.'),
('menu_categories.delete', 'menu_categories', 'delete', 'Remove menu sections.'),
-- tables (seating catalog)
('tables.create', 'tables', 'create', 'Add new seating labels or zones.'),
('tables.read', 'tables', 'read', 'View floor-plan table definitions.'),
('tables.update', 'tables', 'update', 'Edit labels, sort order, or active flag.'),
('tables.delete', 'tables', 'delete', 'Remove seating definitions (blocked while orders reference them).');

-- Admin: all permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT 'a0000001-0001-4000-8000-000000000001', id FROM permissions;

-- Waiter: full orders; read menus, seating, users
INSERT INTO role_permissions (role_id, permission_id) VALUES
('a0000001-0001-4000-8000-000000000002', 'orders.create'),
('a0000001-0001-4000-8000-000000000002', 'orders.read'),
('a0000001-0001-4000-8000-000000000002', 'orders.update'),
('a0000001-0001-4000-8000-000000000002', 'orders.delete'),
('a0000001-0001-4000-8000-000000000002', 'menu_items.read'),
('a0000001-0001-4000-8000-000000000002', 'menu_categories.read'),
('a0000001-0001-4000-8000-000000000002', 'tables.read'),
('a0000001-0001-4000-8000-000000000002', 'users.read');

-- Hostess: full seating; orders without delete; read menus
INSERT INTO role_permissions (role_id, permission_id) VALUES
('a0000001-0001-4000-8000-000000000003', 'tables.create'),
('a0000001-0001-4000-8000-000000000003', 'tables.read'),
('a0000001-0001-4000-8000-000000000003', 'tables.update'),
('a0000001-0001-4000-8000-000000000003', 'tables.delete'),
('a0000001-0001-4000-8000-000000000003', 'orders.create'),
('a0000001-0001-4000-8000-000000000003', 'orders.read'),
('a0000001-0001-4000-8000-000000000003', 'orders.update'),
('a0000001-0001-4000-8000-000000000003', 'menu_items.read'),
('a0000001-0001-4000-8000-000000000003', 'menu_categories.read');

-- Read-only: read everywhere
INSERT INTO role_permissions (role_id, permission_id)
SELECT 'a0000001-0001-4000-8000-000000000004', id FROM permissions WHERE action = 'read';
