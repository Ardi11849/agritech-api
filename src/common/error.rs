use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

use crate::common::response::ApiResponse;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Database Error: {0}")]
    Database(String),

    #[error("Timeout Error: {0}")]
    Timeout(String),

    #[error("Internal Server Error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound(msg) => {
                ApiResponse::error_response(StatusCode::NOT_FOUND, "Not Found", msg)
            }
            AppError::BadRequest(msg) => {
                ApiResponse::error_response(StatusCode::BAD_REQUEST, "Bad Request", msg)
            }
            AppError::Unauthorized(msg) => {
                ApiResponse::error_response(StatusCode::UNAUTHORIZED, "Unauthorized", msg)
            }
            AppError::Forbidden(msg) => {
                ApiResponse::error_response(StatusCode::FORBIDDEN, "Forbidden", msg)
            }
            AppError::Conflict(msg) => {
                ApiResponse::error_response(StatusCode::CONFLICT, "Conflict", msg)
            }
            AppError::Database(msg) => {
                tracing::error!("Database error: {}", msg);
                ApiResponse::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Service Unavailable",
                    "Database connection failed",
                )
            }
            AppError::Timeout(msg) => {
                tracing::error!("Timeout error: {}", msg);
                ApiResponse::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Service Unavailable",
                    "Database connection failed",
                )
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                ApiResponse::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server Error",
                    msg,
                )
            }
        }
    }
}

/// Menerjemahkan pelanggaran Foreign Key (SQLSTATE 23503) menjadi pesan Bahasa Indonesia ramah pengguna.
pub fn translate_foreign_key_violation(constraint: &str, message: &str) -> String {
    let check = |key: &str| constraint.contains(key) || message.contains(key);

    if check("purchase_items_session_id_fkey") {
        "Sesi tanam tidak dapat dihapus karena sudah memiliki riwayat alokasi biaya pada transaksi Pembelian. Hapus atau ubah alokasi nota pembelian terkait terlebih dahulu.".to_string()
    } else if check("sales_session_id_fkey") {
        "Sesi tanam tidak dapat dihapus karena sudah memiliki riwayat transaksi Penjualan hasil panen.".to_string()
    } else if check("daily_readings_session_id_fkey") {
        "Sesi tanam tidak dapat dihapus karena sudah memiliki catatan monitoring sensor harian."
            .to_string()
    } else if check("planting_sessions_plant_type_id_fkey") {
        "Komoditas tanaman tidak dapat dihapus karena masih digunakan pada sesi tanam. Nonaktifkan tanaman sebagai alternatif.".to_string()
    } else if check("planting_sessions_location_id_fkey") {
        "Lokasi kebun tidak dapat dihapus karena masih digunakan pada sesi tanam.".to_string()
    } else if check("planting_sessions_installation_id_fkey") {
        "Unit instalasi tidak dapat dihapus karena masih digunakan pada sesi tanam.".to_string()
    } else if check("installations_location_id_fkey") {
        "Lokasi kebun tidak dapat dihapus karena masih memiliki unit instalasi terdaftar. Pindahkan atau hapus unit instalasi terlebih dahulu.".to_string()
    } else if check("purchase_headers_location_id_fkey") {
        "Lokasi kebun tidak dapat dihapus karena sudah tercatat pada riwayat transaksi Pembelian."
            .to_string()
    } else if check("sales_location_id_fkey") {
        "Lokasi kebun tidak dapat dihapus karena sudah tercatat pada riwayat transaksi Penjualan."
            .to_string()
    } else if check("master_items_category_id_fkey") {
        "Kategori barang tidak dapat dihapus karena masih memiliki katalog barang terdaftar. Pindahkan atau hapus barang terkait terlebih dahulu.".to_string()
    } else if check("purchase_items_item_id_fkey") {
        "Barang tidak dapat dihapus karena sudah tercatat pada riwayat transaksi Pembelian. Nonaktifkan status barang sebagai alternatif.".to_string()
    } else if check("inventory_ledgers_item_id_fkey") {
        "Barang tidak dapat dihapus karena sudah memiliki riwayat mutasi di Kartu Stok Gudang. Nonaktifkan status barang sebagai alternatif.".to_string()
    } else if check("inventory_ledgers_supplier_id_fkey")
        || check("purchase_headers_supplier_id_fkey")
    {
        "Data pemasok tidak dapat dihapus karena sudah memiliki riwayat transaksi pengadaan atau mutasi gudang. Nonaktifkan pemasok sebagai alternatif.".to_string()
    } else if check("inventory_ledgers_customer_id_fkey") || check("sales_customer_id_fkey") {
        "Data pelanggan tidak dapat dihapus karena sudah memiliki riwayat transaksi penjualan atau mutasi gudang. Nonaktifkan pelanggan sebagai alternatif.".to_string()
    } else if check("inventory_ledgers_installation_id_fkey") {
        "Unit instalasi tidak dapat dihapus karena sudah memiliki riwayat pemakaian barang di Kartu Stok Gudang.".to_string()
    } else if check("users_role_id_fkey") || check("role_module_permissions_role_id_fkey") {
        "Peran (Role) tidak dapat dihapus karena masih digunakan oleh akun pengguna aktif."
            .to_string()
    } else if check("system_modules_group_id_fkey")
        || check("role_module_permissions_module_group_id_fkey")
    {
        "Grup navigasi tidak dapat dihapus karena masih memiliki modul sistem di dalamnya."
            .to_string()
    } else if check("system_modules_parent_id_fkey")
        || check("role_module_permissions_module_id_fkey")
    {
        "Modul sistem tidak dapat dihapus karena masih memiliki sub-modul atau terhubung dengan pengaturan hak akses.".to_string()
    } else {
        "Data ini tidak dapat dihapus atau diubah karena masih terhubung dengan transaksi atau data lain di dalam sistem. Nonaktifkan data atau hapus transaksi terkait terlebih dahulu.".to_string()
    }
}

