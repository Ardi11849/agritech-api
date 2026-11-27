-- Up Migration: Pillar 2 (Plan) — Standard Operating Procedures & Farm Kanban Tasks

-- 1. SOP Master Table
CREATE TABLE IF NOT EXISTS sops (
    id VARCHAR(50) PRIMARY KEY,
    title VARCHAR(150) NOT NULL,
    category VARCHAR(50) NOT NULL DEFAULT 'Umum',
    description TEXT NOT NULL DEFAULT '',
    version INTEGER NOT NULL DEFAULT 1,
    status VARCHAR(20) NOT NULL DEFAULT 'Draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) NOT NULL DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) NOT NULL DEFAULT 'system',

    CONSTRAINT chk_sops_title_nonempty CHECK (TRIM(title) <> ''),
    CONSTRAINT chk_sops_category_valid CHECK (
        category IN ('Nutrisi', 'Hama', 'Panen', 'Maintenance', 'Pembibitan', 'Umum')
    ),
    CONSTRAINT chk_sops_status_valid CHECK (status IN ('Draft', 'Published', 'Archived')),
    CONSTRAINT chk_sops_version_positive CHECK (version >= 1)
);

CREATE INDEX IF NOT EXISTS idx_sops_category ON sops (category);
CREATE INDEX IF NOT EXISTS idx_sops_status ON sops (status);
CREATE INDEX IF NOT EXISTS idx_sops_modify_at ON sops (modify_at DESC);

-- 2. SOP Steps Detail Table (Ordered Master-Detail, ON DELETE CASCADE)
CREATE TABLE IF NOT EXISTS sop_steps (
    id VARCHAR(50) PRIMARY KEY,
    sop_id VARCHAR(50) NOT NULL REFERENCES sops(id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    step_number INTEGER NOT NULL,
    instruction TEXT NOT NULL,
    notes TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT uq_sop_steps_order UNIQUE (sop_id, step_number),
    CONSTRAINT chk_sop_steps_number_positive CHECK (step_number >= 1),
    CONSTRAINT chk_sop_steps_instruction_nonempty CHECK (TRIM(instruction) <> '')
);

CREATE INDEX IF NOT EXISTS idx_sop_steps_sop_id_order
    ON sop_steps (sop_id, step_number ASC);

-- 3. Farm Tasks Table (6-Stage Agile Kanban Board)
CREATE TABLE IF NOT EXISTS farm_tasks (
    id VARCHAR(50) PRIMARY KEY,
    title VARCHAR(150) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status VARCHAR(30) NOT NULL DEFAULT 'Open',
    priority VARCHAR(20) NOT NULL DEFAULT 'Medium',
    story_points INTEGER NOT NULL DEFAULT 0,
    due_date DATE,

    -- Optional Foreign Keys (Audit & Board tetap utuh jika referensi dihapus)
    assigned_to VARCHAR(50) REFERENCES users(id)
        ON UPDATE CASCADE ON DELETE SET NULL,
    session_id VARCHAR(50) REFERENCES planting_sessions(id)
        ON UPDATE CASCADE ON DELETE SET NULL,
    sop_id VARCHAR(50) REFERENCES sops(id)
        ON UPDATE CASCADE ON DELETE SET NULL,

    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) NOT NULL DEFAULT 'system',
    modify_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modify_by VARCHAR(100) NOT NULL DEFAULT 'system',

    CONSTRAINT chk_farm_tasks_title_nonempty CHECK (TRIM(title) <> ''),
    CONSTRAINT chk_farm_tasks_status_valid CHECK (
        status IN ('Open', 'Set Story Point', 'In Progress', 'Resolve', 'In Review', 'Complete')
    ),
    CONSTRAINT chk_farm_tasks_priority_valid CHECK (priority IN ('Low', 'Medium', 'High')),
    CONSTRAINT chk_farm_tasks_story_points CHECK (story_points >= 0 AND story_points <= 100)
);

CREATE INDEX IF NOT EXISTS idx_farm_tasks_status ON farm_tasks (status);
CREATE INDEX IF NOT EXISTS idx_farm_tasks_priority ON farm_tasks (priority);
CREATE INDEX IF NOT EXISTS idx_farm_tasks_due_date ON farm_tasks (due_date);
CREATE INDEX IF NOT EXISTS idx_farm_tasks_assigned_to
    ON farm_tasks (assigned_to) WHERE assigned_to IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_farm_tasks_session_id
    ON farm_tasks (session_id) WHERE session_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_farm_tasks_sop_id
    ON farm_tasks (sop_id) WHERE sop_id IS NOT NULL;

