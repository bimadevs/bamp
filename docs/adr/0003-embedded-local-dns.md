# Embedded Local DNS for Wildcard Domains

Editing the system `hosts` file for every project requires elevated administrative privileges (UAC/sudo) and does not support wildcard patterns. BAMP embeds a lightweight local UDP DNS resolver in Rust listening for `*.test` queries and returning `127.0.0.1`, enabling zero-configuration virtual hosts whenever a project folder is detected, with an automatic fallback to `hosts` file modification if port 53 is unavailable.
