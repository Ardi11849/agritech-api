-- Up Migration: Cleanup Module Groups, Hierarchy, Module Taxonomy, and Data Quality Fixes
-- PRD: QA & Business Logic Audit Remediation ([MAJOR-03], [MINOR-02], [MINOR-04], [MINOR-05])

DO $$
DECLARE
    v_reports_group_id VARCHAR(50);
    v_report_singular_id VARCHAR(50);
    v_master_group_id VARCHAR(50);
    r_admin RECORD;
BEGIN
    -- Pastikan skema tabel sudah siap sebelum mengeksekusi
    IF EXISTS (
        SELECT 1 FROM information_schema.tables
        WHERE table_schema = 'public' AND table_name = 'system_module_groups'
    ) THEN
        -- 1. Identifikasi / Buat Group 'Reports' (plural) yang standar
        SELECT id INTO v_reports_group_id
        FROM system_module_groups
        WHERE LOWER(TRIM(name)) IN ('reports', 'laporan')
        ORDER BY created_at ASC
        LIMIT 1;

        -- Jika belum ada group Reports, gunakan group default atau buat baru
        IF v_reports_group_id IS NULL THEN
            v_reports_group_id := 'grp_reports';
            INSERT INTO system_module_groups (id, name, icon, is_active, created_by, modify_by)
            VALUES (v_reports_group_id, 'Reports', 'LuBarChart3', TRUE, 'migration-cleanup', 'migration-cleanup')
            ON CONFLICT (id) DO NOTHING;
        ELSE
            UPDATE system_module_groups
            SET name = 'Reports',
                icon = 'LuBarChart3',
                is_active = TRUE,
                modify_at = CURRENT_TIMESTAMP,
                modify_by = 'migration-cleanup'
            WHERE id = v_reports_group_id;
        END IF;

        -- 2. Identifikasi group singular 'Report' yang duplikat/kosong
        FOR v_report_singular_id IN
            SELECT id FROM system_module_groups
            WHERE LOWER(TRIM(name)) = 'report' AND id <> v_reports_group_id
        LOOP
            -- Pindahkan semua modul yang terikat ke group 'Report' lama ke group 'Reports'
            UPDATE system_modules
            SET group_id = v_reports_group_id,
                modify_at = CURRENT_TIMESTAMP,
                modify_by = 'migration-cleanup'
            WHERE group_id = v_report_singular_id;

            -- Pindahkan atau hapus permissions yang mengacu ke group lama
            DELETE FROM role_module_permissions
            WHERE module_group_id = v_report_singular_id;

            -- Hapus group 'Report' lama yang sudah kosong
            DELETE FROM system_module_groups
            WHERE id = v_report_singular_id;
        END LOOP;

        -- 3. Pastikan group 'Master Data' (atau 'Masters') ada untuk modul master
        SELECT id INTO v_master_group_id
        FROM system_module_groups
        WHERE LOWER(TRIM(name)) IN ('master data', 'masters', 'master')
        ORDER BY created_at ASC
        LIMIT 1;

        IF v_master_group_id IS NOT NULL THEN
            -- [MAJOR-03] Pindahkan modul /masters/items ke grup Master Data
            UPDATE system_modules
            SET group_id = v_master_group_id,
                modify_at = CURRENT_TIMESTAMP,
                modify_by = 'migration-cleanup'
            WHERE path = '/masters/items' AND group_id <> v_master_group_id;

            -- Sinkronkan module_group_id pada role_module_permissions untuk /masters/items
            UPDATE role_module_permissions
            SET module_group_id = v_master_group_id,
                modify_at = CURRENT_TIMESTAMP,
                modify_by = 'migration-cleanup'
            WHERE module_id IN (SELECT id FROM system_modules WHERE path = '/masters/items');
        END IF;

        -- 4. [MINOR-02] Standardisasi Nama Modul Sistem (Label Sidebar 5 Pilar)
        UPDATE system_modules SET name = 'Tanaman', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path IN ('/masters/plant', '/masters/plants');
        UPDATE system_modules SET name = 'Lokasi & Lahan', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/masters/locations';
        UPDATE system_modules SET name = 'Unit Instalasi', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/masters/installations';
        UPDATE system_modules SET name = 'Barang & Kategori', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/masters/items';
        UPDATE system_modules SET name = 'Pemasok', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/masters/suppliers';
        UPDATE system_modules SET name = 'Pelanggan', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/masters/customers';
        UPDATE system_modules SET name = 'SOP Kebun', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/plan/sop';
        UPDATE system_modules SET name = 'Tugas & Kanban', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/plan/tasks';
        UPDATE system_modules SET name = 'Sesi Tanam', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/production/sessions';
        UPDATE system_modules SET name = 'Catatan Harian', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/production/daily-readings';
        UPDATE system_modules SET name = 'Gudang & Inventori', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/production/inventory';
        UPDATE system_modules SET name = 'Pembelian', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/finance/purchases';
        UPDATE system_modules SET name = 'Penjualan', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/finance/sales';
        UPDATE system_modules SET name = 'Analisa HPP', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/finance/hpp';
        UPDATE system_modules SET name = 'Perpajakan', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/finance/tax';
        UPDATE system_modules SET name = 'Laporan Keuangan', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/reports/summary';
        UPDATE system_modules SET name = 'Audit Trail', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/reports/audit';
        UPDATE system_modules SET name = 'AI Audit', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/reports/ai-audit';
        UPDATE system_modules SET name = 'Pengguna', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/masters/users';
        UPDATE system_modules SET name = 'Hak Akses', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/masters/roles';
        UPDATE system_modules SET name = 'Modul Sistem', modify_by = 'migration-cleanup', modify_at = CURRENT_TIMESTAMP WHERE path = '/masters/modules';

        -- 5. [MINOR-04] Perbaikan Typo Data Master Unit Instalasi: 'Modue Aeropoik 1' -> 'Module Aeroponik 1'
        IF EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = 'installations'
        ) THEN
            UPDATE installations
            SET name = 'Module Aeroponik 1',
                modify_at = CURRENT_TIMESTAMP,
                modify_by = 'migration-cleanup'
            WHERE name = 'Modue Aeropoik 1';
        END IF;

        -- 6. [MINOR-05] Perapihan Snapshot Nama Pelanggan Kosong / Whitespace di Tabel Sales
        IF EXISTS (
            SELECT 1 FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = 'sales'
        ) THEN
            UPDATE sales
            SET customer_name_snapshot = 'Pelanggan Umum',
                modify_at = CURRENT_TIMESTAMP,
                modify_by = 'migration-cleanup'
            WHERE customer_id IS NULL AND (customer_name_snapshot IS NULL OR TRIM(customer_name_snapshot) = '');
        END IF;

    END IF;
END $$;
