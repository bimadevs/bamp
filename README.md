# BAMP: Ultra-Lightweight Modern Local Development Stack Manager

**BAMP (Bima Apache/Nginx MySQL PHP)** adalah aplikasi manager web server lokal modern, super ringan (<30MB RAM idle), cepat, dan mudah digunakan untuk pengembang PHP, WordPress, dan Laravel.

---

## 📖 Panduan Pengguna untuk Pemula

Bagi Anda yang baru mulai belajar coding web atau baru menggunakan BAMP, kami telah menyediakan buku panduan lengkap yang sangat mudah dipahami:

👉 **[Baca Panduan Lengkap Penggunaan BAMP di Sini (PANDUAN-PENGGUNA.md)](./PANDUAN-PENGGUNA.md)**

Panduan tersebut memandu Anda langkah demi langkah:
1. Cara menjalankan BAMP
2. Mengenal tampilan dashboard
3. Membuat website pertama Anda dalam 1 menit
4. Mengelola database dengan Adminer
5. Menguji pengiriman email dengan Mailpit
6. Mengaktifkan HTTPS gembok hijau lokal
7. Mengatasi masalah umum (troubleshooting)

---

## 🚀 Fitur Unggulan

- **Super Ringan & Cepat**: Dibangun dengan Rust (Tauri v2) dan React 19 dengan konsumsi RAM webview < 25MB.
- **Zero-Zombie Guarantee**: Semua child process (PHP, Nginx, MySQL, Mailpit) dipastikan mati total saat BAMP ditutup, tanpa meninggalkan proses yatim (*orphan processes*).
- **Domain Otomatis (`*.test`)**: Embedded DNS Resolver UDP di port 53 memetakan semua folder `www/<nama-folder>` langsung ke `http://<nama-folder>.test`.
- **SSL Wildcard Otomatis**: Menghasilkan Root CA lokal dan sertifikat HTTPS hijau otomatis untuk semua domain `*.test`.
- **Dual Web Server**: Mendukung Nginx (default cepat) dan Apache (`httpd`) secara berdampingan.
- **Tooling Bawaan**: Termasuk database manager Adminer single-file (<500KB) dan server uji coba email Mailpit lokal.
- **Command Palette (`Ctrl+K`)**: Pencarian instan untuk membuka project atau mengontrol service dengan keyboard.

---

## 💻 Menjalankan BAMP dalam Mode Pengembangan

Pastikan Anda telah memasang **Node.js**, **pnpm**, dan **Rust**:

```bash
# Pasang dependensi frontend
pnpm install

# Jalankan dalam mode desktop development
pnpm tauri dev

# Uji coba build produksi
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
```

---

## 📄 Lisensi

Didistribusikan di bawah lisensi MIT.
