import { useEffect, useState } from "react";
import { api } from "../lib/api";
import type { SshHost } from "../types";

export function useSshHosts() {
  const [hosts, setHosts] = useState<SshHost[]>([]);

  useEffect(() => {
    api.listSshHosts().then((h) => setHosts(h));
  }, []);

  return hosts;
}
