// Mock Tauri IPC for BAMP E2E testing
(() => {
  const callbacks = new Map();
  const eventListeners = new Map();

  function registerCallback(cb, once = false) {
    const id = window.crypto.getRandomValues(new Uint32Array(1))[0];
    callbacks.set(id, (data) => {
      if (once) callbacks.delete(id);
      return cb && cb(data);
    });
    return id;
  }

  function unregisterCallback(id) {
    callbacks.delete(id);
  }

  function runCallback(id, data) {
    const cb = callbacks.get(id);
    if (cb) {
      cb(data);
    }
  }

  function emitEvent(event, payload) {
    const handlers = eventListeners.get(event) || [];
    for (const handlerId of handlers) {
      runCallback(handlerId, { event, payload });
    }
  }

  // Initial state
  let activeWebServer = "nginx";
  let stackStatus = {
    isRunning: false,
    services: {
      nginx: { id: "nginx", name: "Nginx Web Server", status: "stopped", port: 80 },
      apache: { id: "apache", name: "Apache HTTP Server", status: "stopped", port: 80 },
      php: { id: "php", name: "PHP-FPM", status: "stopped", port: 9000 },
      mysql: { id: "mysql", name: "MySQL / MariaDB", status: "stopped", port: 3306 },
      mailpit: { id: "mailpit", name: "Mailpit SMTP & Web", status: "stopped", port: 8025 }
    }
  };

  let sites = [
    {
      id: "site-wordpress",
      name: "wordpress",
      domain: "wordpress.test",
      path: "/var/www/wordpress",
      documentRoot: "/var/www/wordpress/public",
      phpVersion: "default",
      hasSsl: true,
      status: "active"
    },
    {
      id: "site-laravel",
      name: "laravel",
      domain: "laravel.test",
      path: "/var/www/laravel",
      documentRoot: "/var/www/laravel/public",
      phpVersion: "php83",
      hasSsl: false,
      status: "active"
    }
  ];

  let availablePhpVersions = ["default", "php83", "php82"];

  let phpConfig = {
    iniPath: "/etc/php/8.3/fpm/php.ini",
    extensions: [
      { name: "curl", enabled: true, description: "cURL client" },
      { name: "mysqli", enabled: true, description: "MySQL database connector" },
      { name: "pdo_mysql", enabled: true, description: "PDO MySQL driver" },
      { name: "mbstring", enabled: true, description: "Multibyte string support" },
      { name: "xdebug", enabled: false, description: "Step debugger and profiler" }
    ],
    directives: [
      { key: "memory_limit", value: "256M", description: "Maximum memory limit" },
      { key: "upload_max_filesize", value: "64M", description: "Maximum file upload size" },
      { key: "max_execution_time", value: "60", description: "Maximum script execution time in seconds" }
    ]
  };

  let packages = [
    { id: "php83", name: "PHP 8.3 (Latest)", version: "8.3.16", is_installed: true, download_url: "" },
    { id: "php82", name: "PHP 8.2 (LTS)", version: "8.2.27", is_installed: false, download_url: "" },
    { id: "php81", name: "PHP 8.1", version: "8.1.31", is_installed: false, download_url: "" }
  ];

  function emitLog(service, msg, level = "info") {
    emitEvent("log-stream", { service, msg, level });
  }

  async function mockInvoke(cmd, args = {}) {
    // Event plugin handling
    if (cmd === "plugin:event|listen") {
      const event = args.event;
      const handlerId = args.handler;
      if (!eventListeners.has(event)) {
        eventListeners.set(event, []);
      }
      eventListeners.get(event).push(handlerId);
      return handlerId;
    }

    if (cmd === "plugin:event|unlisten") {
      const event = args.event;
      const handlerId = args.id;
      const handlers = eventListeners.get(event);
      if (handlers) {
        const idx = handlers.indexOf(handlerId);
        if (idx !== -1) handlers.splice(idx, 1);
      }
      return null;
    }

    if (cmd === "plugin:opener|open_url" || cmd === "plugin:opener|open_path") {
      return null;
    }

    // Core BAMP handlers
    switch (cmd) {
      case "get_stack_status":
        return JSON.parse(JSON.stringify(stackStatus));

      case "get_sites":
        return JSON.parse(JSON.stringify(sites));

      case "get_available_php_versions":
        return [...availablePhpVersions];

      case "get_settings":
        return {
          activeWebServer,
          defaultPhpVersion: "default",
          linkedFolders: [],
          sitePhpOverrides: {}
        };

      case "set_active_web_server": {
        activeWebServer = args.serverId;
        emitLog("config", `Active web server switched to ${activeWebServer.toUpperCase()}`);
        return activeWebServer;
      }

      case "start_all_services": {
        stackStatus.isRunning = true;
        for (const [id, s] of Object.entries(stackStatus.services)) {
          if ((id === "nginx" && activeWebServer !== "nginx") || (id === "apache" && activeWebServer !== "apache")) {
            s.status = "stopped";
          } else {
            s.status = "running";
            s.pid = 1000 + Math.floor(Math.random() * 500);
          }
        }
        emitLog("supervisor", "All active services started.", "info");
        return JSON.parse(JSON.stringify(stackStatus));
      }

      case "stop_all_services": {
        stackStatus.isRunning = false;
        for (const s of Object.values(stackStatus.services)) {
          s.status = "stopped";
          delete s.pid;
        }
        emitLog("supervisor", "All services stopped.", "info");
        return JSON.parse(JSON.stringify(stackStatus));
      }

      case "start_service": {
        const s = stackStatus.services[args.serviceId];
        if (s) {
          s.status = "running";
          s.pid = 2000 + Math.floor(Math.random() * 500);
        }
        stackStatus.isRunning = true;
        emitLog(args.serviceId, `Service started.`, "info");
        return JSON.parse(JSON.stringify(s));
      }

      case "stop_service": {
        const s = stackStatus.services[args.serviceId];
        if (s) {
          s.status = "stopped";
          delete s.pid;
        }
        stackStatus.isRunning = Object.values(stackStatus.services).some((srv) => srv.status === "running");
        emitLog(args.serviceId, `Service stopped.`, "info");
        return JSON.parse(JSON.stringify(s));
      }

      case "restart_service": {
        const s = stackStatus.services[args.serviceId];
        if (s) {
          s.status = "running";
          s.pid = 3000 + Math.floor(Math.random() * 500);
        }
        stackStatus.isRunning = true;
        emitLog(args.serviceId, `Service restarted.`, "info");
        return JSON.parse(JSON.stringify(s));
      }

      case "install_root_ca": {
        emitLog("ssl", "Root CA successfully installed in trust store.", "info");
        return "Root CA successfully installed in trust store.";
      }

      case "add_linked_folder": {
        const folderName = (args.path || "custom-project").split(/[/\\\\]/).filter(Boolean).pop() || "custom-project";
        const newSite = {
          id: `site-${Date.now()}`,
          name: folderName,
          domain: `${folderName}.test`,
          path: args.path,
          documentRoot: args.path,
          phpVersion: "default",
          hasSsl: true,
          status: "active"
        };
        sites.push(newSite);
        emitLog("config", `Linked project folder ${folderName} saved.`, "info");
        return JSON.parse(JSON.stringify(sites));
      }

      case "set_site_php_version": {
        const site = sites.find((s) => s.id === args.siteId);
        if (site) {
          site.phpVersion = args.phpVersion;
        }
        emitLog("config", `Updated PHP version to ${args.phpVersion}`, "info");
        return JSON.parse(JSON.stringify(site));
      }

      case "get_php_config":
        return JSON.parse(JSON.stringify(phpConfig));

      case "toggle_php_extension": {
        const ext = phpConfig.extensions.find((e) => e.name === args.extensionName);
        if (ext) {
          ext.enabled = args.enabled;
        }
        emitLog("php", `Extension ${args.extensionName} toggled to ${args.enabled}`, "info");
        return args.enabled;
      }

      case "update_php_directive": {
        const dir = phpConfig.directives.find((d) => d.key === args.key);
        if (dir) {
          dir.value = args.value;
        }
        emitLog("php", `Directive ${args.key} set to ${args.value}`, "info");
        return args.value;
      }

      case "launch_isolated_terminal": {
        emitLog("tools", "Terminal opened with BAMP environment variables.", "info");
        return "Terminal opened with BAMP environment variables.";
      }

      case "launch_database_client": {
        emitLog("tools", "Database action: adminer", "info");
        return "adminer";
      }

      case "get_package_hub":
        return JSON.parse(JSON.stringify(packages));

      case "install_package": {
        const pkg = packages.find((p) => p.id === args.packageId);
        if (pkg) {
          pkg.is_installed = true;
        }
        emitLog("package-hub", `Installed package ${args.packageId}`, "info");
        return `Package ${args.packageId} installed successfully.`;
      }

      case "scaffold_app": {
        const cleanName = (args.projectName || "project").toLowerCase().replace(/\s+/g, "-");
        const newSite = {
          id: `site-${Date.now()}`,
          name: cleanName,
          domain: `${cleanName}.test`,
          path: `/var/www/${cleanName}`,
          documentRoot: `/var/www/${cleanName}${args.appType === "laravel" ? "/public" : ""}`,
          phpVersion: "default",
          hasSsl: true,
          status: "active"
        };
        sites.push(newSite);
        emitLog("scaffold", `Created ${args.appType} project at http://${cleanName}.test`, "info");
        return `http://${cleanName}.test`;
      }

      default:
        console.warn(`[Mock Tauri] Unhandled command: ${cmd}`, args);
        return null;
    }
  }

  // Setup window globals
  window.__TAURI_INTERNALS__ = {
    invoke: mockInvoke,
    transformCallback: registerCallback,
    unregisterCallback,
    runCallback,
    callbacks,
    metadata: {
      currentWindow: { label: "main" },
      currentWebview: { windowLabel: "main", label: "main" }
    }
  };

  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    unregisterListener: (event, id) => {
      const handlers = eventListeners.get(event);
      if (handlers) {
        const idx = handlers.indexOf(id);
        if (idx !== -1) handlers.splice(idx, 1);
      }
    }
  };

  // Mock window.prompt for folder linking
  window.prompt = () => "/home/bimadev/project/rea/bamp/www/my-linked-store";
  window.alert = (msg) => { console.log("[ALERT]", msg); };
})();
