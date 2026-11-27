-- Up Migration: Pillar 4 (Finance Phase 2) — HPP Snapshot Columns & MSME Tax Management

-- 1. Tambahkan kolom Snapshot Finansial HPP pada planting_sessions
ALTER TABLE planting_sessions
    ADD COLUMN IF NOT EXISTS final_hpp NUMERIC(14, 2),
    ADD COLUMN IF NOT EXISTS final_revenue NUMERIC(14, 2),
    ADD COLUMN IF NOT EXISTS final_shipping_subsidy NUMERIC(14, 2),
    ADD COLUMN IF NOT EXISTS final_net_profit NUMERIC(14, 2),
    ADD COLUMN IF NOT EXISTS hpp_snapshot_locked_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS hpp_snapshot_locked_by VARCHAR(100);

ALTER TABLE planting_sessions
    DROP CONSTRAINT IF EXISTS chk_planting_sessions_final_hpp_non_neg;

ALTER TABLE planting_sessions
    ADD CONSTRAINT chk_planting_sessions_final_hpp_non_neg CHECK (
        (final_hpp IS NULL OR final_hpp >= 0.00)
        AND (final_revenue IS NULL OR final_revenue >= 0.00)
        AND (final_shipping_subsidy IS NULL OR final_shipping_subsidy >= 0.00)
    );

-- 2. Tabel Konfigurasi Tarif Pajak (tax_settings)
CREATE TABLE IF NOT EXISTS tax_settings (
    id VARCHAR(50) PRIMARY KEY,
    scheme_name VARCHAR(150) NOT NULL,
    rate NUMERIC(6, 4) NOT NULL DEFAULT 0.0050,
    description TEXT NOT NULL DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) NOT NULL DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) NOT NULL DEFAULT 'system',

    CONSTRAINT chk_tax_settings_name_nonempty CHECK (TRIM(scheme_name) <> ''),
    CONSTRAINT chk_tax_settings_rate_range CHECK (rate >= 0.0000 AND rate <= 1.0000)
);

-- Seed Default: PPh Final UMKM 0.50% (Idempotent, aman pada existing & fresh DB)
INSERT INTO tax_settings (id, scheme_name, rate, description, is_active, created_by, modify_by)
VALUES (
    'tax_cfg_default',
    'PPh Final UMKM (PP 23/2018 & PP 55/2022)',
    0.0050,
    'Dikenakan atas peredaran bruto penjualan bagi wajib pajak dengan omzet < Rp 4.8 Miliar per tahun',
    TRUE,
    'migration-finance-p2',
    'migration-finance-p2'
)
ON CONFLICT (id) DO NOTHING;

-- 3. Tabel Riwayat Laporan & Pembayaran Pajak Bulanan (tax_reports)
CREATE TABLE IF NOT EXISTS tax_reports (
    id VARCHAR(50) PRIMARY KEY,
    period_month INTEGER NOT NULL,
    period_year INTEGER NOT NULL,
    total_revenue NUMERIC(14, 2) NOT NULL DEFAULT 0.00,
    tax_rate NUMERIC(6, 4) NOT NULL DEFAULT 0.0050,
    tax_amount NUMERIC(14, 2) NOT NULL DEFAULT 0.00,
    status VARCHAR(20) NOT NULL DEFAULT 'Unpaid',
    payment_date TIMESTAMPTZ,
    billing_code VARCHAR(50) NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) NOT NULL DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) NOT NULL DEFAULT 'system',

    CONSTRAINT uq_tax_reports_period UNIQUE (period_month, period_year),
    CONSTRAINT chk_tax_reports_month CHECK (period_month >= 1 AND period_month <= 12),
    CONSTRAINT chk_tax_reports_year CHECK (period_year >= 2000 AND period_year <= 2100),
    CONSTRAINT chk_tax_reports_amounts_non_neg CHECK (
        total_revenue >= 0.00 AND tax_amount >= 0.00
    ),
    CONSTRAINT chk_tax_reports_rate_range CHECK (tax_rate >= 0.0000 AND tax_rate <= 1.0000),
    CONSTRAINT chk_tax_reports_status CHECK (status IN ('Unpaid', 'Paid'))
);

CREATE INDEX IF NOT EXISTS idx_tax_reports_period_desc
    ON tax_reports (period_year DESC, period_month DESC);
CREATE INDEX IF NOT EXISTS idx_tax_reports_status
    ON tax_reports (status);

