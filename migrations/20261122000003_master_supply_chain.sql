-- Up Migration: Master Supply Chain (item_categories, master_items, suppliers, customers)

-- 1. Item Categories Table (Master Kategori Barang)
CREATE TABLE IF NOT EXISTS item_categories (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) DEFAULT 'system'
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_item_categories_name_lower
    ON item_categories (LOWER(name));
CREATE INDEX IF NOT EXISTS idx_item_categories_is_active
    ON item_categories (is_active);

-- 2. Master Items Table (Katalog Barang & Ambang Stok Minimum)
CREATE TABLE IF NOT EXISTS master_items (
    id VARCHAR(50) PRIMARY KEY,
    category_id VARCHAR(50) NOT NULL REFERENCES item_categories(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    name VARCHAR(150) NOT NULL,
    default_unit VARCHAR(30) NOT NULL DEFAULT 'PCS',
    min_stock_level NUMERIC(12, 2) NOT NULL DEFAULT 0.00,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) DEFAULT 'system',

    CONSTRAINT chk_master_items_min_stock CHECK (min_stock_level >= 0.00),
    CONSTRAINT chk_master_items_unit_nonempty CHECK (TRIM(default_unit) <> '')
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_master_items_name_lower
    ON master_items (LOWER(name));
CREATE INDEX IF NOT EXISTS idx_master_items_category_id
    ON master_items (category_id);
CREATE INDEX IF NOT EXISTS idx_master_items_is_active
    ON master_items (is_active);

-- 3. Suppliers Table (Master Pemasok / Vendor)
CREATE TABLE IF NOT EXISTS suppliers (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(150) NOT NULL,
    category VARCHAR(100) NOT NULL DEFAULT 'Lainnya',
    contact_person VARCHAR(100) NOT NULL DEFAULT '',
    phone VARCHAR(25),
    address TEXT NOT NULL DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) DEFAULT 'system'
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_suppliers_name_lower
    ON suppliers (LOWER(name));
CREATE INDEX IF NOT EXISTS idx_suppliers_category
    ON suppliers (category);
CREATE INDEX IF NOT EXISTS idx_suppliers_is_active
    ON suppliers (is_active);

-- 4. Customers Table (Master Pelanggan / Mitra Pembeli)
CREATE TABLE IF NOT EXISTS customers (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(150) NOT NULL,
    category VARCHAR(100) NOT NULL DEFAULT 'End-User',
    contact_person VARCHAR(100) NOT NULL DEFAULT '',
    phone VARCHAR(25),
    address TEXT NOT NULL DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) DEFAULT 'system'
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_customers_name_lower
    ON customers (LOWER(name));
CREATE INDEX IF NOT EXISTS idx_customers_category
    ON customers (category);
CREATE INDEX IF NOT EXISTS idx_customers_is_active
    ON customers (is_active);

-- 5. Seed Kategori Barang Standar Awal (Idempotent, dieksekusi sekali via SQLx migration)
INSERT INTO item_categories (id, name, description, is_active, created_by, modify_by)
VALUES
    ('cat_benih_bibit', 'Benih & Bibit', 'Benih tanaman, bibit semai, dan propagasi', TRUE, 'system', 'system'),
    ('cat_nutrisi_pupuk', 'Nutrisi & Pupuk', 'Larutan nutrisi AB Mix, pH Up/Down, dan pupuk pelengkap', TRUE, 'system', 'system'),
    ('cat_media_tanam', 'Media Tanam', 'Rockwool, netpot, cocopeat, dan media semai lainnya', TRUE, 'system', 'system'),
    ('cat_peralatan_aset', 'Peralatan & Aset', 'Pompa, sensor TDS/pH, pipa, tray, dan perlengkapan kebun', TRUE, 'system', 'system'),
    ('cat_kemasan', 'Kemasan', 'Plastik kemasan panen, label, stiker, dan kardus pengiriman', TRUE, 'system', 'system')
ON CONFLICT DO NOTHING;
