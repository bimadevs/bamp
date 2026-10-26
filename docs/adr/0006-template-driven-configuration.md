# Template-Driven Configuration Engine

Instead of hardcoding web server and service configurations into compiled Rust binaries, BAMP uses external template files stored in `templates/` with safe placeholder substitution. This guarantees that beginners receive working zero-configuration setups out of the box, while advanced developers can customize their base server directives (such as custom CORS headers, fastcgi buffer limits, or SSL cipher suites) without BAMP overwriting their changes during virtual host generation.
