# 07 - Developer Tooling: Adminer & Mailpit Integration

Status: ready-for-agent

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
- Visiting `http://mailpit.test` opens the Mailpit interface.
- PHP `mail()` calls are captured in Mailpit inbox without errors.
- Visiting `http://adminer.test` opens Adminer connected to `127.0.0.1:3306`.

## Comments
