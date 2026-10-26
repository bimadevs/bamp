import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
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

export default function App() {
  const [stackStatus, setStackStatus] = useState<StackStatus>({
    isRunning: false,
    services: {},
  });
  const [sites, setSites] = useState<Site[]>([]);
  const [logs, setLogs] = useState<Array<{ id: string; time: string; level: string; msg: string; service: string }>>([]);
  const [selectedLogFilter, setSelectedLogFilter] = useState("all");
  const [isLogOpen, setIsLogOpen] = useState(false);
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [actionLoading, setActionLoading] = useState<string | null>(null);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  const addLog = (service: string, msg: string, level = "info") => {
    const time = new Date().toLocaleTimeString();
    setLogs((prev) => [
      ...prev.slice(-150),
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

  useEffect(() => {
    fetchStatus();
    fetchSites();
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
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  // Global Controls
  const handleStartAll = async () => {
    setActionLoading("start_all");
    addLog("supervisor", "Starting all services...", "info");
    try {
      const status = await invoke<StackStatus>("start_all_services");
      setStackStatus(status);
      addLog("supervisor", "Start all completed.", "info");
      showToast("All services started.");
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
      showToast("Linked project folder registered.");
    } catch (err: unknown) {
      alert(`Error linking folder: ${String(err)}`);
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
      <header className="flex items-center justify-between px-6 py-4 border-b border-[var(--color-border)] bg-[var(--color-bg-surface)]">
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-[var(--color-primary-subtle)] text-[var(--color-primary)] font-bold text-lg">
            B
          </div>
          <div>
            <h1 className="text-base font-semibold tracking-tight text-white m-0">BAMP</h1>
            <p className="text-xs text-[var(--color-text-muted)] m-0">Apache / Nginx • MySQL • PHP Manager</p>
          </div>
        </div>

        {/* Global Toolbar */}
        <div className="flex items-center gap-2">
          <button
            onClick={() => setIsCommandPaletteOpen(true)}
            className="flex items-center gap-2 px-3 py-1.5 text-xs text-[var(--color-text-secondary)] bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-md transition-colors"
          >
            <Search className="w-3.5 h-3.5" />
            <span>Search sites & tools</span>
            <kbd className="px-1.5 py-0.5 text-[10px] bg-[var(--color-bg-base)] rounded border border-[var(--color-border)] text-[var(--color-text-muted)]">
              Ctrl+K
            </kbd>
          </button>

          <button
            onClick={handleInstallRootCa}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-[var(--color-text-secondary)] hover:text-white bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-md transition-colors"
            title="Install BAMP Root CA to trust store for valid HTTPS"
          >
            <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />
            <span>Install CA</span>
          </button>

          <button
            onClick={() => openUrl("http://localhost:8025")}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-[var(--color-text-secondary)] hover:text-white bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-md transition-colors"
            title="Open Mailpit local mail catcher"
          >
            <Mail className="w-3.5 h-3.5 text-indigo-400" />
            <span>Mailbox</span>
          </button>

          <button
            onClick={() => openUrl("http://adminer.test")}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-[var(--color-text-secondary)] hover:text-white bg-[var(--color-bg-subtle)] hover:bg-[var(--color-bg-hover)] border border-[var(--color-border)] rounded-md transition-colors"
            title="Open Adminer database manager"
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
              </div>
              <p className="text-xs text-[var(--color-text-muted)] m-0">
                {serviceList.filter((s) => s.status === "running").length} of {serviceList.length} services active
              </p>
            </div>
          </div>

          <div className="flex items-center gap-3">
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
          <h2 className="text-xs font-semibold uppercase tracking-wider text-[var(--color-text-muted)] m-0">
            Core Services
          </h2>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {serviceList.map((service) => {
              const isRunning = service.status === "running";
              const isConflict = service.status === "conflict";
              const isError = service.status === "error";

              return (
                <div
                  key={service.id}
                  className="flex flex-col justify-between p-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-surface)] hover:border-[var(--color-bg-hover)] transition-colors"
                >
                  <div className="flex items-start justify-between">
                    <div>
                      <div className="flex items-center gap-2">
                        <Server className="w-4 h-4 text-[var(--color-text-muted)]" />
                        <span className="text-sm font-semibold text-white">{service.name}</span>
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
                  <th className="py-2.5 px-4 font-medium">PHP</th>
                  <th className="py-2.5 px-4 font-medium">SSL</th>
                  <th className="py-2.5 px-4 font-medium text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-[var(--color-border-subtle)]">
                {filteredSites.length === 0 ? (
                  <tr>
                    <td colSpan={5} className="py-8 text-center text-[var(--color-text-muted)]">
                      No project folders found in <code className="text-white">www/</code>. Create a folder to start!
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
                      <td className="py-3 px-4 text-[var(--color-text-secondary)]">{site.phpVersion}</td>
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

      {/* Log Console Drawer */}
      <div className="border-t border-[var(--color-border)] bg-[var(--color-bg-surface)]">
        <div
          className="flex items-center justify-between px-6 py-2 cursor-pointer bg-[var(--color-bg-subtle)]"
          onClick={() => setIsLogOpen(!isLogOpen)}
        >
          <div className="flex items-center gap-2 text-xs font-semibold text-[var(--color-text-secondary)]">
            <Terminal className="w-3.5 h-3.5 text-indigo-400" />
            <span>Activity Logs ({logs.length})</span>
          </div>
          <div className="flex items-center gap-2">
            {isLogOpen ? <ChevronDown className="w-4 h-4" /> : <ChevronUp className="w-4 h-4" />}
          </div>
        </div>

        {isLogOpen && (
          <div className="p-4 bg-[var(--color-bg-base)]">
            <div className="flex items-center gap-2 mb-2">
              {["all", "supervisor", "nginx", "apache", "php", "mysql", "mailpit", "ssl"].map((f) => (
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

            <div className="h-40 overflow-y-auto font-mono text-[11px] space-y-1 p-2 rounded bg-[var(--color-bg-surface)] border border-[var(--color-border)]">
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
            </div>
          </div>
        )}
      </div>

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
                  openUrl("http://adminer.test");
                  setIsCommandPaletteOpen(false);
                }}
                className="w-full flex items-center gap-2 px-3 py-2 text-xs rounded hover:bg-[var(--color-bg-subtle)] text-white text-left cursor-pointer"
              >
                <Database className="w-3.5 h-3.5 text-amber-400" />
                <span>Open Adminer Database Manager</span>
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
