import { useEffect, useState } from "react";
import { save, open } from "@tauri-apps/plugin-dialog";
import { api, type StartTunnelSpec } from "../lib/api";
import { usePollInterval } from "./use-poll-interval";
import type { Tunnel, TunnelFailure } from "../types";

export function useTunnels() {
  const { intervalMs } = usePollInterval();
  const [tunnels, setTunnels] = useState<Tunnel[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [failure, setFailure] = useState<TunnelFailure | null>(null);
  const [starting, setStarting] = useState(false);
  const [freeing, setFreeing] = useState(false);

  const refresh = () => {
    api.listTunnels().then(setTunnels).catch(() => {});
    api.takeTunnelFailures().then((failures) => {
      if (failures.length > 0) {
        setError(null);
        setFailure(failures[failures.length - 1]);
      }
    });
  };

  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, intervalMs);
    return () => clearInterval(interval);
  }, [intervalMs]);

  async function start(spec: StartTunnelSpec) {
    setStarting(true);
    try {
      await api.startTunnel(spec);
      setError(null);
      setFailure(null);
      await refresh();
      return true;
    } catch (err) {
      setError(String(err));
      return false;
    } finally {
      setStarting(false);
    }
  }

  async function startSaved(id: string) {
    try {
      await api.startSavedTunnel(id);
      setError(null);
      setFailure(null);
    } catch (err) {
      setError(String(err));
    } finally {
      await refresh();
    }
  }

  async function update(id: string, spec: StartTunnelSpec) {
    try {
      await api.updateTunnel(id, spec);
      setError(null);
      return true;
    } catch (err) {
      setError(String(err));
      return false;
    } finally {
      await refresh();
    }
  }

  async function stop(id: string) {
    await api.stopTunnel(id);
    await refresh();
  }

  async function remove(id: string) {
    await api.deleteTunnel(id);
    await refresh();
  }

  async function freeAndRetry(f: TunnelFailure) {
    setFreeing(true);
    try {
      await api.killProcessOnPort(f.localPort);
      await start({
        sshHost: f.sshHost,
        localPort: f.localPort,
        remoteHost: f.remoteHost,
        remotePort: f.remotePort,
      });
    } catch (err) {
      setError(String(err));
    } finally {
      setFreeing(false);
    }
  }

  async function exportToFile() {
    setError(null);
    const path = await save({
      title: "Export tunnels",
      defaultPath: "mapper-tunnels.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!path) return;
    try {
      await api.exportTunnels(path);
    } catch (err) {
      setError(String(err));
    }
  }

  async function importFromFile() {
    setError(null);
    const path = await open({
      title: "Import tunnels",
      multiple: false,
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (!path) return;
    try {
      await api.importTunnels(path as string);
      await refresh();
    } catch (err) {
      setError(String(err));
    }
  }

  return {
    tunnels,
    error,
    setError,
    failure,
    starting,
    freeing,
    start,
    startSaved,
    update,
    stop,
    remove,
    freeAndRetry,
    exportToFile,
    importFromFile,
  };
}
