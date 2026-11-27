# ⚙️ AgriTech API — High-Performance Rust REST Backend

> **Enterprise-Grade RESTful API untuk AgriTech Smart Farming Ecosystem**  
> Dibangun dengan **Rust (Axum + SQLx)**, **PostgreSQL**, presisi mutlak **`rust_decimal`**, dan dokumentasi interaktif **OpenAPI / Swagger UI**.

Untuk dokumentasi lengkap sistem dan arsitektur monorepo, silakan lihat [README.md Utama di Root Proyek](../README.md).

---

## 📑 Daftar Isi

- [Arsitektur & Struktur Direktori Backend](#-arsitektur--struktur-direktori-backend)
- [Pilar Domain API](#-pilar-domain-api)
- [Prinsip Desain & Integritas Data](#-prinsip-desain--integritas-data)
- [Panduan Menjalankan API](#-panduan-menjalankan-api)
- [Dokumentasi Swagger UI](#-dokumentasi-swagger-ui)
- [Pengujian & Verifikasi Mutu](#-pengujian--verifikasi-mutu)

---

## 📂 Arsitektur & Struktur Direktori Backend

Kode backend menerapkan arsitektur modular berlapis (*Layered Domain-Driven Design*):

```text
agritech-api/
├── migrations/                 # Skrip migrasi DDL SQLx PostgreSQL
│   ├── 20261122000001_initial_schema.sql
│   ├── ...
│   ├── 20261125000009_finance_purchases_and_sales.sql
│   └── 20261126000011_finance_hpp_and_tax.sql
│
├── src/
│   ├── common/                 # Modul utilitas bersama
│   │   ├── extractor.rs        # AppJson wrapper untuk parsing request body
│   │   ├── error.rs            # AppError & JSON error envelope seragam
│   │   ├── precision.rs        # Helper pembulatan desimal moneter
│   │   └── time.rs             # Helper waktu WIB (UTC+7)
│   ├── docs/                   # Definisi Utoipa OpenAPI Specification
│   ├── infra/                  # Database pool, migrator, dan database seeder
│   ├── middleware/             # Verifikasi token JWT & dynamic RBAC guard
│   ├── modules/                # 5 Pilar Domain Modules
│   │   ├── auth/               # Autentikasi JWT, rotasi token, user profile
│   │   ├── masters/            # Tanaman, varietas, lokasi, barang, supplier, user, role
│   │   ├── plan/               # SOP agronomi & manajemen task Kanban
│   │   ├── production/         # Sesi tanam, IoT telemetry daily readings, stok gudang
│   │   ├── finance/            # Pembelian, penjualan panen, engine HPP, pajak UMKM
│   │   └── reports/            # Laporan ringkasan & audit log
│   ├── config.rs               # Konfigurasi aplikasi dari environment variable
│   ├── lib.rs                  # Registrasi router & application state builder
│   └── main.rs                 # Binary entrypoint & listener server Axum
│
├── tests/                      # Suite pengujian integrasi (Black-box HTTP testing)
├── Cargo.toml                  # Dependensi crate & metadata proyek
└── .env.example                # Template variabel lingkungan
```

Setiap modul di bawah `src/modules/<nama_modul>/` memiliki pemisahan tanggung jawab yang konsisten:
- `dto.rs`: Struktur data request dan response JSON.
- `repository.rs`: Interaksi query SQL murni dengan PostgreSQL via SQLx.
- `service.rs`: Logika bisnis, validasi, dan komputasi deterministik.
- `handler.rs`: HTTP handlers pemroses request & perakit HTTP status.
- `router.rs`: Pendaftaran rute dan middleware pengamanan endpoint.

---

## 🏛️ Pilar Domain API

1. **Auth & RBAC (`/api/v1/auth`):** Login JWT stateless, pendaftaran pengguna, validasi sesi, dan pemetaan hak akses role per metode HTTP (`can_read`, `can_create`, `can_update`, `can_delete`).
2. **Masters (`/api/v1/masters`):** Katalog tanaman, lokasi kebun, unit meja tanam, barang saprotan, vendor, dan pelanggan.
3. **Plan (`/api/v1/plan`):** Repositori SOP budidaya berversi dan papan tugas Kanban.
4. **Production (`/api/v1/production`):** Pelacakan sesi tanam, pembacaan parameter nutrisi/sensor harian (PPM, pH, Suhu), dan kartu stok gudang.
5. **Finance (`/api/v1/finance`):**
   - `/purchases`: Faktur pembelian multi-item dengan auto-sync stok & moving average cost.
   - `/sales`: Pencatatan penjualan panen, tracking piutang, dan subsidi ongkir.
   - `/hpp`: Mesin alokasi biaya HPP prorata lubang tanam harian & penguncian snapshot.
   - `/tax`: Otomasi PPh Final UMKM 0,5% dari omzet realized & tracking jatuh tempo tgl 15.
6. **Reports (`/api/v1/reports`):** Laporan analitik eksekutif.

---

## 🛡️ Prinsip Desain & Integritas Data

- **Anti-Floating Point Drift:** Seluruh operasi moneter, yield panen, dan pajak wajib menggunakan `rust_decimal::Decimal` (kolom PostgreSQL `NUMERIC(14, 2)` & `NUMERIC(6, 4)`).
- **Fail-Closed Security:** Endpoint yang tidak terdaftar dalam whitelist publik secara otomatis dilindungi middleware RBAC (`401 Unauthorized` / `403 Forbidden`).
- **Standard JSON Envelope:** Seluruh response menggunakan format seragam:
  ```json
  {
    "status": 200,
    "message": "Deskripsi hasil operasi",
    "data": {},
    "error": null
  }
  ```

---

## 🚀 Panduan Menjalankan API

1. **Masuk ke folder `agritech-api`:**
   ```bash
   cd agritech-api
   ```

2. **Buat file `.env`:**
   ```bash
   cp .env.example .env
   ```
   *Sesuaikan parameter koneksi database:*
   ```env
   PORT=8080
   ENV=development
   DB_HOST=localhost
   DB_PORT=5432
   DB_USER=postgres
   DB_PASSWORD=your_password
   DB_NAME=agritech_db
   JWT_SECRET=super_secret_jwt_key_at_least_32_characters_long
   JWT_EXPIRATION_HOURS=24
   CORS_ORIGIN=http://localhost:3000
   ENABLE_SWAGGER=true
   ```

3. **Jalankan Server:**
   ```bash
   cargo run
   ```
   *Server akan otomatis menjalankan migrasi SQLx dan database seeder.*

---

## 📖 Dokumentasi Swagger UI

Dokumentasi interaktif OpenAPI dapat diakses langsung pada peramban:

- **Swagger UI:** `http://localhost:8080/swagger-ui`
- **OpenAPI JSON Spec:** `http://localhost:8080/api-docs/openapi.json`

---

## 🧪 Pengujian & Verifikasi Mutu

```bash
# 1. Pengecekan formatting kode
cargo fmt --check

# 2. Linter ketat tanpa toleransi warning
cargo clippy --all-targets -- -D warnings

# 3. Menjalankan seluruh test suite (Unit & Integration Tests)
cargo test
```
