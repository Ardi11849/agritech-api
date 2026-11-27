-- Migration: Cleanup QA Runtime Test Artifacts and Reset Test Session
-- Sesuai PRD Catatan Transparansi QA dan Temuan Audit full-app-qa.md

-- 1. Kembalikan sesi testing utama ke status Active (sebelum complete-harvest QA)
UPDATE planting_sessions
SET
    status = 'Active',
    is_locked = FALSE,
    actual_yield_kg = NULL,
    harvest_date = NULL,
    saleable_count = NULL,
    mortality_count = 0,
    harvest_notes = '',
    modify_at = CURRENT_TIMESTAMP,
    modify_by = 'system-cleanup'
WHERE id = 'ses_d575ae1c-0efb-48cd-a46e-682bc2f1afbb';

-- 2. Bersihkan catatan harian uji QA (tabel: daily_readings)
DELETE FROM daily_readings
WHERE session_id IN (
    'ses_6bd2a6e9-74ab-4e03-b8e2-c3131b5a1108',
    'ses_c2f0fddc-2cec-43ac-ba65-51c0aee9f819'
) OR session_id IN (
    SELECT id FROM planting_sessions
    WHERE LOWER(session_name) LIKE 'qa%'
       OR session_name LIKE '[QA-TEMP]%'
       OR session_name = ''
       OR TRIM(session_name) = ''
       OR plant_type_id IN (
           SELECT id FROM plant_types
           WHERE LOWER(name) LIKE 'qa%'
              OR name LIKE '[QA-TEMP]%'
              OR name = ''
              OR TRIM(name) = ''
       )
       OR location_id IN (
           SELECT id FROM locations
           WHERE LOWER(name) LIKE 'qa%'
              OR name LIKE '[QA-TEMP]%'
              OR name = ''
              OR TRIM(name) = ''
       )
       OR installation_id IN (
           SELECT id FROM installations
           WHERE LOWER(name) LIKE 'qa%'
              OR name LIKE '[QA-TEMP]%'
              OR name = ''
              OR TRIM(name) = ''
              OR location_id IN (
                  SELECT id FROM locations
                  WHERE LOWER(name) LIKE 'qa%'
                     OR name LIKE '[QA-TEMP]%'
                     OR name = ''
                     OR TRIM(name) = ''
              )
       )
);

-- 3. Bersihkan sesi tanam uji QA (tabel: planting_sessions)
DELETE FROM planting_sessions
WHERE id IN (
    'ses_6bd2a6e9-74ab-4e03-b8e2-c3131b5a1108',
    'ses_c2f0fddc-2cec-43ac-ba65-51c0aee9f819'
)
   OR LOWER(session_name) LIKE 'qa%'
   OR session_name LIKE '[QA-TEMP]%'
   OR session_name = ''
   OR TRIM(session_name) = ''
   OR plant_type_id IN (
       SELECT id FROM plant_types
       WHERE LOWER(name) LIKE 'qa%'
          OR name LIKE '[QA-TEMP]%'
          OR name = ''
          OR TRIM(name) = ''
   )
   OR location_id IN (
       SELECT id FROM locations
       WHERE LOWER(name) LIKE 'qa%'
          OR name LIKE '[QA-TEMP]%'
          OR name = ''
          OR TRIM(name) = ''
   )
   OR installation_id IN (
       SELECT id FROM installations
       WHERE LOWER(name) LIKE 'qa%'
          OR name LIKE '[QA-TEMP]%'
          OR name = ''
          OR TRIM(name) = ''
          OR location_id IN (
              SELECT id FROM locations
              WHERE LOWER(name) LIKE 'qa%'
                 OR name LIKE '[QA-TEMP]%'
                 OR name = ''
                 OR TRIM(name) = ''
          )
   );

-- 4. Bersihkan master unit instalasi uji QA (tabel: installations)
DELETE FROM installations
WHERE location_id IN (
    SELECT id FROM locations
    WHERE LOWER(name) LIKE 'qa%'
       OR name LIKE '[QA-TEMP]%'
       OR name = ''
       OR TRIM(name) = ''
)
   OR LOWER(name) LIKE 'qa%'
   OR name LIKE '[QA-TEMP]%'
   OR name = ''
   OR TRIM(name) = '';

-- 5. Bersihkan master lokasi kebun uji QA (tabel: locations)
DELETE FROM locations
WHERE LOWER(name) LIKE 'qa%'
   OR name LIKE '[QA-TEMP]%'
   OR name = ''
   OR TRIM(name) = '';

-- 6. Bersihkan master tanaman uji QA (tabel: plant_types)
DELETE FROM plant_types
WHERE LOWER(name) LIKE 'qa%'
   OR name LIKE '[QA-TEMP]%'
   OR name = ''
   OR TRIM(name) = '';

