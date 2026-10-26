# BAMP

A lightweight, high-performance, and modular local web development stack manager built with Tauri v2, Rust, and React.

## Language

**Service**:
A background server process managed by BAMP, such as Nginx, Apache, MySQL, PHP-FPM, or Mailpit.
_Avoid_: Daemon, worker, background task

**Stack**:
The active combination of services running together to serve local web applications.
_Avoid_: Environment, bundle

**Virtual Host**:
A web server configuration mapping a local domain name to a specific project directory on disk.
_Avoid_: Vhost, site, server block

**Local Domain**:
A domain name ending in `.test` that resolves to the local loopback address (`127.0.0.1`) without public internet routing.
_Avoid_: Custom domain, fake domain, dev URL

**Root CA**:
A locally generated Certificate Authority trusted by the operating system to issue verified SSL certificates for local domains.
_Avoid_: Self-signed cert, custom SSL, dev cert

**Supervisor**:
The Rust-native subsystem responsible for spawning, health-checking, port-auditing, and cleanly terminating all child service processes.
_Avoid_: Process manager, watcher, orchestrator

**Tray**:
The operating system notification area icon from which quick actions, status checks, and background control can be executed without keeping a GUI window open.
_Avoid_: Taskbar icon, dock icon, status bar

**Linked Folder**:
A user-specified project directory located outside the default root that BAMP actively monitors for automatic virtual host generation.
_Avoid_: External path, custom folder, mapped drive

**Mail Catcher**:
A local SMTP service that intercepts and visualizes outgoing application emails without delivering them to external mailboxes.
_Avoid_: Mail server, SMTP mock, fake email

**Isolated Shell**:
A terminal session initialized with environment variables and `PATH` prepended to point exclusively to BAMP's active service binaries.
_Avoid_: Dev console, embedded terminal, system cmd

**Quick App**:
An automated scaffolding routine that provisions a project directory, database, virtual host, and SSL certificate in a single operation.
_Avoid_: Project wizard, app generator, template cloner

**Version Isolation**:
The capability to assign and run distinct PHP-FPM runtime versions concurrently for different virtual hosts.
_Avoid_: Multi-php, version switching, per-site php

**Directive**:
A key-value configuration setting within a service definition file (such as `php.ini` or `nginx.conf`) that governs runtime behavior.
_Avoid_: Flag, parameter, option

**Extension**:
A dynamically loadable runtime library (such as a PHP `.dll` or `.so`) that provides specific features like database drivers or debugging tools.
_Avoid_: Plugin, addon, module

**Configuration Template**:
A structured blueprint file with variable placeholders used by BAMP to dynamically generate active service and virtual host configurations.
_Avoid_: Preset, boilerplate, stub

**Aggregated Log**:
A unified real-time stream that consolidates diagnostic and error outputs from all managed services into a single interface.
_Avoid_: Console dump, system output, debug trace
