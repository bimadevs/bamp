# Panduan Lengkap Penggunaan BAMP untuk Pemula

Selamat datang di **BAMP (Bima Apache/Nginx MySQL PHP)**! 🎉

Panduan ini dibuat khusus untuk Anda yang baru mulai belajar pemrograman web (coding PHP, HTML, CSS, dan MySQL). BAMP dirancang agar Anda bisa langsung fokus belajar membuat website tanpa perlu dipusingkan oleh pengaturan server yang rumit.

---

## 🌟 Daftar Isi
1. [Apa itu BAMP?](#1-apa-itu-bamp)
2. [Cara Menjalankan BAMP](#2-cara-menjalankan-bamp)
3. [Mengenal Tampilan Aplikasi BAMP](#3-mengenal-tampilan-aplikasi-bamp)
4. [Langkah Demi Langkah: Membuat Website Pertama Anda](#4-langkah-demi-langkah-membuat-website-pertama-anda)
5. [Mengelola Database dengan Adminer](#5-mengelola-database-dengan-adminer)
6. [Melihat Hasil Kirim Email dengan Mailpit](#6-melihat-hasil-kirim-email-dengan-mailpit)
7. [Mengaktifkan HTTPS (Gembok Hijau)](#7-mengaktifkan-https-gembok-hijau)
8. [Tips & Tombol Pintas (Shortcut)](#8-tips--tombol-pintas-shortcut)
9. [Solusi Masalah Umum (Troubleshooting)](#9-solusi-masalah-umum-troubleshooting)

---

## 1. Apa itu BAMP?

Jika Anda pernah mendengar aplikasi seperti **XAMPP** atau **Laragon**, BAMP memiliki fungsi yang serupa namun **jauh lebih ringan, cepat, dan modern**.

BAMP menyatukan semua peralatan yang Anda butuhkan untuk membuat website di komputer lokal Anda:
- **Web Server (Nginx & Apache)**: Komponen yang bertugas menerima permintaan dari browser Anda dan menampilkan halaman web.
- **PHP**: Bahasa pemrograman yang memproses kode logika di server.
- **MySQL / MariaDB**: Tempat menyimpan data website Anda (misalnya data pengguna, artikel blog, produk toko online).
- **Adminer**: Halaman web ringan untuk melihat dan mengedit database Anda.
- **Mailpit**: Kotak surat simulasi untuk menangkap email yang dikirim dari script PHP Anda agar tidak terkirim ke internet sungguhan saat Anda sedang belajar.
- **Domain Otomatis (`*.test`)**: Anda tidak perlu lagi mengetik `localhost/namafolder`. Cukup beri nama folder `toko`, dan website Anda otomatis bisa dibuka di browser dengan alamat `http://toko.test`!

---

## 2. Cara Menjalankan BAMP

### Menjalankan dalam Mode Pengembangan (Development)
Buka terminal di folder project BAMP, lalu jalankan perintah:

```bash
pnpm tauri dev
```

Aplikasi BAMP akan terbuka dalam sebuah jendela desktop yang elegan dan bersih.

---

## 3. Mengenal Tampilan Aplikasi BAMP

Setelah jendela BAMP terbuka, Anda akan melihat antarmuka yang sangat mudah dipahami:

1. **Bagian Atas (Header & Tombol Cepat)**:
   - **Search sites & tools (`Ctrl+K`)**: Pencarian kilat untuk melompat ke project Anda.
   - **Install CA**: Memasang sertifikat keamanan lokal agar website Anda berstatus HTTPS aman.
   - **Mailbox**: Membuka kotak surat Mailpit di browser (`http://localhost:8025`).
   - **Database**: Membuka pengelola database Adminer (`http://adminer.test`).
2. **Hero Bar (Status Utama)**:
   - Menampilkan status apakah server sedang aktif (**Running**) atau mati (**Stopped**).
   - Tombol **Start All**: Menyalakan semua komponen server sekaligus hanya dengan 1 klik!
   - Tombol **Stop All**: Mematikan semua server saat Anda selesai belajar.
3. **Kartu Layanan (Core Services)**:
   - Kotak status untuk **Nginx**, **Apache**, **PHP-FPM**, **MySQL**, dan **Mailpit**.
   - Dilengkapi lampu indikator:
     - 🟢 **Hijau (Running)**: Berjalan normal dan siap digunakan.
     - ⚪ **Abu-abu (Stopped)**: Sedang dimatikan.
     - 🟡 **Kuning (Conflict)**: Port sedang dipakai oleh aplikasi lain (lengkap dengan nama aplikasi dan PID-nya).
     - 🔴 **Merah (Error)**: Terjadi kendala teknis.
   - Anda bisa menyalakan atau mematikan tiap layanan secara terpisah dengan tombol **Start**, **Stop**, atau **Restart**.
4. **Tabel Virtual Hosts & Proyek**:
   - Menampilkan daftar semua website yang Anda buat di folder `www/`.
   - Menampilkan nama domain otomatis (contoh: `demo.test`), lokasi folder, dan status HTTPS.
   - Ada ikon panah kecil untuk **langsung membuka website di browser** dan ikon folder untuk **membuka foldernya di File Explorer**.
5. **Panel Catatan Aktivitas (Activity Logs)**:
   - Di bagian paling bawah layar, klik **Activity Logs** jika Anda ingin melihat catatan teknis proses server.

---

## 4. Langkah Demi Langkah: Membuat Website Pertama Anda

Mari kita buat halaman website pertama Anda dalam waktu kurang dari 1 menit!

### Langkah 4.1: Nyalakan BAMP
Klik tombol **Start All** di bar atas. Pastikan lampu indikator berubah menjadi hijau (**Running**).

### Langkah 4.2: Buat Folder Proyek Baru
1. Buka folder `www` yang ada di dalam direktori BAMP Anda. (Atau klik ikon folder pada antarmuka BAMP).
2. Buat folder baru, misalnya beri nama: `belajar-php`.

### Langkah 4.3: Buat File Koding Pertama
1. Masuk ke dalam folder `belajar-php` yang baru saja Anda buat.
2. Buat file baru bernama `index.php`.
3. Buka file tersebut dengan editor teks favorit Anda (seperti **VS Code** atau Notepad), lalu ketik kode berikut:

```php
<?php
  $nama = "Programmer Hebat";
  echo "<h1>Halo, Dunia!</h1>";
  echo "<p>Selamat datang <strong>$nama</strong> di website lokal BAMP pertama Anda!</p>";
  echo "<p>Waktu server saat ini: " . date("d F Y, H:i:s") . "</p>";
?>
```
4. Simpan file (`Ctrl + S`).

### Langkah 4.4: Buka di Browser Anda
1. Buka Google Chrome, Firefox, atau Edge.
2. Ketik alamat berikut di address bar:
   ```
   http://belajar-php.test
   ```
   *(Perhatikan: nama domain otomatis sama dengan nama folder yang Anda buat + `.test`)*
3. **Selesai!** Halaman website Anda langsung tampil di layar tanpa perlu restart server atau mengedit file sistem apa pun.

---

## 5. Mengelola Database dengan Adminer

Jika website Anda membutuhkan database (seperti untuk form pendaftaran akun atau toko online):

1. Pastikan layanan **MySQL / MariaDB** di dashboard BAMP berstatus **Running** (hijau).
2. Di dashboard BAMP bagian atas, klik tombol **Database** (atau ketik `http://adminer.test` di browser Anda).
3. Halaman login Adminer akan terbuka. Masukkan data berikut:
   - **Sistem**: MySQL
   - **Server**: `127.0.0.1` (atau kosongkan)
   - **Pengguna (Username)**: `root`
   - **Kata sandi (Password)**: *(kosongkan)*
   - **Database**: *(kosongkan untuk masuk ke menu utama)*
4. Klik tombol **Masuk / Login**.
5. Untuk membuat database baru:
   - Klik tulisan **Buat basis data baru**.
   - Beri nama database Anda (misalnya `db_belajar`).
   - Klik **Simpan**.

Sekarang database Anda siap dihubungkan ke kode PHP Anda menggunakan PDO atau MySQLi:
```php
<?php
$host = "127.0.0.1";
$user = "root";
$pass = "";
$db   = "db_belajar";

$koneksi = new mysqli($host, $user, $pass, $db);
if ($koneksi->connect_error) {
    die("Koneksi gagal: " . $koneksi->connect_error);
}
echo "Berhasil terhubung ke database!";
?>
```

---

## 6. Melihat Hasil Kirim Email dengan Mailpit

Saat belajar membuat form lupa password atau aktivasi email, seringkali pemula bingung bagaimana cara mengetesnya tanpa akun email sungguhan. BAMP sudah menyediakan **Mailpit**!

1. Pastikan layanan **Mailpit** di dashboard berstatus **Running** (hijau).
2. Di dashboard BAMP bagian atas, klik tombol **Mailbox** (atau buka `http://localhost:8025`).
3. Mailpit adalah kotak masuk email simulasi.
4. Di file PHP Anda, kirim email seperti biasa menggunakan fungsi `mail()` bawaan PHP:
   ```php
   <?php
   $kepada = "teman@contoh.com";
   $judul = "Selamat Datang!";
   $pesan = "Ini adalah email uji coba dari aplikasi BAMP lokal.";
   $pengirim = "From: noreply@belajar-php.test";

   if (mail($kepada, $judul, $pesan, $pengirim)) {
       echo "Email berhasil dikirim ke Mailpit!";
   } else {
       echo "Gagal mengirim email.";
   }
   ?>
   ```
5. Buka kembali tab `http://localhost:8025`, Anda akan langsung melihat surat tersebut masuk di inbox secara instan tanpa internet!

---

## 7. Mengaktifkan HTTPS (Gembok Hijau)

BAMP secara otomatis membuatkan sertifikat keamanan SSL untuk semua domain `*.test` Anda.
Agar browser Anda (Chrome, Firefox, Edge) menampilkan lambang gembok aman hijau tanpa peringatan:

1. Di dashboard BAMP bar atas, klik tombol **Install CA**.
2. Sistem akan meminta izin otorisasi keamanan satu kali untuk mendaftarkan Root Certificate BAMP ke sistem komputer Anda.
3. Masukkan password akun komputer Anda dan klik **OK/Izinkan**.
4. Sekarang Anda bisa membuka website Anda dengan protokol aman:
   ```
   https://belajar-php.test
   ```

---

## 8. Tips & Tombol Pintas (Shortcut)

- **Pencarian Kilat (`Ctrl + K` atau `Cmd + K`)**:
  Tekan `Ctrl+K` kapan saja di keyboard Anda. Sebuah jendela pencarian akan muncul. Anda bisa mengetik nama proyek untuk langsung membukanya tanpa perlu mencarinya secara manual.
- **Ikon Tray di Taskbar (Pojok Kanan Bawah Komputer)**:
  BAMP memiliki ikon kecil di tray taskbar. Anda bisa klik kanan ikon tersebut untuk:
  - Buka Dashboard
  - Buka folder `www`
  - Start All / Stop All
  - Keluar (Quit BAMP)
- **Menghubungkan Folder Proyek di Luar BAMP (Link Folder)**:
  Jika Anda menyimpan proyek coding di folder lain (misalnya di `D:\Kuliah\Web` atau `~/Coding`), Anda tidak perlu memindahkannya ke dalam folder BAMP! Cukup klik tombol **Link Folder** di dashboard, masukkan alamat foldernya, dan BAMP otomatis akan membuatkan domain `.test` untuk proyek tersebut.

---

## 9. Solusi Masalah Umum (Troubleshooting)

### Q: Status MySQL atau Nginx/Apache berwarna Kuning (Conflict)?
**Penyebab**: Komputer Anda mungkin sudah memiliki aplikasi web server atau database lain yang sedang aktif di latar belakang (misalnya XAMPP lain, MariaDB bawaan Linux, Skype, atau IIS Windows).
**Solusi**:
- Arahkan kursor mouse ke tulisan kartu yang kuning. BAMP akan memberi tahu secara spesifik aplikasi dan nomor PID yang sedang memakai port tersebut.
- Tutup aplikasi yang bentrok tersebut (misalnya matikan Apache XAMPP lama Anda atau jalankan `sudo systemctl stop httpd mariadb` di Linux).
- Klik tombol **Restart** pada kartu layanan di BAMP.

### Q: Kenapa domain `*.test` tidak bisa dibuka di browser?
**Solusi**:
1. Pastikan Anda mengetik `http://` di awal, misalnya `http://namaproject.test`.
2. Pastikan layanan BAMP sedang aktif (**Running**).
3. Jika port 53 (DNS) di komputer Anda dibatasi oleh sistem, BAMP secara otomatis menyediakan tombol sinkronisasi hosts. Pastikan nama domain terdaftar di dashboard BAMP.

### Q: Bagaimana cara menutup BAMP sepenuhnya?
**Solusi**:
Klik tombol **Quit** di menu System Tray atau tutup jendela aplikasi. Fitur **Zero-Zombie Guarantee** di BAMP menjamin bahwa saat aplikasi BAMP ditutup, seluruh proses latar belakang (PHP, Nginx, MySQL, Mailpit) akan langsung dihentikan secara bersih tanpa meninggalkan aplikasi gantung yang memakan memori RAM komputer Anda.

---

*Selamat belajar ngoding dan berkarya bersama BAMP! Jika ada ide atau masukan, jangan ragu untuk mengembangkannya bersama komunitas pengembang.* 🚀
