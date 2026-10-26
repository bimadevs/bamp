# Dual Web Server Architecture via PHP-FPM

Traditional WAMP stacks bind Apache directly to `mod_php`, which consumes excessive RAM and locks the entire server to a single PHP version. BAMP uses PHP-FPM as the execution engine and supports both Nginx and Apache side-by-side, defaulting to Nginx for optimal throughput and memory efficiency while allowing instant 1-click fallback to Apache when `.htaccess` compatibility is required.
