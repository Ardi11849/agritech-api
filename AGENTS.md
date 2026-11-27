# Project Rules: Agritech API

## Tech Stack
- Bahasa: Rust
- Framework: Axum 0.7, Tokio, SQLx (PostgreSQL), Utoipa (OpenAPI/Swagger UI)
- Arsitektur: Modular (pisahkan handler, service, repository, dan model per fitur)

## Development & Hot Reload (`cargo-watch`)
Untuk menjalankan server dengan auto-reload setiap kali kode berubah:
```bash
# Menjalankan server Axum dengan hot-reload
cargo watch -x run

# Menjalankan server dengan clear terminal setiap reload
cargo watch -c -x run

# Menjalankan continuous testing saat file berubah
cargo watch -x test
```
*Catatan: Task VS Code `Cargo Watch: Run Backend (Hot Reload)` juga telah dikonfigurasi di workspace untuk eksekusi 1-klik via `Ctrl + Shift + B`.*

## Aturan Wajib (DO)
- WAJIB gunakan tipe data desimal/presisi tinggi untuk semua perhitungan uang, berat, dan rumus agronomi (hindari f32/f64 untuk angka finansial).
- WAJIB tangani semua error dengan eksplisit (`Result<T, E>`).
- WAJIB buat unit test untuk setiap fungsi kalkulasi di layer service.
- Jika ada code berulang wajib membuat helper untuk di pakai di lain waktu.
- Wajib menambahkan comment di code penting.
- Wajib membuat Docs.

## Larangan (DON'T)
- DILARANG menggunakan `.unwrap()` atau `.expect()` di kode production.
- DILARANG mengubah struktur database tanpa membuat file migrasi baru.
- DILARANG mengedit file di luar folder `agritech-api/`.
- DILARANG membaca atau mencari file di `agritech-frontend/`.

## Checklist Sebelum Selesai
Sebelum menyatakan tugas selesai, WAJIB jalankan perintah berikut dan pastikan semuanya lulus tanpa error/warning:
1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test`