-- 4. Registrasi Idempotent Modul /finance/hpp & /finance/tax (Existing DB Only, Fresh-DB Safe)
DO $$
DECLARE
    v_finance_group_id VARCHAR(50);
    v_hpp_mod_id VARCHAR(50);
    v_tax_mod_id VARCHAR(50);
    r_admin RECORD;
BEGIN
    IF EXISTS (SELECT 1 FROM roles) THEN
        SELECT id INTO v_finance_group_id
        FROM system_module_groups
        WHERE LOWER(TRIM(name)) IN ('finance', 'keuangan')
        LIMIT 1;

        IF v_finance_group_id IS NULL THEN
            v_finance_group_id := 'grp_finance';
            INSERT INTO system_module_groups (id, name, icon, is_active, created_by, modify_by)
            VALUES (v_finance_group_id, 'Finance', 'LuWallet', TRUE, 'migration-finance-p2', 'migration-finance-p2')
            ON CONFLICT (id) DO NOTHING;
        END IF;

        -- Modul /finance/hpp
        SELECT id INTO v_hpp_mod_id
        FROM system_modules
        WHERE LOWER(TRIM(path)) IN ('/finance/hpp', '/hpp')
        LIMIT 1;

        IF v_hpp_mod_id IS NULL THEN
            v_hpp_mod_id := 'mod_finance_hpp';
            INSERT INTO system_modules (id, group_id, parent_id, name, path, icon, is_active, created_by, modify_by)
            VALUES (v_hpp_mod_id, v_finance_group_id, NULL, 'Analisa HPP', '/finance/hpp', 'LuCalculator', TRUE, 'migration-finance-p2', 'migration-finance-p2')
            ON CONFLICT (id) DO NOTHING;
        ELSE
            UPDATE system_modules
            SET group_id = v_finance_group_id,
                path = '/finance/hpp',
                modify_at = CURRENT_TIMESTAMP,
                modify_by = 'migration-finance-p2'
            WHERE id = v_hpp_mod_id;
        END IF;

        -- Modul /finance/tax
        SELECT id INTO v_tax_mod_id
        FROM system_modules
        WHERE LOWER(TRIM(path)) IN ('/finance/tax', '/tax')
        LIMIT 1;

        IF v_tax_mod_id IS NULL THEN
            v_tax_mod_id := 'mod_finance_tax';
            INSERT INTO system_modules (id, group_id, parent_id, name, path, icon, is_active, created_by, modify_by)
            VALUES (v_tax_mod_id, v_finance_group_id, NULL, 'Perpajakan', '/finance/tax', 'LuReceipt', TRUE, 'migration-finance-p2', 'migration-finance-p2')
            ON CONFLICT (id) DO NOTHING;
        ELSE
            UPDATE system_modules
            SET group_id = v_finance_group_id,
                path = '/finance/tax',
                modify_at = CURRENT_TIMESTAMP,
                modify_by = 'migration-finance-p2'
            WHERE id = v_tax_mod_id;
        END IF;

        -- Grant Full Permissions ke ADMINISTRATOR / SUPER ADMIN
        FOR r_admin IN
            SELECT id FROM roles WHERE UPPER(TRIM(name)) IN ('ADMINISTRATOR', 'SUPER ADMIN')
        LOOP
            INSERT INTO role_module_permissions (
                id, role_id, module_group_id, module_id,
                can_create, can_read, can_update, can_delete, created_by, modify_by
            ) VALUES (
                'perm_hpp_' || LEFT(r_admin.id, 36), r_admin.id, v_finance_group_id, v_hpp_mod_id,
                TRUE, TRUE, TRUE, TRUE, 'migration-finance-p2', 'migration-finance-p2'
            ) ON CONFLICT (role_id, module_group_id, module_id) DO NOTHING;

            INSERT INTO role_module_permissions (
                id, role_id, module_group_id, module_id,
                can_create, can_read, can_update, can_delete, created_by, modify_by
            ) VALUES (
                'perm_tax_' || LEFT(r_admin.id, 36), r_admin.id, v_finance_group_id, v_tax_mod_id,
                TRUE, TRUE, TRUE, TRUE, 'migration-finance-p2', 'migration-finance-p2'
            ) ON CONFLICT (role_id, module_group_id, module_id) DO NOTHING;
        END LOOP;
    END IF;
END $$;
