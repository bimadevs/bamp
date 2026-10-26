# 01 - Rust Backend Scaffolding & Dependencies

Status: ready-for-agent

## Goal
Configure `src-tauri/Cargo.toml` with the required crates for async runtime, socket handling, file watching, and SSL generation, and establish the Rust module hierarchy.

## Requirements
1. Add dependencies to `src-tauri/Cargo.toml`:
   - `tokio = { version = "1", features = ["full"] }`
   - `notify = "6"`
   - `rcgen = "0.13"` (for X.509 certificate and CA generation)
   - `serde = { version = "1", features = ["derive"] }`
   - `serde_json = "1"`
   - `tauri = { version = "2", features = ["tray-icon"] }`
2. Create folder and file structure in `src-tauri/src/`:
   - `supervisor/mod.rs`
   - `dns/mod.rs`
   - `ssl/mod.rs`
   - `config/mod.rs`
   - `inspector/mod.rs`
   - `platform/mod.rs`
3. Wire modules into `src-tauri/src/lib.rs`.

## Completion Criteria
- `cargo check` inside `src-tauri` succeeds with zero errors.

## Comments
