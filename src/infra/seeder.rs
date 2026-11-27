use sqlx::PgPool;
use uuid::Uuid;

use crate::common::error::AppError;

pub const DEFAULT_ADMIN_PASSWORD: &str = "Admin123!";

/// Membersihkan seluruh data dummy ber-ID '00000000-%' dan 'perm-00000000-%'
/// agar database bersih dari sisa seeder hardcoded lama.
pub async fn clean_legacy_dummy_data(pool: &PgPool) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;

    // 1. Hapus permissions yang berelasi dengan id 'perm-00000000-%' atau entitas ber-ID '00000000-%'
    sqlx::query(
        r#"
        DELETE FROM role_module_permissions
        WHERE id LIKE 'perm-00000000-%'
           OR role_id LIKE '00000000-%'
           OR module_group_id LIKE '00000000-%'
           OR module_id LIKE '00000000-%';
        "#,
    )
    .execute(&mut *tx)
    .await?;

    // 2. Hapus users ber-ID '00000000-%' atau yang berelasi dengan role '00000000-%'
    sqlx::query(
        r#"
        DELETE FROM users
        WHERE id LIKE '00000000-%'
           OR role_id LIKE '00000000-%';
        "#,
    )
    .execute(&mut *tx)
    .await?;

    // 3. Hapus modules ber-ID '00000000-%' atau yang berelasi dengan group '00000000-%'
    sqlx::query(
        r#"
        DELETE FROM system_modules
        WHERE id LIKE '00000000-%'
           OR group_id LIKE '00000000-%';
        "#,
    )
    .execute(&mut *tx)
    .await?;

    // 4. Hapus module groups ber-ID '00000000-%'
    sqlx::query(
        r#"
        DELETE FROM system_module_groups
        WHERE id LIKE '00000000-%';
        "#,
    )
    .execute(&mut *tx)
    .await?;

    // 5. Hapus roles ber-ID '00000000-%'
    sqlx::query(
        r#"
        DELETE FROM roles
        WHERE id LIKE '00000000-%';
        "#,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    tracing::info!("Legacy dummy records with prefix '00000000-%' cleaned successfully");
    Ok(())
}

