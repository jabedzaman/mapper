import { useEffect, useState } from "react";

const STORAGE_KEY = "mapper.pollIntervalMs";
export const DEFAULT_POLL_INTERVAL_MS = 1000;
export const MIN_POLL_INTERVAL_MS = 1000;
export const MAX_POLL_INTERVAL_MS = 60_000;

function readStored(): number {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    const parsed = raw ? Number(raw) : NaN;
    if (Number.isFinite(parsed)) {
      return Math.min(MAX_POLL_INTERVAL_MS, Math.max(MIN_POLL_INTERVAL_MS, parsed));
    }
  } catch {
    // fall through to default — private window or blocked storage
  }
  return DEFAULT_POLL_INTERVAL_MS;
}

/**
 * The refresh cadence for tunnel status/latency polling and the New Tunnel
 * host check, shared across the app and configurable from Settings.
 * Persisted per-viewer in localStorage; every component reading it also
 * listens for the "storage" event so changing it in Settings applies
 * immediately elsewhere without a restart.
 */
export function usePollInterval() {
  const [intervalMs, setIntervalMsState] = useState(readStored);

  useEffect(() => {
    function onStorage(e: StorageEvent) {
      if (e.key === STORAGE_KEY) setIntervalMsState(readStored());
    }
    window.addEventListener("storage", onStorage);
    return () => window.removeEventListener("storage", onStorage);
  }, []);

  function setIntervalMs(next: number) {
    const clamped = Math.min(MAX_POLL_INTERVAL_MS, Math.max(MIN_POLL_INTERVAL_MS, next));
    setIntervalMsState(clamped);
    try {
      localStorage.setItem(STORAGE_KEY, String(clamped));
    } catch {
      // best-effort only
    }
    // "storage" only fires in OTHER tabs/windows, not this one — dispatch
    // manually so other hook instances in this same window pick it up too.
    window.dispatchEvent(new StorageEvent("storage", { key: STORAGE_KEY }));
  }

  return { intervalMs, setIntervalMs };
}
