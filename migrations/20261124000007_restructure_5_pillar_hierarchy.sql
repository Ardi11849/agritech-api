-- Migration: Restructure System Module Paths into 5-Pillar Architecture
-- Sesuai PRD Arsitektur 5 Pilar Domain (domain-hierarchy-5-pillars-prd.md)

-- 1. Perbarui nama grup operasional menjadi 'Produksi' jika masih bernama 'Operasional Kebun' atau 'Operasional'
UPDATE system_module_groups
SET name = 'Produksi',
    modify_at = CURRENT_TIMESTAMP,
    modify_by = 'migration-5-pillars'
WHERE name = 'Operasional Kebun' OR name = 'Operasional';

-- 2. Perbarui path pada system_modules untuk Pilar Production (/production/*)
UPDATE system_modules
SET path = '/production/sessions',
    modify_at = CURRENT_TIMESTAMP,
    modify_by = 'migration-5-pillars'
WHERE path IN ('/sessions', '/buat-sesi');

UPDATE system_modules
SET path = '/production/daily-readings',
    modify_at = CURRENT_TIMESTAMP,
    modify_by = 'migration-5-pillars'
WHERE path IN ('/daily-readings', '/dataharian');

UPDATE system_modules
SET path = '/production/inventory',
    modify_at = CURRENT_TIMESTAMP,
    modify_by = 'migration-5-pillars'
WHERE path = '/inventory';
