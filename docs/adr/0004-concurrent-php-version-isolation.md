# Concurrent PHP-FPM Version Isolation

To allow developers to work on modern and legacy projects simultaneously without manual stack reconfiguration, BAMP supports running multiple distinct PHP-FPM runtime instances concurrently on dedicated Unix domain sockets or loopback ports. The web server configuration generator dynamically binds each Virtual Host to its designated PHP version pool, defaulting to the global version while supporting explicit per-site overrides.
