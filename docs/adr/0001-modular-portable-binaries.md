# Modular Portable Binaries Architecture

To keep BAMP ultra-lightweight (<20MB core) and prevent monolithic bloat, service binaries (PHP, MySQL, Nginx, Apache) are decoupled from the core application installer into a portable `bin/` directory. BAMP allows users to either drop in custom pre-compiled versions manually or download curated packages via a 1-click cloud downloader, enabling instant multi-version switching without re-installing the application.
