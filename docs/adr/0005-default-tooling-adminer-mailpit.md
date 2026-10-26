# Default Tooling: Adminer and Mailpit

To maintain strict minimalism and eliminate bloat, BAMP bundles Adminer (a single-file PHP database management tool <500KB) and Mailpit (a lightweight Go-based local SMTP testing server <15MB) as default utilities instead of monolithic suites like phpMyAdmin or heavy electron clients. Users retain the ability to launch installed desktop database clients via 1-click hooks or install phpMyAdmin on-demand via the package manager.
