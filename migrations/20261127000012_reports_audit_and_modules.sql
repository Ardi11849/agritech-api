-- Up Migration: Pillar 5 (Reports) — Audit Trail Ledger, JSONB-Safe DB Triggers,
-- Idempotent Historical Backfill, & System Module Registration (Fresh-DB Safe)

-- ============================================================================
-- 1. Tabel Jejak Audit Sistem (audit_logs — Append-Only Immutable Ledger)
-- ============================================================================
CREATE TABLE IF NOT EXISTS audit_logs (
    id VARCHAR(50) PRIMARY KEY,
    user_id VARCHAR(50),
    username VARCHAR(100) NOT NULL DEFAULT 'system',
    action VARCHAR(30) NOT NULL,
    module VARCHAR(50) NOT NULL,
    target_id VARCHAR(50) NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    old_data JSONB,
    new_data JSONB,
    ip_address VARCHAR(45),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT chk_audit_logs_action_valid CHECK (
        action IN ('CREATE', 'UPDATE', 'DELETE', 'LOGIN', 'LOCK', 'ADJUST')
    ),
    CONSTRAINT chk_audit_logs_module_nonempty CHECK (TRIM(module) <> '')
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at_desc
    ON audit_logs (created_at DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_module_created
    ON audit_logs (module, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action
    ON audit_logs (action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_username_lower
    ON audit_logs (LOWER(username));
CREATE INDEX IF NOT EXISTS idx_audit_logs_target_id
    ON audit_logs (target_id) WHERE target_id <> '';

-- ============================================================================
-- 2. Fungsi Trigger Generik JSONB-Safe (fn_record_audit_log)
--    Aman untuk tabel dengan maupun tanpa kolom modify_by (mis. inventory_ledgers)
--    dan otomatis memetakan aksi semantik ADJUST & LOCK sesuai spesifikasi UI/UX.
-- ============================================================================
CREATE OR REPLACE FUNCTION fn_record_audit_log()
RETURNS TRIGGER AS $$
DECLARE
    v_module      VARCHAR(50);
    v_action      VARCHAR(30);
    v_old_json    JSONB := NULL;
    v_new_json    JSONB := NULL;
    v_ref_json    JSONB;
    v_username    VARCHAR(100);
    v_user_id     VARCHAR(50) := NULL;
    v_target_id   VARCHAR(50) := '';
    v_label       TEXT := '';
    v_description TEXT;
    v_audit_id    VARCHAR(50);
BEGIN
    v_module := UPPER(COALESCE(NULLIF(TRIM(TG_ARGV[0]), ''), TG_TABLE_NAME));

    IF TG_OP = 'INSERT' THEN
        v_new_json := to_jsonb(NEW);
        v_ref_json := v_new_json;
        v_username := COALESCE(NULLIF(TRIM(v_new_json->>'created_by'), ''), 'system');

        IF v_module = 'INVENTORY' AND v_new_json->>'movement_type' = 'OPNAME_ADJUSTMENT' THEN
            v_action := 'ADJUST';
        ELSE
            v_action := 'CREATE';
        END IF;

    ELSIF TG_OP = 'UPDATE' THEN
        v_old_json := to_jsonb(OLD);
        v_new_json := to_jsonb(NEW);

        -- Abaikan no-op update yang tidak mengubah isi baris
        IF v_old_json = v_new_json THEN
            RETURN NEW;
        END IF;

        v_ref_json := v_new_json;
        v_username := COALESCE(
            NULLIF(TRIM(v_new_json->>'modify_by'), ''),
            NULLIF(TRIM(v_new_json->>'created_by'), ''),
            'system'
        );

        IF v_module = 'SESSIONS' AND (
            (
                COALESCE((v_old_json->>'is_locked')::boolean, FALSE) = FALSE
                AND COALESCE((v_new_json->>'is_locked')::boolean, FALSE) = TRUE
            )
            OR (
                v_old_json->>'hpp_snapshot_locked_at' IS NULL
                AND v_new_json->>'hpp_snapshot_locked_at' IS NOT NULL
            )
        ) THEN
            v_action := 'LOCK';
        ELSE
            v_action := 'UPDATE';
        END IF;

    ELSIF TG_OP = 'DELETE' THEN
        v_old_json := to_jsonb(OLD);
        v_ref_json := v_old_json;
        v_username := COALESCE(
            NULLIF(TRIM(v_old_json->>'modify_by'), ''),
            NULLIF(TRIM(v_old_json->>'created_by'), ''),
            'system'
        );
        v_action := 'DELETE';
    ELSE
        RETURN NULL;
    END IF;

    v_target_id := COALESCE(v_ref_json->>'id', '');

    -- Resolve user_id opsional dari tabel users (tidak error jika aktor 'system' atau service account)
    IF LOWER(v_username) <> 'system' THEN
        SELECT u.id INTO v_user_id
        FROM users u
        WHERE LOWER(u.username) = LOWER(v_username)
        LIMIT 1;
    END IF;

    -- Bangun label manusiawi untuk kolom description
    v_label := COALESCE(
        NULLIF(TRIM(v_ref_json->>'invoice_number'), ''),
        NULLIF(TRIM(v_ref_json->>'sale_number'), ''),
        NULLIF(TRIM(v_ref_json->>'session_name'), ''),
        NULLIF(TRIM(v_ref_json->>'title'), ''),
        NULLIF(TRIM(v_ref_json->>'name'), ''),
        NULLIF(TRIM(v_ref_json->>'reference_no'), ''),
        v_target_id
    );

    v_description := v_action || ' pada modul ' || v_module ||
        CASE WHEN v_label <> '' THEN ' (' || v_label || ')' ELSE '' END;

    -- Generator ID unik 36 karakter tanpa ketergantungan ekstensi uuid-ossp / pgcrypto
    v_audit_id := 'aud_' || md5(
        clock_timestamp()::text || '-' || random()::text || '-' || v_module || '-' || v_target_id
    );

    INSERT INTO audit_logs (
        id, user_id, username, action, module, target_id,
        description, old_data, new_data, ip_address, created_at
    ) VALUES (
        v_audit_id, v_user_id, v_username, v_action, v_module, v_target_id,
        v_description, v_old_json, v_new_json, NULL, CURRENT_TIMESTAMP
    );

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 3. Pemasangan Trigger Audit pada 8 Tabel Bisnis Utama (Idempotent)
-- ============================================================================
DROP TRIGGER IF EXISTS trg_audit_purchase_headers ON purchase_headers;
CREATE TRIGGER trg_audit_purchase_headers
    AFTER INSERT OR UPDATE OR DELETE ON purchase_headers
    FOR EACH ROW EXECUTE FUNCTION fn_record_audit_log('PURCHASES');

DROP TRIGGER IF EXISTS trg_audit_sales ON sales;
CREATE TRIGGER trg_audit_sales
    AFTER INSERT OR UPDATE OR DELETE ON sales
    FOR EACH ROW EXECUTE FUNCTION fn_record_audit_log('SALES');

DROP TRIGGER IF EXISTS trg_audit_inventory_ledgers ON inventory_ledgers;
CREATE TRIGGER trg_audit_inventory_ledgers
    AFTER INSERT OR UPDATE OR DELETE ON inventory_ledgers
    FOR EACH ROW EXECUTE FUNCTION fn_record_audit_log('INVENTORY');

DROP TRIGGER IF EXISTS trg_audit_planting_sessions ON planting_sessions;
CREATE TRIGGER trg_audit_planting_sessions
    AFTER INSERT OR UPDATE OR DELETE ON planting_sessions
    FOR EACH ROW EXECUTE FUNCTION fn_record_audit_log('SESSIONS');

DROP TRIGGER IF EXISTS trg_audit_tax_reports ON tax_reports;
CREATE TRIGGER trg_audit_tax_reports
    AFTER INSERT OR UPDATE OR DELETE ON tax_reports
    FOR EACH ROW EXECUTE FUNCTION fn_record_audit_log('TAX');

DROP TRIGGER IF EXISTS trg_audit_sops ON sops;
CREATE TRIGGER trg_audit_sops
    AFTER INSERT OR UPDATE OR DELETE ON sops
    FOR EACH ROW EXECUTE FUNCTION fn_record_audit_log('SOP');

DROP TRIGGER IF EXISTS trg_audit_farm_tasks ON farm_tasks;
CREATE TRIGGER trg_audit_farm_tasks
    AFTER INSERT OR UPDATE OR DELETE ON farm_tasks
    FOR EACH ROW EXECUTE FUNCTION fn_record_audit_log('TASKS');

DROP TRIGGER IF EXISTS trg_audit_master_items ON master_items;
CREATE TRIGGER trg_audit_master_items
    AFTER INSERT OR UPDATE OR DELETE ON master_items
    FOR EACH ROW EXECUTE FUNCTION fn_record_audit_log('ITEMS');

-- ============================================================================
-- 4. Idempotent Historical Backfill (Existing DB Data; 0 baris pada Fresh DB)
-- ============================================================================
INSERT INTO audit_logs (
    id, user_id, username, action, module, target_id,
    description, old_data, new_data, ip_address, created_at
)
SELECT
    'bf_prc_' || LEFT(p.id, 40),
    u.id,
    COALESCE(NULLIF(TRIM(p.created_by), ''), 'system'),
    'CREATE',
    'PURCHASES',
    p.id,
    'Backfill historis pembelian (' || p.invoice_number || ')',
    NULL,
    to_jsonb(p),
    NULL,
    p.created_at
FROM purchase_headers p
LEFT JOIN users u ON LOWER(u.username) = LOWER(p.created_by)
ON CONFLICT (id) DO NOTHING;

INSERT INTO audit_logs (
    id, user_id, username, action, module, target_id,
    description, old_data, new_data, ip_address, created_at
)
SELECT
    'bf_sls_' || LEFT(s.id, 40),
    u.id,
    COALESCE(NULLIF(TRIM(s.created_by), ''), 'system'),
    'CREATE',
    'SALES',
    s.id,
    'Backfill historis penjualan (' || s.sale_number || ')',
    NULL,
    to_jsonb(s),
    NULL,
    s.created_at
FROM sales s
LEFT JOIN users u ON LOWER(u.username) = LOWER(s.created_by)
ON CONFLICT (id) DO NOTHING;

INSERT INTO audit_logs (
    id, user_id, username, action, module, target_id,
    description, old_data, new_data, ip_address, created_at
)
SELECT
    'bf_ses_' || LEFT(ps.id, 40),
    u.id,
    COALESCE(NULLIF(TRIM(ps.created_by), ''), 'system'),
    'CREATE',
    'SESSIONS',
    ps.id,
    'Backfill historis sesi tanam (' || ps.session_name || ')',
    NULL,
    to_jsonb(ps),
    NULL,
    ps.created_at
FROM planting_sessions ps
LEFT JOIN users u ON LOWER(u.username) = LOWER(ps.created_by)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- 5. Registrasi Idempotent Grup "Reports" & 3 Modul Pilar 5
--    (Existing DB Only — Fresh-DB Safe Guard mencegah skip seeder.rs)
-- ============================================================================
DO $$
DECLARE
    v_reports_group_id VARCHAR(50);
    v_summary_mod_id   VARCHAR(50);
    v_audit_mod_id     VARCHAR(50);
    v_ai_audit_mod_id  VARCHAR(50);
    r_admin            RECORD;
BEGIN
    IF EXISTS (SELECT 1 FROM roles) THEN
        -- 5.1 Upsert Grup Navigasi "Reports"
        SELECT id INTO v_reports_group_id
        FROM system_module_groups
        WHERE LOWER(TRIM(name)) IN ('reports', 'laporan', 'laporan & audit')
        LIMIT 1;

        IF v_reports_group_id IS NULL THEN
            v_reports_group_id := 'grp_reports';
            INSERT INTO system_module_groups (id, name, icon, is_active, created_by, modify_by)
            VALUES (v_reports_group_id, 'Reports', 'LuBarChart3', TRUE, 'migration-reports-p5', 'migration-reports-p5')
            ON CONFLICT (id) DO NOTHING;
        END IF;

        -- 5.2 Modul /reports/summary
        SELECT id INTO v_summary_mod_id
        FROM system_modules
        WHERE LOWER(TRIM(path)) IN ('/reports/summary', '/laporan', '/reports')
        LIMIT 1;

        IF v_summary_mod_id IS NULL THEN
            v_summary_mod_id := 'mod_reports_summary';
            INSERT INTO system_modules (id, group_id, parent_id, name, path, icon, is_active, created_by, modify_by)
            VALUES (v_summary_mod_id, v_reports_group_id, NULL, 'Laporan Keuangan', '/reports/summary', 'LuFileSpreadsheet', TRUE, 'migration-reports-p5', 'migration-reports-p5')
            ON CONFLICT (id) DO NOTHING;
        ELSE
            UPDATE system_modules
            SET group_id = v_reports_group_id,
                path = '/reports/summary',
                modify_at = CURRENT_TIMESTAMP,
                modify_by = 'migration-reports-p5'
            WHERE id = v_summary_mod_id;
        END IF;

        -- 5.3 Modul /reports/audit
        SELECT id INTO v_audit_mod_id
        FROM system_modules
        WHERE LOWER(TRIM(path)) IN ('/reports/audit', '/audit', '/audit-trail')
        LIMIT 1;

        IF v_audit_mod_id IS NULL THEN
            v_audit_mod_id := 'mod_reports_audit';
            INSERT INTO system_modules (id, group_id, parent_id, name, path, icon, is_active, created_by, modify_by)
            VALUES (v_audit_mod_id, v_reports_group_id, NULL, 'Audit Trail', '/reports/audit', 'LuHistory', TRUE, 'migration-reports-p5', 'migration-reports-p5')
            ON CONFLICT (id) DO NOTHING;
        ELSE
            UPDATE system_modules
            SET group_id = v_reports_group_id,
                path = '/reports/audit',
                modify_at = CURRENT_TIMESTAMP,
                modify_by = 'migration-reports-p5'
            WHERE id = v_audit_mod_id;
        END IF;

        -- 5.4 Modul /reports/ai-audit
        SELECT id INTO v_ai_audit_mod_id
        FROM system_modules
        WHERE LOWER(TRIM(path)) IN ('/reports/ai-audit', '/ai-audit')
        LIMIT 1;

        IF v_ai_audit_mod_id IS NULL THEN
            v_ai_audit_mod_id := 'mod_reports_ai_audit';
            INSERT INTO system_modules (id, group_id, parent_id, name, path, icon, is_active, created_by, modify_by)
            VALUES (v_ai_audit_mod_id, v_reports_group_id, NULL, 'AI Audit', '/reports/ai-audit', 'LuSparkles', TRUE, 'migration-reports-p5', 'migration-reports-p5')
            ON CONFLICT (id) DO NOTHING;
        ELSE
            UPDATE system_modules
            SET group_id = v_reports_group_id,
                path = '/reports/ai-audit',
                modify_at = CURRENT_TIMESTAMP,
                modify_by = 'migration-reports-p5'
            WHERE id = v_ai_audit_mod_id;
        END IF;

        -- 5.5 Grant Full Permissions ke ADMINISTRATOR / SUPER ADMIN
        FOR r_admin IN
            SELECT id FROM roles WHERE UPPER(TRIM(name)) IN ('ADMINISTRATOR', 'SUPER ADMIN')
        LOOP
            INSERT INTO role_module_permissions (
                id, role_id, module_group_id, module_id,
                can_create, can_read, can_update, can_delete, created_by, modify_by
            ) VALUES (
                'perm_rsum_' || LEFT(r_admin.id, 36), r_admin.id, v_reports_group_id, v_summary_mod_id,
                TRUE, TRUE, TRUE, TRUE, 'migration-reports-p5', 'migration-reports-p5'
            ) ON CONFLICT (role_id, module_group_id, module_id) DO NOTHING;

            INSERT INTO role_module_permissions (
                id, role_id, module_group_id, module_id,
                can_create, can_read, can_update, can_delete, created_by, modify_by
            ) VALUES (
                'perm_raud_' || LEFT(r_admin.id, 36), r_admin.id, v_reports_group_id, v_audit_mod_id,
                TRUE, TRUE, TRUE, TRUE, 'migration-reports-p5', 'migration-reports-p5'
            ) ON CONFLICT (role_id, module_group_id, module_id) DO NOTHING;

            INSERT INTO role_module_permissions (
                id, role_id, module_group_id, module_id,
                can_create, can_read, can_update, can_delete, created_by, modify_by
            ) VALUES (
                'perm_raia_' || LEFT(r_admin.id, 36), r_admin.id, v_reports_group_id, v_ai_audit_mod_id,
                TRUE, TRUE, TRUE, TRUE, 'migration-reports-p5', 'migration-reports-p5'
            ) ON CONFLICT (role_id, module_group_id, module_id) DO NOTHING;
        END LOOP;
    END IF;
END $$;
