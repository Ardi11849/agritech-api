-- Up Migration: Inventory & Warehouse Ledger

-- 1. Tambahkan kolom saldo stok riil & rata-rata biaya satuan pada master_items
ALTER TABLE master_items
    ADD COLUMN IF NOT EXISTS current_stock NUMERIC(12, 2) NOT NULL DEFAULT 0.00,
    ADD COLUMN IF NOT EXISTS average_unit_cost NUMERIC(14, 2) NOT NULL DEFAULT 0.00;

ALTER TABLE master_items
    ADD CONSTRAINT chk_master_items_current_stock CHECK (current_stock >= 0.00),
    ADD CONSTRAINT chk_master_items_average_unit_cost CHECK (average_unit_cost >= 0.00);

CREATE INDEX IF NOT EXISTS idx_master_items_current_stock
    ON master_items (current_stock);

-- 2. Tabel Buku Besar Mutasi Gudang (Append-Only Immutable Ledger)
CREATE TABLE IF NOT EXISTS inventory_ledgers (
    id VARCHAR(50) PRIMARY KEY,
    item_id VARCHAR(50) NOT NULL REFERENCES master_items(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    movement_type VARCHAR(30) NOT NULL,
    quantity NUMERIC(12, 2) NOT NULL,
    delta_quantity NUMERIC(12, 2) NOT NULL,
    balance_before NUMERIC(12, 2) NOT NULL,
    balance_after NUMERIC(12, 2) NOT NULL,
    unit_cost NUMERIC(14, 2) NOT NULL DEFAULT 0.00,
    average_unit_cost_after NUMERIC(14, 2) NOT NULL DEFAULT 0.00,
    total_cost_value NUMERIC(14, 2) NOT NULL DEFAULT 0.00,

    -- Relasi Mitra & Tujuan Opsional (Audit Trail tetap aman jika master mitra dihapus)
    supplier_id VARCHAR(50) REFERENCES suppliers(id)
        ON UPDATE CASCADE ON DELETE SET NULL,
    customer_id VARCHAR(50) REFERENCES customers(id)
        ON UPDATE CASCADE ON DELETE SET NULL,
    installation_id VARCHAR(50) REFERENCES installations(id)
        ON UPDATE CASCADE ON DELETE SET NULL,

    reference_no VARCHAR(100) NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    transaction_date TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) NOT NULL DEFAULT 'system',

    -- Domain Invariants di Level Database
    CONSTRAINT chk_inventory_movement_type CHECK (
        movement_type IN ('STOCK_IN', 'STOCK_OUT', 'RETURN_IN', 'RETURN_OUT', 'OPNAME_ADJUSTMENT')
    ),
    CONSTRAINT chk_inventory_quantity_positive CHECK (
        (movement_type = 'OPNAME_ADJUSTMENT' AND quantity >= 0.00)
        OR (movement_type <> 'OPNAME_ADJUSTMENT' AND quantity > 0.00)
    ),
    CONSTRAINT chk_inventory_balances_non_negative CHECK (
        balance_before >= 0.00 AND balance_after >= 0.00
    ),
    CONSTRAINT chk_inventory_costs_non_negative CHECK (
        unit_cost >= 0.00 AND average_unit_cost_after >= 0.00 AND total_cost_value >= 0.00
    ),
    CONSTRAINT chk_inventory_balance_equation CHECK (
        balance_after = balance_before + delta_quantity
    )
);

-- Indeks Performa Kartu Stok & Filter Analitik
CREATE INDEX IF NOT EXISTS idx_inventory_ledgers_item_timeline
    ON inventory_ledgers (item_id, transaction_date DESC, created_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_inventory_ledgers_movement_type
    ON inventory_ledgers (movement_type);
CREATE INDEX IF NOT EXISTS idx_inventory_ledgers_transaction_date
    ON inventory_ledgers (transaction_date DESC);
CREATE INDEX IF NOT EXISTS idx_inventory_ledgers_supplier_id
    ON inventory_ledgers (supplier_id) WHERE supplier_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_inventory_ledgers_customer_id
    ON inventory_ledgers (customer_id) WHERE customer_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_inventory_ledgers_installation_id
    ON inventory_ledgers (installation_id) WHERE installation_id IS NOT NULL;
