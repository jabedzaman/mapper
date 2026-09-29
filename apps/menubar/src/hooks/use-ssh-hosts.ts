import { useCallback, useEffect, useState } from "react";
import { api } from "../lib/api";
import type { SshHost } from "../types";

export function useSshHosts() {
  const [hosts, setHosts] = useState<SshHost[]>([]);

  const refresh = useCallback(() => {
    api.listSshHosts().then((h) => setHosts(h));
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { hosts, refresh };
}
