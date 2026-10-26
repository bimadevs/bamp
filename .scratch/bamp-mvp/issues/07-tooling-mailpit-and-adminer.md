# 07 - Developer Tooling: Adminer & Mailpit Integration

Status: resolved

## Goal
Integrate Adminer for lightweight database management and Mailpit for local email inspection into the BAMP environment.

## Requirements
1. Bundle Adminer:
   - Provide script or embedded asset to fetch the latest single-file `adminer.php` into `data/adminer/index.php`.
   - Configure Nginx virtual host for `http://adminer.test` or route `http://localhost/adminer` to serve this file.
2. Mailpit Integration:
   - Supervise the Mailpit binary via the Supervisor.
   - Listen on SMTP port `1025` and Web UI port `8025`.
   - Configure PHP's `php.ini` `sendmail_path` directive to automatically route outgoing emails to `127.0.0.1:1025`.
3. Add quick launcher buttons in the React dashboard:
   - "Open Database Manager (Adminer)"
   - "Open Mailbox (Mailpit)"

## Completion Criteria
- [x] Visiting `http://mailpit.test` opens the Mailpit interface.
- [x] PHP `mail()` calls are captured in Mailpit inbox without errors.
- [x] Visiting `http://adminer.test` opens Adminer connected to `127.0.0.1:3306`.

## Comments
- Downloaded and bundled single-file Adminer database manager into `data/adminer/index.php`.
- Auto-registered `adminer.test` virtual host pointing to `data/adminer/` in `ConfigManager`.
- Provisioned portable Mailpit binary in `bin/mailpit/mailpit` and registered in `ProcessSupervisor` on ports 1025 (SMTP) and 8025 (Web UI).
- Configured mail catching parameters for integration into the React dashboard.
