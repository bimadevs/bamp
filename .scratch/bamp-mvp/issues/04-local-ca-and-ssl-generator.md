# 04 - Local Root CA & SSL Generator

Status: ready-for-agent

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
- Valid X.509 certificates generated in `data/ssl/`.
- Openssl verify check (`openssl verify -CAfile data/ssl/bamp-root-ca.crt data/ssl/cert.pem`) outputs `OK`.

## Comments
