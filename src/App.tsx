import { useState, useEffect, useMemo, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openUrl, openPath } from "@tauri-apps/plugin-opener";
import {
  Play,
  Square,
  RotateCw,
  ExternalLink,
  ShieldCheck,
  AlertTriangle,
  Database,
  Mail,
  Search,
  Terminal,
  ChevronDown,
  ChevronUp,
  Plus,
  RefreshCw,
  Server,
  FolderOpen,
  Settings,
  Layers,
  Box,
  Sparkles,
  Check,
  X,
  Globe,
} from "lucide-react";
import "./App.css";

export type ServiceStatus = "stopped" | "starting" | "running" | "stopping" | "conflict" | "error";

export interface ServiceState {
  id: string;
  name: string;
  status: ServiceStatus;
  pid?: number;
  port?: number;
  version?: string;
  error?: string;
}

export interface StackStatus {
  isRunning: boolean;
  services: Record<string, ServiceState>;
}

export interface Site {
  id: string;
  name: string;
  domain: string;
  path: string;
  documentRoot: string;
  phpVersion: string;
  hasSsl: boolean;
  status: "active" | "error";
}

export interface AppSettings {
  activeWebServer: string;
  defaultPhpVersion: string;
  linkedFolders: string[];
  sitePhpOverrides: Record<string, string>;
}

export interface PhpExtension {
  name: string;
  enabled: boolean;
  description: string;
}

export interface PhpDirective {
  key: string;
  value: string;
  description: string;
}

export interface PhpConfig {
  iniPath: string;
  extensions: PhpExtension[];
  directives: PhpDirective[];
}

export interface PackageInfo {
  id: string;
  name: string;
  version: string;
  is_installed: boolean;
  path?: string;
  download_url: string;
}

