import { Header } from "./components/Header";
import { TunnelForm } from "./components/TunnelForm";
import { FailureBanner } from "./components/FailureBanner";
import { OrphanBanner } from "./components/OrphanBanner";
import { TunnelList } from "./components/TunnelList";
import { useSshHosts } from "./hooks/useSshHosts";
import { useTunnels } from "./hooks/useTunnels";
import { useOrphanedProcesses } from "./hooks/useOrphanedProcesses";

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

  return (
    <div className="flex h-screen flex-col bg-background text-foreground">
      <Header />

      <OrphanBanner orphans={orphans} killing={killing} onKillAll={killAll} />

      <TunnelForm hosts={hosts} starting={starting} onSubmit={start} onValidationError={setError}>
        {error && <p className="text-xs text-destructive">{error}</p>}
        {failure && (
          <FailureBanner failure={failure} freeing={freeing} onFreeAndRetry={freeAndRetry} />
        )}
      </TunnelForm>

      <TunnelList
        tunnels={tunnels}
        onStart={startSaved}
        onStop={stop}
        onDelete={remove}
        onExport={exportToFile}
        onImport={importFromFile}
      />
    </div>
  );
}
