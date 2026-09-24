import { useEffect, useState } from "react";
import { api } from "../lib/api";

const POLL_INTERVAL_MS = 1500;

/** Only polls while `enabled` — no point streaming logs for a collapsed tunnel. */
export function useTunnelLog(id: string, enabled: boolean) {
  const [lines, setLines] = useState<string[]>([]);

  useEffect(() => {
    if (!enabled) return;
    const refresh = () => api.getTunnelLog(id).then(setLines);
    refresh();
    const interval = setInterval(refresh, POLL_INTERVAL_MS);
    return () => clearInterval(interval);
  }, [id, enabled]);

  return lines;
}