-- 7. Bersihkan buku besar mutasi inventori uji QA (tabel: inventory_ledgers)
DELETE FROM inventory_ledgers
WHERE LOWER(notes) LIKE 'qa%'
   OR notes LIKE '[QA-TEMP]%'
   OR item_id IN (
       SELECT id FROM master_items
       WHERE category_id IN (
           SELECT id FROM item_categories
           WHERE LOWER(name) LIKE 'qa%'
              OR name LIKE '[QA-TEMP]%'
              OR name = ''
              OR TRIM(name) = ''
       )
          OR LOWER(name) LIKE 'qa%'
          OR name LIKE '[QA-TEMP]%'
          OR name = ''
          OR TRIM(name) = ''
   )
   OR supplier_id IN (
       SELECT id FROM suppliers
       WHERE LOWER(name) LIKE 'qa%'
          OR name LIKE '[QA-TEMP]%'
          OR name = ''
          OR TRIM(name) = ''
   )
   OR customer_id IN (
       SELECT id FROM customers
       WHERE LOWER(name) LIKE 'qa%'
          OR name LIKE '[QA-TEMP]%'
          OR name = ''
          OR TRIM(name) = ''
   );

-- 8. Bersihkan master barang uji QA (tabel: master_items)
DELETE FROM master_items
WHERE category_id IN (
    SELECT id FROM item_categories
    WHERE LOWER(name) LIKE 'qa%'
       OR name LIKE '[QA-TEMP]%'
       OR name = ''
       OR TRIM(name) = ''
)
   OR LOWER(name) LIKE 'qa%'
   OR name LIKE '[QA-TEMP]%'
   OR name = ''
   OR TRIM(name) = '';

-- 9. Bersihkan master kategori barang uji QA (tabel: item_categories)
DELETE FROM item_categories
WHERE LOWER(name) LIKE 'qa%'
   OR name LIKE '[QA-TEMP]%'
   OR name = ''
   OR TRIM(name) = '';

-- 10. Bersihkan pemasok dan pelanggan uji QA (tabel: suppliers, customers)
DELETE FROM suppliers
WHERE LOWER(name) LIKE 'qa%'
   OR name LIKE '[QA-TEMP]%'
   OR name = ''
   OR TRIM(name) = '';

DELETE FROM customers
WHERE LOWER(name) LIKE 'qa%'
   OR name LIKE '[QA-TEMP]%'
   OR name = ''
   OR TRIM(name) = '';

-- 11. Bersihkan user residu uji privilege escalation / RBAC breach (tabel: users)
DELETE FROM users
WHERE (
    role_id IN (
        SELECT id FROM roles
        WHERE LOWER(name) LIKE 'qa%'
           OR name LIKE '[QA-TEMP]%'
           OR UPPER(name) LIKE 'RBAC_BREACH%'
           OR name = ''
           OR TRIM(name) = ''
    )
    OR LOWER(username) LIKE 'qa%'
    OR LOWER(username) LIKE 'bad user%'
    OR UPPER(username) LIKE 'RBAC_BREACH%'
    OR LOWER(name) LIKE 'qa%'
    OR name LIKE '[QA-TEMP]%'
    OR username = ''
    OR TRIM(username) = ''
)
AND LOWER(username) NOT IN ('admin', 'ardi')
AND id NOT IN ('00000000-0000-0000-0000-000000000002');

-- 12. Bersihkan matriks perizinan role uji QA (tabel: role_module_permissions)
DELETE FROM role_module_permissions
WHERE role_id IN (
    SELECT id FROM roles
    WHERE LOWER(name) LIKE 'qa%'
       OR name LIKE '[QA-TEMP]%'
       OR UPPER(name) LIKE 'RBAC_BREACH%'
       OR name = ''
       OR TRIM(name) = ''
);

-- 13. Bersihkan role uji QA (tabel: roles)
DELETE FROM roles
WHERE (
    LOWER(name) LIKE 'qa%'
    OR name LIKE '[QA-TEMP]%'
    OR UPPER(name) LIKE 'RBAC_BREACH%'
    OR name = ''
    OR TRIM(name) = ''
)
AND LOWER(name) NOT IN ('super admin', 'administrator')
AND id NOT IN ('00000000-0000-0000-0000-000000000001');

-- 14. Bersihkan modul dan grup uji QA jika ada (tabel: system_modules, system_module_groups)
DELETE FROM system_modules
WHERE (
    LOWER(name) LIKE 'qa%'
    OR name LIKE '[QA-TEMP]%'
    OR LOWER(path) LIKE '%qaprivesc%'
    OR name = ''
    OR TRIM(name) = ''
)
AND path NOT IN (
    '/dashboard',
    '/masters/users',
    '/masters/roles',
    '/masters/modules',
    '/masters/plant',
    '/masters/locations',
    '/masters/installations',
    '/masters/items',
    '/masters/suppliers',
    '/masters/customers',
    '/inventory',
    '/sessions',
    '/daily-readings'
);

DELETE FROM system_module_groups
WHERE LOWER(name) LIKE 'qa%'
   OR name LIKE '[QA-TEMP]%'
   OR name = ''
   OR TRIM(name) = '';
