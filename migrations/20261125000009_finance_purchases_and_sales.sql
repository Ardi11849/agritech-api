-- Up Migration: Pillar 4 (Finance Phase 1) — Purchases, Sales, & Inventory Sync Relation

-- 1. Purchase Headers Table (Nota Pembelian)
CREATE TABLE IF NOT EXISTS purchase_headers (
    id VARCHAR(50) PRIMARY KEY,
    invoice_number VARCHAR(50) NOT NULL UNIQUE,
    transaction_date DATE NOT NULL,
    supplier_id VARCHAR(50) REFERENCES suppliers(id)
        ON UPDATE CASCADE ON DELETE SET NULL,
    supplier_name_snapshot VARCHAR(150) NOT NULL DEFAULT '',
    location_id VARCHAR(50) NOT NULL REFERENCES locations(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,

    subtotal_amount NUMERIC(14, 2) NOT NULL DEFAULT 0.00,
    global_discount NUMERIC(14, 2) NOT NULL DEFAULT 0.00,
    global_shipping NUMERIC(14, 2) NOT NULL DEFAULT 0.00,
    total_amount NUMERIC(14, 2) NOT NULL DEFAULT 0.00,

    notes TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) NOT NULL DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) NOT NULL DEFAULT 'system',

    CONSTRAINT chk_purchase_headers_amounts_non_neg CHECK (
        subtotal_amount >= 0.00
        AND global_discount >= 0.00
        AND global_shipping >= 0.00
        AND total_amount >= 0.00
    ),
    CONSTRAINT chk_purchase_headers_discount_le_sub CHECK (
        global_discount <= subtotal_amount
    ),
    CONSTRAINT chk_purchase_headers_total_eq CHECK (
        total_amount = subtotal_amount - global_discount + global_shipping
    )
);

