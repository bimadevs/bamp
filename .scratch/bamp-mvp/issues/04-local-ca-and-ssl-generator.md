# 04 - Local Root CA & SSL Generator

Status: resolved

## Goal
Implement automated X.509 Root CA generation and wildcard `*.test` SSL certificate issuance in `src-tauri/src/ssl/` so local domains have valid, trusted HTTPS out of the box.

## Requirements
1. Using `rcgen`:
   - Generate BAMP Root CA (`bamp-root-ca.crt` & `bamp-root-ca.key`).
   - Generate Wildcard Certificate for `*.test`, `localhost`, and `127.0.0.1` signed by the Root CA.
   - Save output to `data/ssl/cert.pem` and `data/ssl/key.pem`.
2. Provide platform installation routines:
   - Windows: Install Root CA into `Root` certificate store via `certutil`.
   - Linux: Copy to `/usr/local/share/ca-certificates/` and invoke `update-ca-certificates`.
3. Expose Tauri command `ensure_ssl_certificates` returning cert paths.

## Completion Criteria
- [x] Valid X.509 certificates generated in `data/ssl/`.
- [x] Openssl verify check (`openssl verify -CAfile data/ssl/bamp-root-ca.crt data/ssl/cert.pem`) outputs `OK`.

## Comments
- Implemented `SslManager::ensure_certificates` using `rcgen 0.13` generating BAMP Root CA and `*.test` wildcard leaf certificates.
- Bundled leaf certificate and CA into `data/ssl/cert.pem` fullchain for web server use.
- Implemented `install_root_ca` supporting Windows `certutil` and Linux `trust anchor` / `update-ca-certificates`.
- Exposed Tauri commands `ensure_ssl_certificates` and `install_root_ca`.
- Verified certificate validity with OpenSSL (`openssl verify -CAfile bamp-root-ca.crt cert.pem`) returning OK in unit tests.
