import { useEffect, useState } from "react";
import { api } from "../lib/api";
import type { OrphanedProcess } from "../types";

export function useOrphanedProcesses() {
  const [orphans, setOrphans] = useState<OrphanedProcess[]>([]);
  const [killing, setKilling] = useState(false);

  useEffect(() => {
    // Only worth checking once at launch — these are leftovers from a
    // previous crashed run, not something that appears during normal use.
    api.listOrphanedSsh().then(setOrphans);
  }, []);

  async function killAll() {
    setKilling(true);
    try {
      await Promise.all(orphans.map((o) => api.killOrphanedSsh(o.pid)));
      setOrphans([]);
    } finally {
      setKilling(false);
    }
  }

  return { orphans, killing, killAll };
}
