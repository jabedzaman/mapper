import { useState } from "react";
import { Header } from "./components/Header";
import { AddTunnelPage } from "./components/AddTunnelPage";
import { FailureBanner } from "./components/FailureBanner";
import { OrphanBanner } from "./components/OrphanBanner";
import { TunnelList } from "./components/TunnelList";
import { useSshHosts } from "./hooks/useSshHosts";
import { useTunnels } from "./hooks/useTunnels";
import { useOrphanedProcesses } from "./hooks/useOrphanedProcesses";
import type { StartTunnelSpec } from "./lib/api";

type View = "home" | "add";

export default function App() {
  const hosts = useSshHosts();
  const {
    tunnels,
    error,
    setError,
    failure,
    starting,
    freeing,
    start,
    startSaved,
    stop,
    remove,
    freeAndRetry,
    exportToFile,
    importFromFile,
  } = useTunnels();
  const { orphans, killing, killAll } = useOrphanedProcesses();
  const [view, setView] = useState<View>("home");

  async function handleAdd(spec: StartTunnelSpec) {
    const ok = await start(spec);
    if (ok) setView("home");
    return ok;
  }

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-background text-foreground">
      <Header onAdd={() => setView("add")} />

      {failure && (
        <div className="px-4 pt-3">
          <FailureBanner failure={failure} freeing={freeing} onFreeAndRetry={freeAndRetry} />
        </div>
      )}

      {view === "add" ? (
        <AddTunnelPage
          hosts={hosts}
          starting={starting}
          onSubmit={handleAdd}
          onValidationError={setError}
          onBack={() => setView("home")}
        >
          {error && <p className="text-xs text-destructive">{error}</p>}
        </AddTunnelPage>
      ) : (
        <>
          <OrphanBanner orphans={orphans} killing={killing} onKillAll={killAll} />
          <TunnelList
            tunnels={tunnels}
            onStart={startSaved}
            onStop={stop}
            onDelete={remove}
            onExport={exportToFile}
            onImport={importFromFile}
          />
        </>
      )}
    </div>
  );
}
