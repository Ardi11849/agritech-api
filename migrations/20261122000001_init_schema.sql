-- Up Migration: Initial Schema for Agritech Core API

-- 1. Roles Table
CREATE TABLE IF NOT EXISTS roles (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) DEFAULT 'system'
);

CREATE INDEX IF NOT EXISTS idx_roles_name ON roles(name);

-- 2. Users Table
CREATE TABLE IF NOT EXISTS users (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(100) NOT NULL UNIQUE,
    username VARCHAR(50) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    role_id VARCHAR(50) NOT NULL REFERENCES roles(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) DEFAULT 'system'
);

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_role_id ON users(role_id);

-- 3. System Module Groups Table
CREATE TABLE IF NOT EXISTS system_module_groups (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    icon VARCHAR(100) DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) DEFAULT 'system'
);

-- 4. System Modules Table (Self-referencing for Sub-Modules)
CREATE TABLE IF NOT EXISTS system_modules (
    id VARCHAR(50) PRIMARY KEY,
    group_id VARCHAR(50) NOT NULL REFERENCES system_module_groups(id) ON DELETE CASCADE,
    parent_id VARCHAR(50) REFERENCES system_modules(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    path VARCHAR(100) NOT NULL,
    icon VARCHAR(100) DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) DEFAULT 'system'
);

CREATE INDEX IF NOT EXISTS idx_system_modules_group_id ON system_modules(group_id);
CREATE INDEX IF NOT EXISTS idx_system_modules_parent_id ON system_modules(parent_id);

-- 5. Role Module Permissions Table (RBAC Matrix)
CREATE TABLE IF NOT EXISTS role_module_permissions (
    id VARCHAR(50) PRIMARY KEY,
    role_id VARCHAR(50) NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    module_group_id VARCHAR(50) NOT NULL REFERENCES system_module_groups(id) ON DELETE CASCADE,
    module_id VARCHAR(50) NOT NULL REFERENCES system_modules(id) ON DELETE CASCADE,
    can_create BOOLEAN NOT NULL DEFAULT FALSE,
    can_read BOOLEAN NOT NULL DEFAULT FALSE,
    can_update BOOLEAN NOT NULL DEFAULT FALSE,
    can_delete BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) DEFAULT 'system',
    CONSTRAINT idx_role_module UNIQUE (role_id, module_group_id, module_id)
);

CREATE INDEX IF NOT EXISTS idx_rmp_role_id ON role_module_permissions(role_id);
CREATE INDEX IF NOT EXISTS idx_rmp_module_id ON role_module_permissions(module_id);