-- 4. Registrasi Idempotent Grup Navigasi "Plan" & Modul /plan/sop, /plan/tasks (Hanya pada Existing DB)
DO $$
DECLARE
    v_plan_group_id VARCHAR(50);
    v_sop_mod_id VARCHAR(50);
    v_tasks_mod_id VARCHAR(50);
    r_admin RECORD;
BEGIN
    -- Guard: Jalankan hanya bila database sudah memiliki role (mencegah skip seeder pada fresh DB)
    IF EXISTS (SELECT 1 FROM roles) THEN
        -- 4.1 Upsert Grup Navigasi "Plan"
        SELECT id INTO v_plan_group_id
        FROM system_module_groups
        WHERE LOWER(TRIM(name)) = 'plan'
        LIMIT 1;

        IF v_plan_group_id IS NULL THEN
            v_plan_group_id := 'grp_plan';
            INSERT INTO system_module_groups (id, name, icon, is_active, created_by, modify_by)
            VALUES (v_plan_group_id, 'Plan', 'LuClipboardCheck', TRUE, 'migration-plan-pillar', 'migration-plan-pillar')
            ON CONFLICT (id) DO NOTHING;
        END IF;

        -- 4.2 Upsert Modul "/plan/sop"
        SELECT id INTO v_sop_mod_id
        FROM system_modules
        WHERE LOWER(TRIM(path)) IN ('/plan/sop', '/sop')
        LIMIT 1;

        IF v_sop_mod_id IS NULL THEN
            v_sop_mod_id := 'mod_plan_sop';
            INSERT INTO system_modules (id, group_id, parent_id, name, path, icon, is_active, created_by, modify_by)
            VALUES (v_sop_mod_id, v_plan_group_id, NULL, 'SOP Kebun', '/plan/sop', 'LuBookOpen', TRUE, 'migration-plan-pillar', 'migration-plan-pillar')
            ON CONFLICT (id) DO NOTHING;
        ELSE
            UPDATE system_modules
            SET group_id = v_plan_group_id,
                path = '/plan/sop',
                modify_at = CURRENT_TIMESTAMP,
                modify_by = 'migration-plan-pillar'
            WHERE id = v_sop_mod_id;
        END IF;

        -- 4.3 Upsert Modul "/plan/tasks"
        SELECT id INTO v_tasks_mod_id
        FROM system_modules
        WHERE LOWER(TRIM(path)) IN ('/plan/tasks', '/tasks')
        LIMIT 1;

        IF v_tasks_mod_id IS NULL THEN
            v_tasks_mod_id := 'mod_plan_tasks';
            INSERT INTO system_modules (id, group_id, parent_id, name, path, icon, is_active, created_by, modify_by)
            VALUES (v_tasks_mod_id, v_plan_group_id, NULL, 'Tugas & Kanban', '/plan/tasks', 'LuKanban', TRUE, 'migration-plan-pillar', 'migration-plan-pillar')
            ON CONFLICT (id) DO NOTHING;
        ELSE
            UPDATE system_modules
            SET group_id = v_plan_group_id,
                path = '/plan/tasks',
                modify_at = CURRENT_TIMESTAMP,
                modify_by = 'migration-plan-pillar'
            WHERE id = v_tasks_mod_id;
        END IF;

        -- 4.4 Berikan Hak Akses Penuh ke Role ADMINISTRATOR / SUPER ADMIN
        FOR r_admin IN
            SELECT id FROM roles WHERE UPPER(TRIM(name)) IN ('ADMINISTRATOR', 'SUPER ADMIN')
        LOOP
            INSERT INTO role_module_permissions (
                id, role_id, module_group_id, module_id,
                can_create, can_read, can_update, can_delete, created_by, modify_by
            ) VALUES (
                'perm_sop_' || LEFT(r_admin.id, 36), r_admin.id, v_plan_group_id, v_sop_mod_id,
                TRUE, TRUE, TRUE, TRUE, 'migration-plan-pillar', 'migration-plan-pillar'
            ) ON CONFLICT (role_id, module_group_id, module_id) DO NOTHING;

            INSERT INTO role_module_permissions (
                id, role_id, module_group_id, module_id,
                can_create, can_read, can_update, can_delete, created_by, modify_by
            ) VALUES (
                'perm_tsk_' || LEFT(r_admin.id, 36), r_admin.id, v_plan_group_id, v_tasks_mod_id,
                TRUE, TRUE, TRUE, TRUE, 'migration-plan-pillar', 'migration-plan-pillar'
            ) ON CONFLICT (role_id, module_group_id, module_id) DO NOTHING;
        END LOOP;
    END IF;
END $$;
