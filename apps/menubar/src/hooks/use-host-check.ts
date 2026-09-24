import { useEffect, useState } from "react";
import { api } from "../lib/api";
import { usePollInterval } from "./use-poll-interval";
import type { HostCheck } from "../types";

const DEBOUNCE_MS = 400;

/**
 * Live reachability/ping check for whatever host is currently selected on
 * the New Tunnel form — debounced on change (covers free-typed host entry)
 * and re-checked on the shared poll interval so it stays current while the
 * form is open, same cadence as the tunnel list's own polling.
 */
export function useHostCheck(sshHost: string) {
  const { intervalMs } = usePollInterval();
  const [check, setCheck] = useState<HostCheck | null>(null);
  const [checking, setChecking] = useState(false);

  useEffect(() => {
    if (!sshHost.trim()) {
      setCheck(null);
      setChecking(false);
      return;
    }

    let cancelled = false;
    const run = () => {
      setChecking(true);
      api
        .checkSshHost(sshHost)
        .then((result) => {
          if (!cancelled) setCheck(result);
        })
        .finally(() => {
          if (!cancelled) setChecking(false);
        });
    };

    const debounce = setTimeout(run, DEBOUNCE_MS);
    const interval = setInterval(run, intervalMs);
    return () => {
      cancelled = true;
      clearTimeout(debounce);
      clearInterval(interval);
    };
  }, [sshHost, intervalMs]);

  return { check, checking };
}