CREATE INDEX IF NOT EXISTS idx_purchase_headers_tx_date
    ON purchase_headers (transaction_date DESC, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_purchase_headers_location_id
    ON purchase_headers (location_id);
CREATE INDEX IF NOT EXISTS idx_purchase_headers_supplier_id
    ON purchase_headers (supplier_id) WHERE supplier_id IS NOT NULL;

-- 2. Purchase Items Table (Detail Item Pembelian + Alokasi Biaya HPP)
CREATE TABLE IF NOT EXISTS purchase_items (
    id VARCHAR(50) PRIMARY KEY,
    purchase_id VARCHAR(50) NOT NULL REFERENCES purchase_headers(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    line_number INTEGER NOT NULL,
    item_id VARCHAR(50) REFERENCES master_items(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    item_name VARCHAR(150) NOT NULL,
    category VARCHAR(100) NOT NULL DEFAULT 'Umum',
    unit VARCHAR(30) NOT NULL DEFAULT 'PCS',

    quantity NUMERIC(12, 2) NOT NULL,
    price_per_unit NUMERIC(14, 2) NOT NULL,
    subtotal NUMERIC(14, 2) NOT NULL,
    discount NUMERIC(14, 2) NOT NULL DEFAULT 0.00,
    shipping_cost NUMERIC(14, 2) NOT NULL DEFAULT 0.00,
    total_cost NUMERIC(14, 2) NOT NULL,
    effective_unit_cost NUMERIC(14, 2) NOT NULL,

    allocation_type VARCHAR(20) NOT NULL,
    session_id VARCHAR(50) REFERENCES planting_sessions(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    allocation_days INTEGER NOT NULL DEFAULT 1,
    daily_allocation_rate NUMERIC(14, 2) NOT NULL DEFAULT 0.00,
    is_synced_to_inventory BOOLEAN NOT NULL DEFAULT FALSE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT uq_purchase_items_line UNIQUE (purchase_id, line_number),
    CONSTRAINT chk_purchase_items_line_positive CHECK (line_number >= 1),
    CONSTRAINT chk_purchase_items_name_nonempty CHECK (TRIM(item_name) <> ''),
    CONSTRAINT chk_purchase_items_unit_nonempty CHECK (TRIM(unit) <> ''),
    CONSTRAINT chk_purchase_items_qty_positive CHECK (quantity > 0.00),
    CONSTRAINT chk_purchase_items_amounts_non_neg CHECK (
        price_per_unit >= 0.00
        AND subtotal >= 0.00
        AND discount >= 0.00
        AND shipping_cost >= 0.00
        AND total_cost >= 0.00
        AND effective_unit_cost >= 0.00
        AND daily_allocation_rate >= 0.00
    ),
    CONSTRAINT chk_purchase_items_total_eq CHECK (
        total_cost = subtotal - discount + shipping_cost
    ),
    CONSTRAINT chk_purchase_items_allocation_type CHECK (
        allocation_type IN ('Direct', 'Shared', 'Asset')
    ),
    CONSTRAINT chk_purchase_items_direct_session CHECK (
        (allocation_type = 'Direct' AND session_id IS NOT NULL)
        OR (allocation_type IN ('Shared', 'Asset') AND session_id IS NULL)
    ),
    CONSTRAINT chk_purchase_items_allocation_days CHECK (
        allocation_days >= 1 AND allocation_days <= 3650
    )
);

CREATE INDEX IF NOT EXISTS idx_purchase_items_purchase_id
    ON purchase_items (purchase_id, line_number ASC);
CREATE INDEX IF NOT EXISTS idx_purchase_items_item_id
    ON purchase_items (item_id) WHERE item_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_purchase_items_session_id
    ON purchase_items (session_id) WHERE session_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_purchase_items_allocation_type
    ON purchase_items (allocation_type);

-- 3. Hubungkan inventory_ledgers dengan purchase_items (Pelacakan Auto-Sync STOCK_IN)
ALTER TABLE inventory_ledgers
    ADD COLUMN IF NOT EXISTS purchase_item_id VARCHAR(50)
        REFERENCES purchase_items(id) ON UPDATE CASCADE ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_inventory_ledgers_purchase_item_id
    ON inventory_ledgers (purchase_item_id) WHERE purchase_item_id IS NOT NULL;

-- 4. Sales Table (Penjualan Hasil Panen, Piutang, & Subsidi Ongkir)
CREATE TABLE IF NOT EXISTS sales (
    id VARCHAR(50) PRIMARY KEY,
    sale_number VARCHAR(50) NOT NULL UNIQUE,
    sale_date DATE NOT NULL,
    session_id VARCHAR(50) NOT NULL REFERENCES planting_sessions(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    location_id VARCHAR(50) NOT NULL REFERENCES locations(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    customer_id VARCHAR(50) REFERENCES customers(id)
        ON UPDATE CASCADE ON DELETE SET NULL,
    customer_name_snapshot VARCHAR(150) NOT NULL DEFAULT '',
    sales_channel VARCHAR(50) NOT NULL DEFAULT 'Direct',

    unit VARCHAR(30) NOT NULL DEFAULT 'KG',
    requested_quantity NUMERIC(12, 2) NOT NULL DEFAULT 0.00,
    quantity NUMERIC(12, 2) NOT NULL,
    fulfillment_rate_percent NUMERIC(5, 2) NOT NULL DEFAULT 100.00,

    price_per_unit NUMERIC(14, 2) NOT NULL,
    gross_revenue NUMERIC(14, 2) NOT NULL,
    discount NUMERIC(14, 2) NOT NULL DEFAULT 0.00,
    total_revenue NUMERIC(14, 2) NOT NULL,
    shipping_cost NUMERIC(14, 2) NOT NULL DEFAULT 0.00,
    shipping_borne_by VARCHAR(20) NOT NULL DEFAULT 'Pembeli',
    seller_shipping_subsidy NUMERIC(14, 2) NOT NULL DEFAULT 0.00,
    net_proceeds NUMERIC(14, 2) NOT NULL,

    status VARCHAR(20) NOT NULL DEFAULT 'Order',
    payment_status VARCHAR(20) NOT NULL DEFAULT 'Piutang',
    paid_at TIMESTAMPTZ,

    notes TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) NOT NULL DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) NOT NULL DEFAULT 'system',

    CONSTRAINT chk_sales_unit_nonempty CHECK (TRIM(unit) <> ''),
    CONSTRAINT chk_sales_qty_valid CHECK (requested_quantity >= 0.00 AND quantity > 0.00),
    CONSTRAINT chk_sales_fulfillment CHECK (
        fulfillment_rate_percent >= 0.00 AND fulfillment_rate_percent <= 100.00
    ),
    CONSTRAINT chk_sales_amounts_non_neg CHECK (
        price_per_unit >= 0.00
        AND gross_revenue >= 0.00
        AND discount >= 0.00
        AND total_revenue >= 0.00
        AND shipping_cost >= 0.00
        AND seller_shipping_subsidy >= 0.00
    ),
    CONSTRAINT chk_sales_discount_le_gross CHECK (discount <= gross_revenue),
    CONSTRAINT chk_sales_revenue_eq CHECK (total_revenue = gross_revenue - discount),
    CONSTRAINT chk_sales_net_proceeds_eq CHECK (
        net_proceeds = total_revenue - seller_shipping_subsidy
    ),
    CONSTRAINT chk_sales_shipping_borne_by CHECK (shipping_borne_by IN ('Pembeli', 'Penjual')),
    CONSTRAINT chk_sales_status CHECK (status IN ('Order', 'Realized')),
    CONSTRAINT chk_sales_payment_status CHECK (payment_status IN ('Lunas', 'Piutang'))
);

CREATE INDEX IF NOT EXISTS idx_sales_sale_date ON sales (sale_date DESC, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_sales_session_id ON sales (session_id);
CREATE INDEX IF NOT EXISTS idx_sales_location_id ON sales (location_id);
CREATE INDEX IF NOT EXISTS idx_sales_customer_id
    ON sales (customer_id) WHERE customer_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_sales_status_payment ON sales (status, payment_status);

-- 5. Registrasi Idempotent Grup "Finance" & Modul /finance/purchases, /finance/sales (Existing DB Only)
DO $$
DECLARE
    v_finance_group_id VARCHAR(50);
    v_purchases_mod_id VARCHAR(50);
    v_sales_mod_id VARCHAR(50);
    r_admin RECORD;
BEGIN
    -- Guard Fresh DB: Jalankan hanya bila roles sudah terisi (seeder.rs menangani fresh DB)
    IF EXISTS (SELECT 1 FROM roles) THEN
        SELECT id INTO v_finance_group_id
        FROM system_module_groups
        WHERE LOWER(TRIM(name)) IN ('finance', 'keuangan')
        LIMIT 1;

        IF v_finance_group_id IS NULL THEN
            v_finance_group_id := 'grp_finance';
            INSERT INTO system_module_groups (id, name, icon, is_active, created_by, modify_by)
            VALUES (v_finance_group_id, 'Finance', 'LuWallet', TRUE, 'migration-finance-p1', 'migration-finance-p1')
            ON CONFLICT (id) DO NOTHING;
        END IF;

        -- Modul /finance/purchases
        SELECT id INTO v_purchases_mod_id
        FROM system_modules
        WHERE LOWER(TRIM(path)) IN ('/finance/purchases', '/purchases', '/pembelian')
        LIMIT 1;

        IF v_purchases_mod_id IS NULL THEN
            v_purchases_mod_id := 'mod_finance_purchases';
            INSERT INTO system_modules (id, group_id, parent_id, name, path, icon, is_active, created_by, modify_by)
            VALUES (v_purchases_mod_id, v_finance_group_id, NULL, 'Pembelian', '/finance/purchases', 'LuShoppingCart', TRUE, 'migration-finance-p1', 'migration-finance-p1')
            ON CONFLICT (id) DO NOTHING;
        ELSE
            UPDATE system_modules
            SET group_id = v_finance_group_id,
                path = '/finance/purchases',
                modify_at = CURRENT_TIMESTAMP,
                modify_by = 'migration-finance-p1'
            WHERE id = v_purchases_mod_id;
        END IF;

        -- Modul /finance/sales
        SELECT id INTO v_sales_mod_id
        FROM system_modules
        WHERE LOWER(TRIM(path)) IN ('/finance/sales', '/sales', '/penjualan')
        LIMIT 1;

        IF v_sales_mod_id IS NULL THEN
            v_sales_mod_id := 'mod_finance_sales';
            INSERT INTO system_modules (id, group_id, parent_id, name, path, icon, is_active, created_by, modify_by)
            VALUES (v_sales_mod_id, v_finance_group_id, NULL, 'Penjualan', '/finance/sales', 'LuBanknote', TRUE, 'migration-finance-p1', 'migration-finance-p1')
            ON CONFLICT (id) DO NOTHING;
        ELSE
            UPDATE system_modules
            SET group_id = v_finance_group_id,
                path = '/finance/sales',
                modify_at = CURRENT_TIMESTAMP,
                modify_by = 'migration-finance-p1'
            WHERE id = v_sales_mod_id;
        END IF;

        -- Grant Full Permissions ke ADMINISTRATOR / SUPER ADMIN
        FOR r_admin IN
            SELECT id FROM roles WHERE UPPER(TRIM(name)) IN ('ADMINISTRATOR', 'SUPER ADMIN')
        LOOP
            INSERT INTO role_module_permissions (
                id, role_id, module_group_id, module_id,
                can_create, can_read, can_update, can_delete, created_by, modify_by
            ) VALUES (
                'perm_prc_' || LEFT(r_admin.id, 36), r_admin.id, v_finance_group_id, v_purchases_mod_id,
                TRUE, TRUE, TRUE, TRUE, 'migration-finance-p1', 'migration-finance-p1'
            ) ON CONFLICT (role_id, module_group_id, module_id) DO NOTHING;

            INSERT INTO role_module_permissions (
                id, role_id, module_group_id, module_id,
                can_create, can_read, can_update, can_delete, created_by, modify_by
            ) VALUES (
                'perm_sls_' || LEFT(r_admin.id, 36), r_admin.id, v_finance_group_id, v_sales_mod_id,
                TRUE, TRUE, TRUE, TRUE, 'migration-finance-p1', 'migration-finance-p1'
            ) ON CONFLICT (role_id, module_group_id, module_id) DO NOTHING;
        END LOOP;
    END IF;
END $$;