pub async fn seed_database(pool: &PgPool) -> Result<(), AppError> {
    // 1. Bersihkan sisa data dummy '00000000-%' terlebih dahulu
    clean_legacy_dummy_data(pool).await?;

    // 2. Cek apakah tabel roles, users, atau system_modules sudah memiliki data
    let (role_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM roles")
        .fetch_one(pool)
        .await?;
    let (user_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    let (module_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM system_modules")
        .fetch_one(pool)
        .await?;

    if role_count > 0 || user_count > 0 || module_count > 0 {
        tracing::info!(
            "Database already contains records (roles: {}, users: {}, modules: {}). Skipping seeder.",
            role_count,
            user_count,
            module_count
        );
        return Ok(());
    }

    tracing::info!("Database tables are empty. Seeding initial data with dynamic UUIDs...");
    let mut tx = pool.begin().await?;

    // 1. Seed Role: ADMINISTRATOR
    let admin_role_id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO roles (id, name, is_active, created_by, modify_by)
        VALUES ($1, 'ADMINISTRATOR', true, 'system', 'system');
        "#,
    )
    .bind(&admin_role_id)
    .execute(&mut *tx)
    .await?;

    // 2. Seed Superadmin User: admin (Password Bcrypt of 'Admin123!')
    let admin_user_id = Uuid::new_v4().to_string();
    let password_hash = bcrypt::hash(DEFAULT_ADMIN_PASSWORD, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Failed to hash seed password: {e}")))?;

    sqlx::query(
        r#"
        INSERT INTO users (id, name, email, username, password, role_id, is_active, created_by, modify_by)
        VALUES ($1, 'Administrator AgriTech', 'admin@agritech.com', 'admin', $2, $3, true, 'system', 'system');
        "#,
    )
    .bind(&admin_user_id)
    .bind(&password_hash)
    .bind(&admin_role_id)
    .execute(&mut *tx)
    .await?;

    // 3. Seed Module Groups (Master Data, Plan, Produksi, Finance, Manajemen Sistem)
    let group_master_data_id = Uuid::new_v4().to_string();
    let group_plan_id = Uuid::new_v4().to_string();
    let group_ops_id = Uuid::new_v4().to_string();
    let group_finance_id = Uuid::new_v4().to_string();
    let group_sys_id = Uuid::new_v4().to_string();

    let groups = [
        (&group_master_data_id, "Master Data", "LuDatabase"),
        (&group_plan_id, "Plan", "LuClipboardCheck"),
        (&group_ops_id, "Produksi", "LuActivity"),
        (&group_finance_id, "Finance", "LuWallet"),
        (&group_sys_id, "Manajemen Sistem", "LuSettings"),
    ];

    for (group_id, name, icon) in groups {
        sqlx::query(
            r#"
            INSERT INTO system_module_groups (id, name, icon, is_active, created_by, modify_by)
            VALUES ($1, $2, $3, true, 'system', 'system');
            "#,
        )
        .bind(group_id)
        .bind(name)
        .bind(icon)
        .execute(&mut *tx)
        .await?;
    }

    // 4. Seed Modules
    let modules = [
        (
            &group_master_data_id,
            "Tanaman",
            "/masters/plant",
            "LuSprout",
        ),
        (
            &group_master_data_id,
            "Lokasi & Lahan",
            "/masters/locations",
            "LuMapPin",
        ),
        (
            &group_master_data_id,
            "Unit Instalasi",
            "/masters/installations",
            "LuLayers",
        ),
        (
            &group_master_data_id,
            "Barang & Kategori",
            "/masters/items",
            "LuPackage",
        ),
        (
            &group_master_data_id,
            "Pemasok",
            "/masters/suppliers",
            "LuTruck",
        ),
        (
            &group_master_data_id,
            "Pelanggan",
            "/masters/customers",
            "LuUserCheck",
        ),
        (&group_plan_id, "SOP Kebun", "/plan/sop", "LuBookOpen"),
        (&group_plan_id, "Tugas & Kanban", "/plan/tasks", "LuKanban"),
        (
            &group_ops_id,
            "Sesi Tanam",
            "/production/sessions",
            "LuCalendar",
        ),
        (
            &group_ops_id,
            "Catatan Harian",
            "/production/daily-readings",
            "LuClipboardList",
        ),
        (
            &group_ops_id,
            "Gudang & Inventori",
            "/production/inventory",
            "LuWarehouse",
        ),
        (
            &group_finance_id,
            "Pembelian",
            "/finance/purchases",
            "LuShoppingCart",
        ),
        (
            &group_finance_id,
            "Penjualan",
            "/finance/sales",
            "LuBanknote",
        ),
        (
            &group_finance_id,
            "Analisa HPP",
            "/finance/hpp",
            "LuCalculator",
        ),
        (&group_finance_id, "Perpajakan", "/finance/tax", "LuReceipt"),
        (&group_sys_id, "Pengguna", "/masters/users", "LuUsers"),
        (&group_sys_id, "Hak Akses", "/masters/roles", "LuShield"),
        (
            &group_sys_id,
            "Modul Sistem",
            "/masters/modules",
            "LuLayers",
        ),
    ];

    for (group_id, name, path, icon) in modules {
        let mod_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO system_modules (id, group_id, parent_id, name, path, icon, is_active, created_by, modify_by)
            VALUES ($1, $2, NULL, $3, $4, $5, true, 'system', 'system');
            "#,
        )
        .bind(&mod_id)
        .bind(group_id)
        .bind(name)
        .bind(path)
        .bind(icon)
        .execute(&mut *tx)
        .await?;

        // 5. Seed Permissions for ADMINISTRATOR
        let perm_id = format!("perm-{}", Uuid::new_v4());
        sqlx::query(
            r#"
            INSERT INTO role_module_permissions (id, role_id, module_group_id, module_id, can_create, can_read, can_update, can_delete, created_by, modify_by)
            VALUES ($1, $2, $3, $4, true, true, true, true, 'system', 'system');
            "#,
        )
        .bind(&perm_id)
        .bind(&admin_role_id)
        .bind(group_id)
        .bind(&mod_id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    tracing::info!("Initial database seeding completed successfully with dynamic UUIDs");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_password_verification() {
        let hash = bcrypt::hash(DEFAULT_ADMIN_PASSWORD, bcrypt::DEFAULT_COST)
            .expect("Hash password must succeed");
        assert!(bcrypt::verify(DEFAULT_ADMIN_PASSWORD, &hash).expect("Verification must succeed"));
    }

    #[test]
    fn test_uuid_generation_format() {
        let id = Uuid::new_v4().to_string();
        assert_eq!(id.len(), 36);
        assert!(!id.starts_with("00000000-"));
    }
}
