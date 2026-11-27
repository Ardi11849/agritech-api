-- Up Migration: Planting Sessions & Daily Readings (Operational Farm Tracking)

-- 1. Planting Sessions Table (Siklus Sesi Tanam + Target Parameter Snapshot)
CREATE TABLE IF NOT EXISTS planting_sessions (
    id VARCHAR(50) PRIMARY KEY,
    session_name VARCHAR(150) NOT NULL,
    plant_type_id VARCHAR(50) NOT NULL REFERENCES plant_types(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    location_id VARCHAR(50) NOT NULL REFERENCES locations(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,
    installation_id VARCHAR(50) REFERENCES installations(id)
        ON UPDATE CASCADE ON DELETE RESTRICT,

    -- Jadwal Siklus Tanam
    seeding_date DATE,
    start_date DATE NOT NULL,
    estimated_harvest_date DATE NOT NULL,
    harvest_date DATE,

    -- Populasi & Status Siklus
    total_holes INTEGER NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'Active',
    is_locked BOOLEAN NOT NULL DEFAULT FALSE,

    -- Snapshot Parameter Standar Tanaman (Dibekukan saat Sesi Dibuat)
    target_growth_est_days INTEGER NOT NULL,
    target_veg_phase_days INTEGER NOT NULL DEFAULT 14,
    target_yield_per_hole NUMERIC(10, 2) NOT NULL,

    target_min_ph NUMERIC(4, 2) NOT NULL,
    target_ideal_min_ph NUMERIC(4, 2) NOT NULL,
    target_ideal_max_ph NUMERIC(4, 2) NOT NULL,
    target_max_ph NUMERIC(4, 2) NOT NULL,

    target_ppm_veg_weeks JSONB NOT NULL DEFAULT '[]'::jsonb,
    target_ppm_gen_weeks JSONB NOT NULL DEFAULT '[]'::jsonb,
    target_ideal_min_ppm_veg INTEGER NOT NULL DEFAULT 0,
    target_ideal_max_ppm_veg INTEGER NOT NULL DEFAULT 0,
    target_tol_min_ppm_veg INTEGER NOT NULL DEFAULT 0,
    target_tol_max_ppm_veg INTEGER NOT NULL DEFAULT 0,
    target_ideal_min_ppm_gen INTEGER NOT NULL DEFAULT 0,
    target_ideal_max_ppm_gen INTEGER NOT NULL DEFAULT 0,
    target_tol_min_ppm_gen INTEGER NOT NULL DEFAULT 0,
    target_tol_max_ppm_gen INTEGER NOT NULL DEFAULT 0,

    target_ideal_min_temp_env NUMERIC(5, 2) NOT NULL,
    target_ideal_max_temp_env NUMERIC(5, 2) NOT NULL,
    target_ideal_min_temp_water NUMERIC(5, 2) NOT NULL,
    target_ideal_max_temp_water NUMERIC(5, 2) NOT NULL,
    target_ideal_min_hum NUMERIC(5, 2) NOT NULL,
    target_ideal_max_hum NUMERIC(5, 2) NOT NULL,

    -- Hasil Panen Akhir (Diisi saat complete-harvest)
    actual_yield_kg NUMERIC(12, 2),
    saleable_count INTEGER,
    mortality_count INTEGER NOT NULL DEFAULT 0,
    harvest_notes TEXT NOT NULL DEFAULT '',

    -- Catatan Umum & Audit Metadata
    notes TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) NOT NULL DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) NOT NULL DEFAULT 'system',

    -- Domain Invariants
    CONSTRAINT chk_session_status CHECK (status IN ('Active', 'Completed', 'Cancelled')),
    CONSTRAINT chk_session_total_holes CHECK (total_holes >= 1),
    CONSTRAINT chk_session_dates CHECK (seeding_date IS NULL OR seeding_date <= start_date),
    CONSTRAINT chk_session_est_harvest CHECK (estimated_harvest_date >= start_date),
    CONSTRAINT chk_session_locked_completed CHECK (status <> 'Completed' OR is_locked = TRUE),
    CONSTRAINT chk_session_target_days CHECK (target_growth_est_days >= 1 AND target_veg_phase_days >= 1),
    CONSTRAINT chk_session_target_yield CHECK (target_yield_per_hole > 0.00),
    CONSTRAINT chk_session_harvest_values CHECK (
        (actual_yield_kg IS NULL OR actual_yield_kg >= 0.00)
        AND (saleable_count IS NULL OR saleable_count >= 0)
        AND mortality_count >= 0
    )
);

CREATE INDEX IF NOT EXISTS idx_planting_sessions_status
    ON planting_sessions (status);
CREATE INDEX IF NOT EXISTS idx_planting_sessions_location_id
    ON planting_sessions (location_id);
CREATE INDEX IF NOT EXISTS idx_planting_sessions_plant_type_id
    ON planting_sessions (plant_type_id);
CREATE INDEX IF NOT EXISTS idx_planting_sessions_installation_id
    ON planting_sessions (installation_id) WHERE installation_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_planting_sessions_start_date
    ON planting_sessions (start_date DESC);

-- 2. Daily Readings Table (Log Monitoring Sensor & Mortalitas Harian)
CREATE TABLE IF NOT EXISTS daily_readings (
    id VARCHAR(50) PRIMARY KEY,
    session_id VARCHAR(50) NOT NULL REFERENCES planting_sessions(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    reading_date DATE NOT NULL,
    reading_time TIME NOT NULL DEFAULT CURRENT_TIME,

    -- Parameter Sensor Presisi Decimal
    ppm NUMERIC(8, 2) NOT NULL,
    ph NUMERIC(4, 2) NOT NULL,
    temp_env NUMERIC(5, 2),
    temp_water NUMERIC(5, 2),
    humidity NUMERIC(5, 2),

    -- Mortalitas & Catatan Lapangan
    mortality_count INTEGER NOT NULL DEFAULT 0,
    notes TEXT NOT NULL DEFAULT '',

    -- Audit Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) NOT NULL DEFAULT 'system',

    -- Domain Invariants
    CONSTRAINT chk_daily_reading_ppm CHECK (ppm >= 0.00),
    CONSTRAINT chk_daily_reading_ph CHECK (ph >= 0.00 AND ph <= 14.00),
    CONSTRAINT chk_daily_reading_humidity CHECK (
        humidity IS NULL OR (humidity >= 0.00 AND humidity <= 100.00)
    ),
    CONSTRAINT chk_daily_reading_mortality CHECK (mortality_count >= 0)
);

-- Indeks Kronologis Riwayat Sesi & Deteksi "Belum Lapor Hari Ini"
CREATE INDEX IF NOT EXISTS idx_daily_readings_session_timeline
    ON daily_readings (session_id, reading_date DESC, reading_time DESC, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_daily_readings_date_session
    ON daily_readings (reading_date, session_id);