/// Menerjemahkan pelanggaran Unique Constraint (SQLSTATE 23505) menjadi pesan Bahasa Indonesia ramah pengguna.
pub fn translate_unique_violation(constraint: &str, message: &str) -> String {
    let check = |key: &str| constraint.contains(key) || message.contains(key);

    if check("roles_name_key") {
        "Nama peran (Role) tersebut sudah terdaftar. Gunakan nama peran lain.".to_string()
    } else if check("users_username_key") {
        "Username tersebut sudah digunakan oleh akun lain. Silakan pilih username yang berbeda."
            .to_string()
    } else if check("users_email_key") {
        "Alamat email tersebut sudah terdaftar pada akun lain.".to_string()
    } else if check("idx_role_module") {
        "Pengaturan hak akses untuk modul dan peran ini sudah terdaftar.".to_string()
    } else if check("uq_plant_types_name_variety") {
        "Kombinasi nama komoditas dan varietas tanaman tersebut sudah terdaftar di katalog."
            .to_string()
    } else if check("uq_locations_name") {
        "Nama lokasi kebun tersebut sudah terdaftar. Gunakan nama lokasi yang berbeda.".to_string()
    } else if check("uq_installations_location_name") {
        "Nama unit instalasi tersebut sudah digunakan pada lokasi kebun yang dipilih.".to_string()
    } else if check("uq_item_categories_name_lower") {
        "Nama kategori barang tersebut sudah ada. Gunakan nama kategori lain.".to_string()
    } else if check("uq_master_items_name_lower") {
        "Nama barang tersebut sudah terdaftar di katalog master barang.".to_string()
    } else if check("uq_suppliers_name_lower") {
        "Nama pemasok (supplier) tersebut sudah terdaftar di direktori.".to_string()
    } else if check("uq_customers_name_lower") {
        "Nama pelanggan (customer) tersebut sudah terdaftar di direktori.".to_string()
    } else if check("uq_sop_steps_order") {
        "Nomor urut langkah prosedur (SOP) tidak boleh ganda dalam satu dokumen SOP.".to_string()
    } else if check("purchase_headers_invoice_number_key") {
        "Nomor nota pembelian (Invoice) tersebut sudah tercatat di sistem. Gunakan nomor nota yang unik.".to_string()
    } else if check("sales_sale_number_key") {
        "Nomor faktur penjualan tersebut sudah tercatat di sistem. Gunakan nomor faktur yang unik."
            .to_string()
    } else if check("uq_tax_reports_period") {
        "Laporan pajak untuk periode bulan dan tahun tersebut sudah tercatat di sistem.".to_string()
    } else {
        "Data dengan nama, nomor, atau kode tersebut sudah terdaftar di dalam sistem (duplikat)."
            .to_string()
    }
}