export default function App() {
  const [stackStatus, setStackStatus] = useState<StackStatus>({
    isRunning: false,
    services: {},
  });
  const [activeWebServer, setActiveWebServer] = useState<string>("nginx");
  const [sites, setSites] = useState<Site[]>([]);
  const [availablePhpVersions, setAvailablePhpVersions] = useState<string[]>(["default"]);
  const [logs, setLogs] = useState<Array<{ id: string; time: string; level: string; msg: string; service: string }>>([]);
  const [selectedLogFilter, setSelectedLogFilter] = useState("all");
  const [isLogOpen, setIsLogOpen] = useState(false);
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  const [isPhpModalOpen, setIsPhpModalOpen] = useState(false);
  const [isQuickAppOpen, setIsQuickAppOpen] = useState(false);
  const [isPackageHubOpen, setIsPackageHubOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [actionLoading, setActionLoading] = useState<string | null>(null);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  // PHP Manager Modal State
  const [phpConfig, setPhpConfig] = useState<PhpConfig | null>(null);
  const [activePhpTab, setActivePhpTab] = useState<"extensions" | "directives">("extensions");
  const [directiveEdits, setDirectiveEdits] = useState<Record<string, string>>({});

  // Quick App Scaffolder State
  const [scaffoldType, setScaffoldType] = useState<"blank" | "wordpress" | "laravel">("blank");
  const [scaffoldName, setScaffoldName] = useState("");
  const [scaffoldLoading, setScaffoldLoading] = useState(false);

  // Package Hub State
  const [packages, setPackages] = useState<PackageInfo[]>([]);

  const logEndRef = useRef<HTMLDivElement>(null);

  const addLog = (service: string, msg: string, level = "info") => {
    const time = new Date().toLocaleTimeString();
    setLogs((prev) => [
      ...prev.slice(-300),
      { id: Math.random().toString(), time, level, msg, service },
    ]);
  };

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 4000);
  };

  // Poll stack status and sites periodically
  const fetchStatus = async () => {
    try {
      const status = await invoke<StackStatus>("get_stack_status");
      setStackStatus(status);
    } catch (e) {
      console.error("Failed to get stack status:", e);
    }
  };

  const fetchSites = async () => {
    try {
      const s = await invoke<Site[]>("get_sites");
      setSites(s);
    } catch (e) {
      console.error("Failed to get sites:", e);
    }
  };

  const fetchPhpVersions = async () => {
    try {
      const vers = await invoke<string[]>("get_available_php_versions");
      setAvailablePhpVersions(vers);
    } catch (e) {
      console.error("Failed to get PHP versions:", e);
    }
  };

  const fetchSettings = async () => {
    try {
      const s = await invoke<AppSettings>("get_settings");
      if (s.activeWebServer) {
        setActiveWebServer(s.activeWebServer);
      }
    } catch (e) {
      console.error("Failed to get settings:", e);
    }
  };

  const fetchPhpConfig = async () => {
    try {
      const cfg = await invoke<PhpConfig>("get_php_config");
      setPhpConfig(cfg);
      const initialDirectives: Record<string, string> = {};
      cfg.directives.forEach((d) => {
        initialDirectives[d.key] = d.value;
      });
      setDirectiveEdits(initialDirectives);
    } catch (e) {
      console.error("Failed to get PHP config:", e);
    }
  };

  const fetchPackageHub = async () => {
    try {
      const pkgs = await invoke<PackageInfo[]>("get_package_hub");
      setPackages(pkgs);
    } catch (e) {
      console.error("Failed to get packages:", e);
    }
  };

  // Live Piped Log Stream from Rust Backend
  useEffect(() => {
    const unlistenPromise = listen<{ service: string; level: string; msg: string }>("log-stream", (event) => {
      const time = new Date().toLocaleTimeString();
      setLogs((prev) => [
        ...prev.slice(-300),
        { id: Math.random().toString(), time, level: event.payload.level, msg: event.payload.msg, service: event.payload.service },
      ]);
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  useEffect(() => {
    if (isLogOpen && logEndRef.current) {
      logEndRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [logs, isLogOpen]);

  useEffect(() => {
    fetchStatus();
    fetchSites();
    fetchPhpVersions();
    fetchSettings();
    addLog("system", "BAMP Developer Dashboard initialized.", "info");

    const timer = setInterval(() => {
      fetchStatus();
      fetchSites();
    }, 2500);

    return () => clearInterval(timer);
  }, []);

  // Keyboard shortcut Ctrl+K / Cmd+K for Command Palette
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setIsCommandPaletteOpen((prev) => !prev);
      } else if (e.key === "Escape") {
        setIsCommandPaletteOpen(false);
        setIsPhpModalOpen(false);
        setIsQuickAppOpen(false);
        setIsPackageHubOpen(false);
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  // Web Server Switcher (Nginx <-> Apache)
  const handleSwitchWebServer = async (server: "nginx" | "apache") => {
    if (server === activeWebServer) return;
    setActionLoading("webserver_switch");
    try {
      await invoke("set_active_web_server", { serverId: server });
      setActiveWebServer(server);
      showToast(`Switched active web server to ${server.toUpperCase()}`);
      addLog("config", `Active web server switched to ${server.toUpperCase()}`, "info");
      await fetchStatus();
    } catch (err: unknown) {
      showToast(`Error switching web server: ${String(err)}`);
    } finally {
      setActionLoading(null);
    }
  };

  // Global Controls
  const handleStartAll = async () => {
    setActionLoading("start_all");
    addLog("supervisor", "Starting all services...", "info");
    try {
      const status = await invoke<StackStatus>("start_all_services");
      setStackStatus(status);
      addLog("supervisor", "Start all completed.", "info");
      showToast("All active services started.");
    } catch (err: unknown) {
      addLog("supervisor", `Start all error: ${String(err)}`, "error");
      showToast(`Error: ${String(err)}`);
    } finally {
      setActionLoading(null);
    }
  };

  const handleStopAll = async () => {
    setActionLoading("stop_all");
    addLog("supervisor", "Stopping all services...", "info");
    try {
      const status = await invoke<StackStatus>("stop_all_services");
      setStackStatus(status);
      addLog("supervisor", "All services stopped.", "info");
      showToast("All services stopped.");
    } catch (err: unknown) {
      addLog("supervisor", `Stop all error: ${String(err)}`, "error");
    } finally {
      setActionLoading(null);
    }
  };

  // Individual Service Controls
  const handleStartService = async (serviceId: string) => {
    setActionLoading(serviceId);
    try {
      await invoke("start_service", { serviceId });
      addLog(serviceId, `Service started.`, "info");
      await fetchStatus();
    } catch (err: unknown) {
      addLog(serviceId, `Failed to start: ${String(err)}`, "error");
      showToast(`Failed to start ${serviceId}: ${String(err)}`);
    } finally {
      setActionLoading(null);
    }
  };

  const handleStopService = async (serviceId: string) => {
    setActionLoading(serviceId);
    try {
      await invoke("stop_service", { serviceId });
      addLog(serviceId, `Service stopped.`, "info");
      await fetchStatus();
    } catch (err: unknown) {
      addLog(serviceId, `Failed to stop: ${String(err)}`, "error");
    } finally {
      setActionLoading(null);
    }
  };

  const handleRestartService = async (serviceId: string) => {
    setActionLoading(serviceId);
    try {
      await invoke("restart_service", { serviceId });
      addLog(serviceId, `Service restarted.`, "info");
      await fetchStatus();
    } catch (err: unknown) {
      addLog(serviceId, `Failed to restart: ${String(err)}`, "error");
    } finally {
      setActionLoading(null);
    }
  };

  const handleInstallRootCa = async () => {
    addLog("ssl", "Attempting Root CA installation into trust store...", "info");
    try {
      const res = await invoke<string>("install_root_ca");
      addLog("ssl", res, "info");
      showToast(res);
    } catch (err: unknown) {
      addLog("ssl", String(err), "error");
      showToast(String(err));
    }
  };

  const handleAddLinkedFolder = async () => {
    const folderPath = prompt("Enter full folder path to link into BAMP:");
    if (!folderPath) return;
    try {
      const updated = await invoke<Site[]>("add_linked_folder", { path: folderPath.trim() });
      setSites(updated);
      showToast("Linked project folder saved and virtual host generated.");
    } catch (err: unknown) {
      alert(`Error linking folder: ${String(err)}`);
    }
  };

  const handleSetSitePhpVersion = async (siteId: string, version: string) => {
    try {
      const updated = await invoke<Site>("set_site_php_version", { siteId, phpVersion: version });
      setSites((prev) => prev.map((s) => (s.id === siteId ? updated : s)));
      showToast(`PHP version for ${updated.domain} set to ${version}`);
      addLog("config", `Updated ${updated.domain} to ${version}`, "info");
    } catch (err: unknown) {
      showToast(`Error changing PHP version: ${String(err)}`);
    }
  };

  // Tool Launchers
  const handleLaunchTerminal = async () => {
    try {
      const res = await invoke<string>("launch_isolated_terminal", { activePhp: null });
      showToast(res);
      addLog("tools", res, "info");
    } catch (err: unknown) {
      showToast(`Failed to open terminal: ${String(err)}`);
    }
  };

  const handleLaunchDatabase = async () => {
    try {
      const res = await invoke<string>("launch_database_client");
      if (res === "adminer") {
        openUrl("http://adminer.test");
      } else {
        showToast(res);
      }
      addLog("tools", `Database action: ${res}`, "info");
    } catch (err: unknown) {
      showToast(`Database launcher: ${String(err)}`);
    }
  };

  // PHP Manager Actions
  const handleToggleExtension = async (extName: string, currentEnabled: boolean) => {
    try {
      const newStatus = await invoke<boolean>("toggle_php_extension", {
        extensionName: extName,
        enabled: !currentEnabled,
      });
      setPhpConfig((prev) => {
        if (!prev) return prev;
        return {
          ...prev,
          extensions: prev.extensions.map((e) => (e.name === extName ? { ...e, enabled: newStatus } : e)),
        };
      });
      showToast(`${extName} ${newStatus ? "enabled" : "disabled"} in php.ini`);
      addLog("php", `Extension ${extName} toggled to ${newStatus}`, "info");
    } catch (err: unknown) {
      showToast(`Error toggling extension: ${String(err)}`);
    }
  };

  const handleSaveDirective = async (key: string) => {
    const val = directiveEdits[key];
    if (val === undefined) return;
    try {
      await invoke("update_php_directive", { key, value: val });
      showToast(`Directive ${key} updated to ${val}`);
      addLog("php", `Directive ${key} set to ${val}`, "info");
    } catch (err: unknown) {
      showToast(`Error saving directive: ${String(err)}`);
    }
  };

  // Quick App Scaffolder Action
  const handleScaffoldSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!scaffoldName.trim()) return;
    setScaffoldLoading(true);
    try {
      const url = await invoke<string>("scaffold_app", {
        appType: scaffoldType,
        projectName: scaffoldName.trim(),
      });
      showToast(`Project created successfully!`);
      addLog("scaffold", `Created ${scaffoldType} project at ${url}`, "info");
      setIsQuickAppOpen(false);
      setScaffoldName("");
      await fetchSites();
      openUrl(url);
    } catch (err: unknown) {
      showToast(`Scaffold error: ${String(err)}`);
    } finally {
      setScaffoldLoading(false);
    }
  };

  // Package Hub Install
  const handleInstallPackage = async (pkgId: string) => {
    try {
      const res = await invoke<string>("install_package", { packageId: pkgId });
      showToast(res);
      addLog("package-hub", res, "info");
      await fetchPackageHub();
      await fetchPhpVersions();
    } catch (err: unknown) {
      showToast(`Package install error: ${String(err)}`);
    }
  };

  const filteredSites = useMemo(() => {
    if (!searchQuery) return sites;
    const q = searchQuery.toLowerCase();
    return sites.filter((s) => s.domain.toLowerCase().includes(q) || s.name.toLowerCase().includes(q));
  }, [sites, searchQuery]);

  const filteredLogs = useMemo(() => {
    if (selectedLogFilter === "all") return logs;
    return logs.filter((l) => l.service.toLowerCase().includes(selectedLogFilter.toLowerCase()));
  }, [logs, selectedLogFilter]);

  const serviceList = Object.values(stackStatus.services);

  return (
    <div className="flex flex-col h-screen w-full bg-[var(--color-bg-base)] text-[var(--color-text-primary)] select-none">
      {/* Top Header / Brand */}
      <header className="flex items-center justify-between px-6 py-3.5 border-b border-[var(--color-border)] bg-[var(--color-bg-surface)]">
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-[var(--color-primary-subtle)] text-[var(--color-primary)] font-bold text-lg">
            B
          </div>
          <div>
            <h1 className="text-base font-semibold tracking-tight text-white m-0">BAMP</h1>
            <p className="text-xs text-[var(--color-text-muted)] m-0">Modern WAMP • Multi-PHP • Zero-Zombie</p>
          </div>
        </div>

        {/* Global Toolbar */}
        <div className="flex items-center gap-2">
          {/* Active Web Server Segmented Toggle */}
          <div className="flex items-center bg-[var(--color-bg-subtle)] border border-[var(--color-border)] rounded-lg p-0.5 text-xs">
            <button
              onClick={() => handleSwitchWebServer("nginx")}
              className={`px-2.5 py-1 rounded-md font-medium transition-colors cursor-pointer ${
                activeWebServer === "nginx"
                  ? "bg-[var(--color-primary)] text-white shadow-xs"
                  : "text-[var(--color-text-secondary)] hover:text-white"
              }`}
            >
              Nginx
            </button>
            <button
              onClick={() => handleSwitchWebServer("apache")}
              className={`px-2.5 py-1 rounded-md font-medium transition-colors cursor-pointer ${
                activeWebServer === "apache"
                  ? "bg-[var(--color-primary)] text-white shadow-xs"
                  : "text-[var(--color-text-secondary)] hover:text-white"
              }`}
            >
              Apache
            </button>
          </div>

          <button
            onClick={() => setIsCommandPaletteOpen(true)}
            className="flex items-center gap-2 px-3 py-1.5 text-xs text-[var(--color-text-secondary)] bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-md transition-colors"
          >
            <Search className="w-3.5 h-3.5" />
            <span>Search</span>
            <kbd className="px-1.5 py-0.5 text-[10px] bg-[var(--color-bg-base)] rounded border border-[var(--color-border)] text-[var(--color-text-muted)]">
              Ctrl+K
            </kbd>
          </button>

          <button
            onClick={handleLaunchTerminal}
            className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium text-[var(--color-text-secondary)] hover:text-white bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-md transition-colors cursor-pointer"
            title="Open Isolated Terminal with BAMP PATH"
          >
            <Terminal className="w-3.5 h-3.5 text-emerald-400" />
            <span>Terminal</span>
          </button>

          <button
            onClick={() => {
              fetchPhpConfig();
              setIsPhpModalOpen(true);
            }}
            className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium text-[var(--color-text-secondary)] hover:text-white bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-md transition-colors cursor-pointer"
            title="Manage PHP Extensions and Directives"
          >
            <Settings className="w-3.5 h-3.5 text-cyan-400" />
            <span>PHP Settings</span>
          </button>

          <button
            onClick={() => {
              fetchPackageHub();
              setIsPackageHubOpen(true);
            }}
            className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium text-[var(--color-text-secondary)] hover:text-white bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-md transition-colors cursor-pointer"
            title="Package Hub & PHP Versions"
          >
            <Box className="w-3.5 h-3.5 text-purple-400" />
            <span>Packages</span>
          </button>

          <button
            onClick={handleInstallRootCa}
            className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium text-[var(--color-text-secondary)] hover:text-white bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-md transition-colors cursor-pointer"
            title="Install BAMP Root CA to trust store for valid HTTPS"
          >
            <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />
            <span>Install CA</span>
          </button>

          <button
            onClick={() => openUrl("http://localhost:8025")}
            className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium text-[var(--color-text-secondary)] hover:text-white bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-md transition-colors cursor-pointer"
            title="Open Mailpit local mail catcher"
          >
            <Mail className="w-3.5 h-3.5 text-indigo-400" />
            <span>Mailbox</span>
          </button>

          <button
            onClick={handleLaunchDatabase}
            className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium text-[var(--color-text-secondary)] hover:text-white bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-md transition-colors cursor-pointer"
            title="Open Database in Desktop Client or Adminer"
          >
            <Database className="w-3.5 h-3.5 text-amber-400" />
            <span>Database</span>
          </button>
        </div>
      </header>

      {/* Main Content Area */}
      <main className="flex-1 overflow-y-auto px-6 py-6 space-y-6">
        {/* Toast Notification */}
        {toastMessage && (
          <div className="fixed top-16 right-6 z-50 bg-[var(--color-bg-subtle)] border border-[var(--color-border)] text-sm px-4 py-2.5 rounded-lg shadow-xl text-white flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-[var(--color-primary)]" />
            <span>{toastMessage}</span>
          </div>
        )}

        {/* Hero Stack Bar */}
        <div className="flex items-center justify-between p-5 rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-surface)]">
          <div className="flex items-center gap-4">
            <div
              className={`w-4 h-4 rounded-full flex items-center justify-center ${
                stackStatus.isRunning ? "bg-emerald-500/20" : "bg-zinc-500/20"
              }`}
            >
              <div
                className={`w-2 h-2 rounded-full ${
                  stackStatus.isRunning ? "bg-emerald-400 animate-pulse" : "bg-zinc-500"
                }`}
              />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="text-sm font-semibold text-white">
                  Stack Status: {stackStatus.isRunning ? "Running" : "Stopped"}
                </span>
                <span className="text-xs px-2 py-0.5 rounded bg-[var(--color-bg-subtle)] border border-[var(--color-border)] text-indigo-300 font-mono">
                  Active Web Server: {activeWebServer.toUpperCase()}
                </span>
              </div>
              <p className="text-xs text-[var(--color-text-muted)] m-0">
                {serviceList.filter((s) => s.status === "running").length} of {serviceList.length} services active
              </p>
            </div>
          </div>

          <div className="flex items-center gap-3">
            <button
              onClick={() => setIsQuickAppOpen(true)}
              className="flex items-center gap-2 px-3 py-2 text-xs font-medium text-white bg-indigo-600/80 hover:bg-indigo-600 rounded-lg shadow-sm transition-colors cursor-pointer"
            >
              <Sparkles className="w-3.5 h-3.5" />
              <span>Quick App</span>
            </button>

            <button
              onClick={handleStartAll}
              disabled={actionLoading === "start_all"}
              className="flex items-center gap-2 px-4 py-2 text-xs font-medium text-white bg-[var(--color-primary)] hover:bg-[var(--color-primary-hover)] disabled:opacity-50 rounded-lg shadow-sm transition-colors cursor-pointer"
            >
              <Play className="w-3.5 h-3.5 fill-current" />
              <span>Start All</span>
            </button>

            <button
              onClick={handleStopAll}
              disabled={actionLoading === "stop_all"}
              className="flex items-center gap-2 px-4 py-2 text-xs font-medium text-[var(--color-text-secondary)] hover:text-white bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-lg transition-colors cursor-pointer"
            >
              <Square className="w-3.5 h-3.5 fill-current" />
              <span>Stop All</span>
            </button>
          </div>
        </div>

        {/* Service Cards Grid */}
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <h2 className="text-xs font-semibold uppercase tracking-wider text-[var(--color-text-muted)] m-0">
              Core Services
            </h2>
            <span className="text-[11px] text-[var(--color-text-muted)]">
              Primary: <strong className="text-white">{activeWebServer.toUpperCase()}</strong> (port 80)
            </span>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {serviceList.map((service) => {
              const isRunning = service.status === "running";
              const isConflict = service.status === "conflict";
              const isError = service.status === "error";
              const isInactiveWebServer =
                (service.id === "nginx" && activeWebServer !== "nginx") ||
                (service.id === "apache" && activeWebServer !== "apache");

              return (
                <div
                  key={service.id}
                  className={`flex flex-col justify-between p-4 rounded-xl border transition-colors ${
                    isInactiveWebServer
                      ? "border-[var(--color-border-subtle)] bg-[var(--color-bg-base)]/50 opacity-60"
                      : "border-[var(--color-border)] bg-[var(--color-bg-surface)] hover:border-[var(--color-bg-hover)]"
                  }`}
                >
                  <div className="flex items-start justify-between">
                    <div>
                      <div className="flex items-center gap-2">
                        <Server className="w-4 h-4 text-[var(--color-text-muted)]" />
                        <span className="text-sm font-semibold text-white">{service.name}</span>
                        {isInactiveWebServer && (
                          <span className="text-[10px] px-1.5 py-0.5 rounded bg-zinc-800 text-zinc-400">
                            Standby
                          </span>
                        )}
                      </div>
                      <div className="flex items-center gap-2 mt-1">
                        {service.port && (
                          <span className="text-[11px] font-mono text-[var(--color-text-muted)]">
                            Port {service.port}
                          </span>
                        )}
                        {service.pid && (
                          <span className="text-[11px] font-mono text-[var(--color-text-muted)]">
                            PID {service.pid}
                          </span>
                        )}
                      </div>
                    </div>

                    {/* Status Badge */}
                    <span
                      className={`inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[11px] font-medium ${
                        isRunning
                          ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20"
                          : isConflict
                          ? "bg-amber-500/10 text-amber-400 border border-amber-500/20"
                          : isError
                          ? "bg-rose-500/10 text-rose-400 border border-rose-500/20"
                          : "bg-zinc-500/10 text-zinc-400 border border-zinc-500/20"
                      }`}
                    >
                      <span
                        className={`w-1.5 h-1.5 rounded-full ${
                          isRunning
                            ? "bg-emerald-400"
                            : isConflict
                            ? "bg-amber-400"
                            : isError
                            ? "bg-rose-400"
                            : "bg-zinc-400"
                        }`}
                      />
                      {service.status}
                    </span>
                  </div>

                  {/* Conflict / Error Diagnostics */}
                  {service.error && (
                    <div className="mt-3 p-2 text-[11px] rounded bg-rose-500/10 border border-rose-500/20 text-rose-300 flex items-start gap-1.5">
                      <AlertTriangle className="w-3.5 h-3.5 flex-shrink-0 mt-0.5" />
                      <span>{service.error}</span>
                    </div>
                  )}

                  {/* Action Buttons */}
                  <div className="flex items-center gap-2 mt-4 pt-3 border-t border-[var(--color-border-subtle)]">
                    {!isRunning ? (
                      <button
                        onClick={() => handleStartService(service.id)}
                        disabled={actionLoading === service.id}
                        className="flex-1 flex items-center justify-center gap-1.5 py-1 text-xs font-medium text-white bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-md transition-colors cursor-pointer"
                      >
                        <Play className="w-3 h-3" />
                        <span>Start</span>
                      </button>
                    ) : (
                      <>
                        <button
                          onClick={() => handleStopService(service.id)}
                          disabled={actionLoading === service.id}
                          className="flex-1 flex items-center justify-center gap-1.5 py-1 text-xs font-medium text-zinc-300 hover:text-white bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-md transition-colors cursor-pointer"
                        >
                          <Square className="w-3 h-3" />
                          <span>Stop</span>
                        </button>
                        <button
                          onClick={() => handleRestartService(service.id)}
                          disabled={actionLoading === service.id}
                          className="p-1 text-zinc-400 hover:text-white bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-md transition-colors cursor-pointer"
                          title="Restart"
                        >
                          <RotateCw className="w-3.5 h-3.5" />
                        </button>
                      </>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Sites Section */}
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <h2 className="text-xs font-semibold uppercase tracking-wider text-[var(--color-text-muted)] m-0">
              Virtual Hosts & Projects ({sites.length})
            </h2>

            <div className="flex items-center gap-2">
              <button
                onClick={() => setIsQuickAppOpen(true)}
                className="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium text-white bg-indigo-600/80 hover:bg-indigo-600 border border-indigo-500/30 rounded-md transition-colors cursor-pointer"
              >
                <Sparkles className="w-3 h-3" />
                <span>Quick App</span>
              </button>
              <button
                onClick={handleAddLinkedFolder}
                className="flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium text-white bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-md transition-colors cursor-pointer"
              >
                <Plus className="w-3 h-3" />
                <span>Link Folder</span>
              </button>
              <button
                onClick={fetchSites}
                className="p-1 text-[var(--color-text-muted)] hover:text-white bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-md transition-colors cursor-pointer"
                title="Refresh sites"
              >
                <RefreshCw className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>

          <div className="overflow-hidden rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-surface)]">
            <table className="w-full text-left text-xs border-collapse">
              <thead>
                <tr className="border-b border-[var(--color-border)] text-[var(--color-text-muted)] bg-[var(--color-bg-subtle)]">
                  <th className="py-2.5 px-4 font-medium">Domain</th>
                  <th className="py-2.5 px-4 font-medium">Document Root</th>
                  <th className="py-2.5 px-4 font-medium">PHP Version</th>
                  <th className="py-2.5 px-4 font-medium">SSL</th>
                  <th className="py-2.5 px-4 font-medium text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-[var(--color-border-subtle)]">
                {filteredSites.length === 0 ? (
                  <tr>
                    <td colSpan={5} className="py-8 text-center text-[var(--color-text-muted)]">
                      No project folders found in <code className="text-white">www/</code>. Click <strong>Quick App</strong> to create one!
                    </td>
                  </tr>
                ) : (
                  filteredSites.map((site) => (
                    <tr key={site.id} className="hover:bg-[var(--color-bg-subtle)]/50 transition-colors">
                      <td className="py-3 px-4 font-mono font-medium text-white">
                        <a
                          href={`http://${site.domain}`}
                          onClick={(e) => {
                            e.preventDefault();
                            openUrl(`http://${site.domain}`);
                          }}
                          className="hover:underline flex items-center gap-1.5 text-indigo-400"
                        >
                          {site.domain}
                          <ExternalLink className="w-3 h-3 opacity-60" />
                        </a>
                      </td>
                      <td className="py-3 px-4 text-[var(--color-text-secondary)] font-mono text-[11px] truncate max-w-xs">
                        {site.documentRoot}
                      </td>
                      <td className="py-3 px-4">
                        <select
                          value={site.phpVersion}
                          onChange={(e) => handleSetSitePhpVersion(site.id, e.target.value)}
                          className="bg-[var(--color-bg-subtle)] text-xs text-white border border-[var(--color-border)] rounded px-2 py-1 outline-none hover:border-[var(--color-primary)] transition-colors cursor-pointer"
                        >
                          {availablePhpVersions.map((v) => (
                            <option key={v} value={v} className="bg-[var(--color-bg-surface)] text-white">
                              {v === "default" ? "default (Pool 9000)" : `${v} (Pool)`}
                            </option>
                          ))}
                        </select>
                      </td>
                      <td className="py-3 px-4">
                        {site.hasSsl ? (
                          <span className="inline-flex items-center gap-1 text-[11px] text-emerald-400">
                            <ShieldCheck className="w-3.5 h-3.5" />
                            <span>HTTPS</span>
                          </span>
                        ) : (
                          <span className="text-[11px] text-[var(--color-text-muted)]">HTTP</span>
                        )}
                      </td>
                      <td className="py-3 px-4 text-right">
                        <div className="flex items-center justify-end gap-1.5">
                          <button
                            onClick={() => openUrl(`http://${site.domain}`)}
                            className="p-1.5 rounded hover:bg-[var(--color-bg-hover)] text-zinc-400 hover:text-white transition-colors"
                            title="Open in Browser"
                          >
                            <ExternalLink className="w-3.5 h-3.5" />
                          </button>
                          <button
                            onClick={() => openPath(site.path)}
                            className="p-1.5 rounded hover:bg-[var(--color-bg-hover)] text-zinc-400 hover:text-white transition-colors"
                            title="Open Project Folder"
                          >
                            <FolderOpen className="w-3.5 h-3.5" />
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      </main>

      {/* Live Log Console Drawer */}
      <div className="border-t border-[var(--color-border)] bg-[var(--color-bg-surface)]">
        <div
          className="flex items-center justify-between px-6 py-2 cursor-pointer bg-[var(--color-bg-subtle)]"
          onClick={() => setIsLogOpen(!isLogOpen)}
        >
          <div className="flex items-center gap-2 text-xs font-semibold text-[var(--color-text-secondary)]">
            <Terminal className="w-3.5 h-3.5 text-indigo-400" />
            <span>Live Activity & Process Logs ({logs.length})</span>
          </div>
          <div className="flex items-center gap-2">
            {isLogOpen ? <ChevronDown className="w-4 h-4" /> : <ChevronUp className="w-4 h-4" />}
          </div>
        </div>

        {isLogOpen && (
          <div className="p-4 bg-[var(--color-bg-base)]">
            <div className="flex items-center gap-2 mb-2">
              {["all", "nginx", "apache", "php", "mysql", "mailpit", "supervisor", "tools"].map((f) => (
                <button
                  key={f}
                  onClick={() => setSelectedLogFilter(f)}
                  className={`px-2 py-0.5 text-[11px] font-medium rounded ${
                    selectedLogFilter === f
                      ? "bg-[var(--color-primary)] text-white"
                      : "bg-[var(--color-bg-subtle)] text-[var(--color-text-muted)] hover:text-white"
                  }`}
                >
                  {f.toUpperCase()}
                </button>
              ))}
              <button
                onClick={() => setLogs([])}
                className="ml-auto text-[11px] text-[var(--color-text-muted)] hover:text-white"
              >
                Clear
              </button>
            </div>

            <div className="h-44 overflow-y-auto font-mono text-[11px] space-y-1 p-2.5 rounded bg-[var(--color-bg-surface)] border border-[var(--color-border)]">
              {filteredLogs.length === 0 ? (
                <div className="text-[var(--color-text-muted)]">No log messages yet.</div>
              ) : (
                filteredLogs.map((l) => (
                  <div key={l.id} className="flex items-start gap-2">
                    <span className="text-[var(--color-text-muted)] select-none">[{l.time}]</span>
                    <span className="text-indigo-400 font-semibold uppercase">[{l.service}]</span>
                    <span className={l.level === "error" ? "text-rose-400" : "text-zinc-300"}>{l.msg}</span>
                  </div>
                ))
              )}
              <div ref={logEndRef} />
            </div>
          </div>
        )}
      </div>

      {/* PHP Settings & Extensions Modal */}
      {isPhpModalOpen && phpConfig && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4">
          <div className="w-full max-w-2xl bg-[var(--color-bg-surface)] border border-[var(--color-border)] rounded-xl shadow-2xl flex flex-col max-h-[85vh]">
            <div className="flex items-center justify-between p-4 border-b border-[var(--color-border)]">
              <div className="flex items-center gap-2">
                <Settings className="w-5 h-5 text-cyan-400" />
                <h3 className="text-sm font-semibold text-white m-0">PHP Configuration Manager</h3>
              </div>
              <button
                onClick={() => setIsPhpModalOpen(false)}
                className="p-1 rounded hover:bg-[var(--color-bg-subtle)] text-zinc-400 hover:text-white"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            {/* Modal Tabs */}
            <div className="flex border-b border-[var(--color-border)] bg-[var(--color-bg-subtle)] px-4">
              <button
                onClick={() => setActivePhpTab("extensions")}
                className={`py-2 px-3 text-xs font-medium border-b-2 transition-colors cursor-pointer ${
                  activePhpTab === "extensions"
                    ? "border-[var(--color-primary)] text-white"
                    : "border-transparent text-[var(--color-text-muted)] hover:text-white"
                }`}
              >
                Extensions ({phpConfig.extensions.filter((e) => e.enabled).length} active)
              </button>
              <button
                onClick={() => setActivePhpTab("directives")}
                className={`py-2 px-3 text-xs font-medium border-b-2 transition-colors cursor-pointer ${
                  activePhpTab === "directives"
                    ? "border-[var(--color-primary)] text-white"
                    : "border-transparent text-[var(--color-text-muted)] hover:text-white"
                }`}
              >
                Directives & Limits
              </button>
            </div>

            {/* Modal Tab Content */}
            <div className="flex-1 overflow-y-auto p-4 space-y-4">
              {activePhpTab === "extensions" ? (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-2.5">
                  {phpConfig.extensions.map((ext) => (
                    <div
                      key={ext.name}
                      className="flex items-center justify-between p-3 rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-bg-subtle)]"
                    >
                      <div>
                        <div className="font-mono text-xs font-semibold text-white">{ext.name}</div>
                        <div className="text-[11px] text-[var(--color-text-muted)]">{ext.description}</div>
                      </div>
                      <button
                        onClick={() => handleToggleExtension(ext.name, ext.enabled)}
                        className={`w-9 h-5 rounded-full p-0.5 transition-colors cursor-pointer ${
                          ext.enabled ? "bg-emerald-500" : "bg-zinc-700"
                        }`}
                      >
                        <div
                          className={`w-4 h-4 rounded-full bg-white transition-transform ${
                            ext.enabled ? "translate-x-4" : "translate-x-0"
                          }`}
                        />
                      </button>
                    </div>
                  ))}
                </div>
              ) : (
                <div className="space-y-3">
                  {phpConfig.directives.map((dir) => (
                    <div
                      key={dir.key}
                      className="p-3 rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-bg-subtle)] space-y-2"
                    >
                      <div className="flex items-center justify-between">
                        <div>
                          <span className="font-mono text-xs font-semibold text-white">{dir.key}</span>
                          <p className="text-[11px] text-[var(--color-text-muted)] m-0">{dir.description}</p>
                        </div>
                        <button
                          onClick={() => handleSaveDirective(dir.key)}
                          className="px-2.5 py-1 text-[11px] font-medium text-white bg-[var(--color-primary)] hover:bg-[var(--color-primary-hover)] rounded transition-colors cursor-pointer"
                        >
                          Save
                        </button>
                      </div>
                      <input
                        type="text"
                        value={directiveEdits[dir.key] ?? dir.value}
                        onChange={(e) =>
                          setDirectiveEdits((prev) => ({ ...prev, [dir.key]: e.target.value }))
                        }
                        className="w-full bg-[var(--color-bg-base)] border border-[var(--color-border)] rounded px-3 py-1.5 text-xs text-white outline-none focus:border-[var(--color-primary)] font-mono"
                      />
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div className="p-3 border-t border-[var(--color-border)] bg-[var(--color-bg-subtle)] text-[11px] text-[var(--color-text-muted)] flex items-center justify-between">
              <span>Config: <code className="text-zinc-300 font-mono">{phpConfig.iniPath}</code></span>
              <span>Changes reload PHP-FPM automatically.</span>
            </div>
          </div>
        </div>
      )}

      {/* Quick App Scaffolder Modal */}
      {isQuickAppOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4">
          <div className="w-full max-w-lg bg-[var(--color-bg-surface)] border border-[var(--color-border)] rounded-xl shadow-2xl flex flex-col p-5 space-y-4">
            <div className="flex items-center justify-between border-b border-[var(--color-border)] pb-3">
              <div className="flex items-center gap-2">
                <Sparkles className="w-5 h-5 text-indigo-400" />
                <h3 className="text-sm font-semibold text-white m-0">Quick App Scaffolder</h3>
              </div>
              <button
                onClick={() => setIsQuickAppOpen(false)}
                className="p-1 rounded hover:bg-[var(--color-bg-subtle)] text-zinc-400 hover:text-white"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <form onSubmit={handleScaffoldSubmit} className="space-y-4">
              <div className="space-y-2">
                <label className="text-xs font-semibold text-[var(--color-text-secondary)]">
                  Project Type
                </label>
                <div className="grid grid-cols-3 gap-2">
                  {[
                    { id: "blank", name: "Blank PHP", icon: Globe, desc: "Lightweight" },
                    { id: "wordpress", name: "WordPress", icon: Box, desc: "CMS Ready" },
                    { id: "laravel", name: "Laravel", icon: Layers, desc: "Modern MVC" },
                  ].map((t) => (
                    <button
                      key={t.id}
                      type="button"
                      onClick={() => setScaffoldType(t.id as any)}
                      className={`flex flex-col items-center justify-center p-3 rounded-lg border text-center transition-colors cursor-pointer ${
                        scaffoldType === t.id
                          ? "border-indigo-500 bg-indigo-500/10 text-white"
                          : "border-[var(--color-border)] bg-[var(--color-bg-subtle)] text-[var(--color-text-secondary)] hover:text-white"
                      }`}
                    >
                      <t.icon className="w-5 h-5 mb-1.5 text-indigo-400" />
                      <span className="text-xs font-semibold">{t.name}</span>
                      <span className="text-[10px] text-[var(--color-text-muted)]">{t.desc}</span>
                    </button>
                  ))}
                </div>
              </div>

              <div className="space-y-1.5">
                <label className="text-xs font-semibold text-[var(--color-text-secondary)]">
                  Project Name
                </label>
                <input
                  type="text"
                  required
                  placeholder="e.g. my-awesome-app"
                  value={scaffoldName}
                  onChange={(e) => setScaffoldName(e.target.value)}
                  className="w-full bg-[var(--color-bg-base)] border border-[var(--color-border)] rounded-lg px-3 py-2 text-sm text-white outline-none focus:border-indigo-500 font-mono"
                />
                <span className="text-[11px] text-[var(--color-text-muted)]">
                  Will map to <code className="text-indigo-400 font-mono">{scaffoldName ? `${scaffoldName.toLowerCase().replace(/ /g, '-')}.test` : 'project.test'}</code> and auto-create MySQL database.
                </span>
              </div>

              <div className="flex justify-end gap-2 pt-2 border-t border-[var(--color-border)]">
                <button
                  type="button"
                  onClick={() => setIsQuickAppOpen(false)}
                  className="px-3 py-1.5 text-xs text-[var(--color-text-secondary)] hover:text-white bg-[var(--color-bg-subtle)] border border-[var(--color-border)] rounded-md transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={scaffoldLoading}
                  className="px-4 py-1.5 text-xs font-semibold text-white bg-indigo-600 hover:bg-indigo-500 rounded-md transition-colors disabled:opacity-50 flex items-center gap-1.5 cursor-pointer"
                >
                  {scaffoldLoading ? <RotateCw className="w-3.5 h-3.5 animate-spin" /> : <Check className="w-3.5 h-3.5" />}
                  <span>Provision Project</span>
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Package Hub Modal */}
      {isPackageHubOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4">
          <div className="w-full max-w-lg bg-[var(--color-bg-surface)] border border-[var(--color-border)] rounded-xl shadow-2xl flex flex-col p-5 space-y-4">
            <div className="flex items-center justify-between border-b border-[var(--color-border)] pb-3">
              <div className="flex items-center gap-2">
                <Box className="w-5 h-5 text-purple-400" />
                <h3 className="text-sm font-semibold text-white m-0">Modular Package Hub</h3>
              </div>
              <button
                onClick={() => setIsPackageHubOpen(false)}
                className="p-1 rounded hover:bg-[var(--color-bg-subtle)] text-zinc-400 hover:text-white"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <p className="text-xs text-[var(--color-text-muted)] m-0">
              Manage portable PHP binaries in <code className="text-white">bin/php/</code>. You can install pre-configured wrappers or drop in your own custom compiled builds.
            </p>

            <div className="space-y-2 max-h-72 overflow-y-auto">
              {packages.map((pkg) => (
                <div
                  key={pkg.id}
                  className="flex items-center justify-between p-3 rounded-lg border border-[var(--color-border-subtle)] bg-[var(--color-bg-subtle)]"
                >
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-semibold text-xs text-white">{pkg.name}</span>
                      <span
                        className={`text-[10px] px-1.5 py-0.5 rounded font-medium ${
                          pkg.is_installed
                            ? "bg-emerald-500/20 text-emerald-400 border border-emerald-500/30"
                            : "bg-zinc-700/50 text-zinc-400"
                        }`}
                      >
                        {pkg.is_installed ? "Installed" : "Available"}
                      </span>
                    </div>
                    <span className="text-[11px] text-[var(--color-text-muted)] font-mono">
                      Version: {pkg.version}
                    </span>
                  </div>

                  {!pkg.is_installed ? (
                    <button
                      onClick={() => handleInstallPackage(pkg.id)}
                      className="px-3 py-1 text-xs font-medium text-white bg-purple-600 hover:bg-purple-500 rounded transition-colors cursor-pointer"
                    >
                      Install
                    </button>
                  ) : (
                    <span className="text-xs text-emerald-400 font-medium flex items-center gap-1">
                      <Check className="w-3.5 h-3.5" />
                      Ready
                    </span>
                  )}
                </div>
              ))}
            </div>

            <div className="flex justify-end pt-2 border-t border-[var(--color-border)]">
              <button
                onClick={() => setIsPackageHubOpen(false)}
                className="px-3 py-1.5 text-xs text-white bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-md transition-colors"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Command Palette Modal */}
      {isCommandPaletteOpen && (
        <div className="fixed inset-0 z-50 flex items-start justify-center pt-20 bg-black/60 backdrop-blur-xs">
          <div className="w-full max-w-lg p-3 rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-surface)] shadow-2xl space-y-3">
            <div className="flex items-center gap-2 px-3 py-2 border-b border-[var(--color-border)]">
              <Search className="w-4 h-4 text-[var(--color-text-muted)]" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search virtual hosts or actions..."
                className="flex-1 bg-transparent border-none text-white text-sm outline-none placeholder:text-[var(--color-text-muted)]"
                autoFocus
              />
              <span className="text-[10px] text-[var(--color-text-muted)]">ESC to close</span>
            </div>

            <div className="max-h-64 overflow-y-auto space-y-1">
              <div className="text-[10px] uppercase font-semibold text-[var(--color-text-muted)] px-3 py-1">
                Quick Actions
              </div>
              <button
                onClick={() => {
                  handleStartAll();
                  setIsCommandPaletteOpen(false);
                }}
                className="w-full flex items-center gap-2 px-3 py-2 text-xs rounded hover:bg-[var(--color-bg-subtle)] text-white text-left cursor-pointer"
              >
                <Play className="w-3.5 h-3.5 text-emerald-400" />
                <span>Start All Services</span>
              </button>
              <button
                onClick={() => {
                  handleStopAll();
                  setIsCommandPaletteOpen(false);
                }}
                className="w-full flex items-center gap-2 px-3 py-2 text-xs rounded hover:bg-[var(--color-bg-subtle)] text-white text-left cursor-pointer"
              >
                <Square className="w-3.5 h-3.5 text-rose-400" />
                <span>Stop All Services</span>
              </button>
              <button
                onClick={() => {
                  handleSwitchWebServer(activeWebServer === "nginx" ? "apache" : "nginx");
                  setIsCommandPaletteOpen(false);
                }}
                className="w-full flex items-center gap-2 px-3 py-2 text-xs rounded hover:bg-[var(--color-bg-subtle)] text-white text-left cursor-pointer"
              >
                <Server className="w-3.5 h-3.5 text-cyan-400" />
                <span>Switch to {activeWebServer === "nginx" ? "Apache" : "Nginx"} Web Server</span>
              </button>
              <button
                onClick={() => {
                  setIsQuickAppOpen(true);
                  setIsCommandPaletteOpen(false);
                }}
                className="w-full flex items-center gap-2 px-3 py-2 text-xs rounded hover:bg-[var(--color-bg-subtle)] text-white text-left cursor-pointer"
              >
                <Sparkles className="w-3.5 h-3.5 text-indigo-400" />
                <span>Quick App Scaffolder (WordPress / Laravel / Blank)</span>
              </button>
              <button
                onClick={() => {
                  handleLaunchTerminal();
                  setIsCommandPaletteOpen(false);
                }}
                className="w-full flex items-center gap-2 px-3 py-2 text-xs rounded hover:bg-[var(--color-bg-subtle)] text-white text-left cursor-pointer"
              >
                <Terminal className="w-3.5 h-3.5 text-emerald-400" />
                <span>Open Isolated Terminal</span>
              </button>
              <button
                onClick={() => {
                  fetchPhpConfig();
                  setIsPhpModalOpen(true);
                  setIsCommandPaletteOpen(false);
                }}
                className="w-full flex items-center gap-2 px-3 py-2 text-xs rounded hover:bg-[var(--color-bg-subtle)] text-white text-left cursor-pointer"
              >
                <Settings className="w-3.5 h-3.5 text-cyan-400" />
                <span>Configure PHP Extensions & Directives</span>
              </button>
              <button
                onClick={() => {
                  handleLaunchDatabase();
                  setIsCommandPaletteOpen(false);
                }}
                className="w-full flex items-center gap-2 px-3 py-2 text-xs rounded hover:bg-[var(--color-bg-subtle)] text-white text-left cursor-pointer"
              >
                <Database className="w-3.5 h-3.5 text-amber-400" />
                <span>Open Database (Client / Adminer)</span>
              </button>
              <button
                onClick={() => {
                  openUrl("http://localhost:8025");
                  setIsCommandPaletteOpen(false);
                }}
                className="w-full flex items-center gap-2 px-3 py-2 text-xs rounded hover:bg-[var(--color-bg-subtle)] text-white text-left cursor-pointer"
              >
                <Mail className="w-3.5 h-3.5 text-indigo-400" />
                <span>Open Mailpit Local Inbox</span>
              </button>

              <div className="text-[10px] uppercase font-semibold text-[var(--color-text-muted)] px-3 py-1 mt-2">
                Virtual Hosts
              </div>
              {filteredSites.map((site) => (
                <button
                  key={site.id}
                  onClick={() => {
                    openUrl(`http://${site.domain}`);
                    setIsCommandPaletteOpen(false);
                  }}
                  className="w-full flex items-center justify-between px-3 py-2 text-xs rounded hover:bg-[var(--color-bg-subtle)] text-white text-left cursor-pointer"
                >
                  <span className="font-mono text-indigo-400">{site.domain}</span>
                  <span className="text-[11px] text-[var(--color-text-muted)]">{site.name}</span>
                </button>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
