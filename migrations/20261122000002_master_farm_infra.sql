-- Up Migration: Master Farm Infrastructure (plant_types, locations, installations)

-- 1. Plant Types Table (Master Tanaman & Parameter Agronomi)
CREATE TABLE IF NOT EXISTS plant_types (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    variety VARCHAR(100) NOT NULL DEFAULT '',
    growth_est_days INTEGER NOT NULL CHECK (growth_est_days >= 1),
    veg_phase_days INTEGER NOT NULL DEFAULT 14 CHECK (veg_phase_days >= 0),
    target_yield_per_hole NUMERIC(10, 2) NOT NULL CHECK (target_yield_per_hole > 0),

    -- pH Boundaries (Scale 0.00 - 14.00, precision 2 decimals)
    min_ph NUMERIC(4, 2) NOT NULL,
    ideal_min_ph NUMERIC(4, 2) NOT NULL,
    ideal_max_ph NUMERIC(4, 2) NOT NULL,
    max_ph NUMERIC(4, 2) NOT NULL,

    -- Weekly PPM Schedule Curves (JSONB Array: [{"min": 560, "max": 700}, ...])
    ideal_ppm_veg_weeks JSONB NOT NULL DEFAULT '[]'::jsonb,
    ideal_ppm_gen_weeks JSONB NOT NULL DEFAULT '[]'::jsonb,

    -- Derived Scalar PPM Summary (Vegetative)
    ideal_min_ppm_veg INTEGER NOT NULL DEFAULT 0 CHECK (ideal_min_ppm_veg >= 0),
    ideal_max_ppm_veg INTEGER NOT NULL DEFAULT 0 CHECK (ideal_max_ppm_veg >= ideal_min_ppm_veg),
    tol_min_ppm_veg INTEGER NOT NULL DEFAULT 0 CHECK (tol_min_ppm_veg >= 0),
    tol_max_ppm_veg INTEGER NOT NULL DEFAULT 0 CHECK (tol_max_ppm_veg >= tol_min_ppm_veg),

    -- Derived Scalar PPM Summary (Generative)
    ideal_min_ppm_gen INTEGER NOT NULL DEFAULT 0 CHECK (ideal_min_ppm_gen >= 0),
    ideal_max_ppm_gen INTEGER NOT NULL DEFAULT 0 CHECK (ideal_max_ppm_gen >= ideal_min_ppm_gen),
    tol_min_ppm_gen INTEGER NOT NULL DEFAULT 0 CHECK (tol_min_ppm_gen >= 0),
    tol_max_ppm_gen INTEGER NOT NULL DEFAULT 0 CHECK (tol_max_ppm_gen >= tol_min_ppm_gen),

    -- Microclimate Parameters (Celsius & Relative Humidity %)
    ideal_min_temp_env NUMERIC(5, 2) NOT NULL,
    ideal_max_temp_env NUMERIC(5, 2) NOT NULL,
    ideal_min_temp_water NUMERIC(5, 2) NOT NULL,
    ideal_max_temp_water NUMERIC(5, 2) NOT NULL,
    ideal_min_hum NUMERIC(5, 2) NOT NULL,
    ideal_max_hum NUMERIC(5, 2) NOT NULL,

    -- Status & Audit Metadata
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) DEFAULT 'system',

    -- Domain Invariants
    CONSTRAINT uq_plant_types_name_variety UNIQUE (name, variety),
    CONSTRAINT chk_plant_ph_order CHECK (
        min_ph >= 0.00
        AND min_ph <= ideal_min_ph
        AND ideal_min_ph <= ideal_max_ph
        AND ideal_max_ph <= max_ph
        AND max_ph <= 14.00
    ),
    CONSTRAINT chk_plant_temp_env CHECK (ideal_min_temp_env <= ideal_max_temp_env),
    CONSTRAINT chk_plant_temp_water CHECK (ideal_min_temp_water <= ideal_max_temp_water),
    CONSTRAINT chk_plant_hum CHECK (
        ideal_min_hum >= 0.00
        AND ideal_min_hum <= ideal_max_hum
        AND ideal_max_hum <= 100.00
    )
);

CREATE INDEX IF NOT EXISTS idx_plant_types_name ON plant_types(name);
CREATE INDEX IF NOT EXISTS idx_plant_types_is_active ON plant_types(is_active);

-- 2. Locations Table (Master Lokasi Kebun / Greenhouse)
CREATE TABLE IF NOT EXISTS locations (
    id VARCHAR(50) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    manager_name VARCHAR(100) NOT NULL DEFAULT '',
    address TEXT NOT NULL DEFAULT '',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) DEFAULT 'system',

    CONSTRAINT uq_locations_name UNIQUE (name)
);

CREATE INDEX IF NOT EXISTS idx_locations_name ON locations(name);
CREATE INDEX IF NOT EXISTS idx_locations_is_active ON locations(is_active);

-- 3. Installations Table (Master Unit Instalasi Tanam)
CREATE TABLE IF NOT EXISTS installations (
    id VARCHAR(50) PRIMARY KEY,
    location_id VARCHAR(50) NOT NULL REFERENCES locations(id) ON UPDATE CASCADE ON DELETE RESTRICT,
    name VARCHAR(100) NOT NULL,
    capacity_holes INTEGER NOT NULL CHECK (capacity_holes >= 1),
    status VARCHAR(20) NOT NULL DEFAULT 'Active' CHECK (status IN ('Active', 'Maintenance', 'Inactive')),
    notes TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) DEFAULT 'system',

    CONSTRAINT uq_installations_location_name UNIQUE (location_id, name)
);

CREATE INDEX IF NOT EXISTS idx_installations_location_id ON installations(location_id);
CREATE INDEX IF NOT EXISTS idx_installations_status ON installations(status);