/// Menerjemahkan pelanggaran Check Constraint (SQLSTATE 23514) menjadi pesan Bahasa Indonesia ramah pengguna.
pub fn translate_check_violation(constraint: &str, message: &str) -> String {
    let check = |key: &str| constraint.contains(key) || message.contains(key);

    if check("chk_master_items_current_stock") || check("chk_inventory_balances_non_negative") {
        "Jumlah stok barang di gudang tidak mencukupi untuk transaksi ini.".to_string()
    } else if check("chk_session_dates") || check("chk_session_est_harvest") {
        "Urutan tanggal semai, tanggal mulai tanam, atau estimasi panen tidak valid.".to_string()
    } else if check("chk_purchase_headers_discount_le_sub") || check("chk_sales_discount_le_gross")
    {
        "Nilai potongan diskon tidak boleh melebihi subtotal transaksi.".to_string()
    } else {
        "Data yang dimasukkan tidak memenuhi aturan validasi bisnis sistem. Periksa kembali nilai angka dan pilihan status Anda.".to_string()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        if let sqlx::Error::Database(db_err) = &err {
            if let Some(code) = db_err.code() {
                let constraint = db_err.constraint().unwrap_or("");
                let message = db_err.message();

                match code.as_ref() {
                    // SQLSTATE 23505: Unique Constraint Violation
                    "23505" => {
                        return AppError::Conflict(translate_unique_violation(constraint, message));
                    }
                    // SQLSTATE 23503: Foreign Key Constraint Violation
                    "23503" => {
                        return AppError::BadRequest(translate_foreign_key_violation(
                            constraint, message,
                        ));
                    }
                    // SQLSTATE 23514: Check Constraint Violation
                    "23514" => {
                        return AppError::BadRequest(translate_check_violation(
                            constraint, message,
                        ));
                    }
                    // SQLSTATE 23502: Not-Null Violation
                    "23502" => {
                        return AppError::BadRequest(
                            "Terdapat kolom wajib diisi yang masih kosong. Mohon lengkapi seluruh kolom bertanda bintang (*).".to_string(),
                        );
                    }
                    // SQLSTATE 22003: Numeric Overflow
                    "22003" => {
                        return AppError::BadRequest(
                            "Nilai angka yang dimasukkan terlalu besar dan melebihi batas maksimum sistem.".to_string(),
                        );
                    }
                    // SQLSTATE 22001: String Truncation
                    "22001" => {
                        return AppError::BadRequest(
                            "Teks yang dimasukkan terlalu panjang dan melebihi batas maksimum karakter yang diizinkan.".to_string(),
                        );
                    }
                    // SQLSTATE 22P02: Invalid Text Representation
                    "22P02" => {
                        return AppError::BadRequest(
                            "Format data atau angka yang dikirimkan tidak valid.".to_string(),
                        );
                    }
                    _ => {}
                }
            }
        }
        AppError::Database(err.to_string())
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        AppError::BadRequest(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        let not_found = AppError::NotFound("item".into()).into_response();
        assert_eq!(not_found.status(), StatusCode::NOT_FOUND);

        let bad_req = AppError::BadRequest("bad input".into()).into_response();
        assert_eq!(bad_req.status(), StatusCode::BAD_REQUEST);

        let conflict = AppError::Conflict("already exists".into()).into_response();
        assert_eq!(conflict.status(), StatusCode::CONFLICT);

        let unauthorized = AppError::Unauthorized("invalid token".into()).into_response();
        assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

        let forbidden = AppError::Forbidden("access denied".into()).into_response();
        assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_translate_foreign_key_violations() {
        // 1. purchase_items_session_id_fkey
        let msg = translate_foreign_key_violation("purchase_items_session_id_fkey", "");
        assert!(msg.contains("riwayat alokasi biaya pada transaksi Pembelian"));

        // 2. sales_session_id_fkey
        let msg = translate_foreign_key_violation(
            "",
            "violates foreign key constraint \"sales_session_id_fkey\"",
        );
        assert!(msg.contains("riwayat transaksi Penjualan hasil panen"));

        // 3. daily_readings_session_id_fkey
        let msg = translate_foreign_key_violation("daily_readings_session_id_fkey", "");
        assert!(msg.contains("catatan monitoring sensor harian"));

        // 4. installations_location_id_fkey
        let msg = translate_foreign_key_violation("installations_location_id_fkey", "");
        assert!(msg.contains("masih memiliki unit instalasi terdaftar"));

        // 5. master_items_category_id_fkey
        let msg = translate_foreign_key_violation("master_items_category_id_fkey", "");
        assert!(msg.contains("masih memiliki katalog barang terdaftar"));

        // 6. purchase_items_item_id_fkey
        let msg = translate_foreign_key_violation("purchase_items_item_id_fkey", "");
        assert!(msg.contains("tercatat pada riwayat transaksi Pembelian"));

        // 7. inventory_ledgers_item_id_fkey
        let msg = translate_foreign_key_violation("inventory_ledgers_item_id_fkey", "");
        assert!(msg.contains("riwayat mutasi di Kartu Stok Gudang"));

        // 8. users_role_id_fkey
        let msg = translate_foreign_key_violation("users_role_id_fkey", "");
        assert!(msg.contains("Peran (Role) tidak dapat dihapus"));

        // 9. system_modules_group_id_fkey
        let msg = translate_foreign_key_violation("system_modules_group_id_fkey", "");
        assert!(msg.contains("Grup navigasi tidak dapat dihapus"));

        // 10. Fallback FK
        let msg = translate_foreign_key_violation("unknown_fkey", "raw message");
        assert!(msg.contains("Data ini tidak dapat dihapus atau diubah karena masih terhubung"));
        assert!(!msg.contains("Foreign key constraint violation"));
    }

    #[test]
    fn test_translate_unique_violations() {
        let msg = translate_unique_violation("roles_name_key", "");
        assert!(msg.contains("Nama peran (Role) tersebut sudah terdaftar"));

        let msg = translate_unique_violation("users_username_key", "");
        assert!(msg.contains("Username tersebut sudah digunakan"));

        let msg = translate_unique_violation("users_email_key", "");
        assert!(msg.contains("Alamat email tersebut sudah terdaftar"));

        let msg = translate_unique_violation("purchase_headers_invoice_number_key", "");
        assert!(msg.contains("Nomor nota pembelian (Invoice) tersebut sudah tercatat"));

        let msg = translate_unique_violation("sales_sale_number_key", "");
        assert!(msg.contains("Nomor faktur penjualan tersebut sudah tercatat"));

        let msg = translate_unique_violation("uq_tax_reports_period", "");
        assert!(msg.contains("Laporan pajak untuk periode bulan dan tahun tersebut sudah tercatat"));

        // Fallback unique
        let msg = translate_unique_violation("unknown_uq", "");
        assert!(msg.contains("Data dengan nama, nomor, atau kode tersebut sudah terdaftar"));
        assert!(!msg.contains("duplicate key"));
    }

    #[test]
    fn test_translate_check_violations() {
        let msg = translate_check_violation("chk_master_items_current_stock", "");
        assert!(msg.contains("Jumlah stok barang di gudang tidak mencukupi"));

        let msg = translate_check_violation("chk_session_dates", "");
        assert!(msg.contains(
            "Urutan tanggal semai, tanggal mulai tanam, atau estimasi panen tidak valid"
        ));

        let msg = translate_check_violation("chk_sales_discount_le_gross", "");
        assert!(msg.contains("Nilai potongan diskon tidak boleh melebihi subtotal"));

        let msg = translate_check_violation("unknown_check", "");
        assert!(msg.contains("Data yang dimasukkan tidak memenuhi aturan validasi bisnis sistem"));
    }
}
