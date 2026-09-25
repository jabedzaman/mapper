import { useState } from "react";
import { Header } from "./components/header";
import { useAppPreferences } from "./hooks/use-app-preferences";
import { AddTunnelPage } from "./components/add-tunnel-page";
import { SettingsPage } from "./components/settings-page";
import { ChangelogPage } from "./components/changelog-page";
import { FailureBanner } from "./components/failure-banner";
import { OrphanBanner } from "./components/orphan-banner";
import { TunnelList } from "./components/tunnel-list";
import { useSshHosts } from "./hooks/use-ssh-hosts";
import { useTunnels } from "./hooks/use-tunnels";
import { useOrphanedProcesses } from "./hooks/use-orphaned-processes";
import type { StartTunnelSpec } from "./lib/api";

type View = "home" | "add" | "edit" | "settings" | "changelog";

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
    update,
    stop,
    remove,
    freeAndRetry,
    exportToFile,
    importFromFile,
  } = useTunnels();
  const { orphans, killing, killAll } = useOrphanedProcesses();
  const preferences = useAppPreferences();
  const [view, setView] = useState<View>("home");
  const [editingId, setEditingId] = useState<string | null>(null);
  const editingTunnel = tunnels.find((t) => t.id === editingId) ?? null;

  async function handleAdd(spec: StartTunnelSpec) {
    const ok = await start(spec);
    if (ok) setView("home");
    return ok;
  }

  async function handleEdit(spec: StartTunnelSpec) {
    if (!editingId) return false;
    const ok = await update(editingId, spec);
    if (ok) {
      setView("home");
      setEditingId(null);
    }
    return ok;
  }

  function openEdit(id: string) {
    setEditingId(id);
    setView("edit");
  }

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-background text-foreground">
      <Header onAdd={() => setView("add")} onSettings={() => setView((v) => (v === "settings" ? "home" : "settings"))} />

      {failure && view === "home" && (
        <div className="px-4 pt-3">
          <FailureBanner failure={failure} freeing={freeing} onFreeAndRetry={freeAndRetry} />
        </div>
      )}

      {view === "add" && (
        <AddTunnelPage
          hosts={hosts}
          starting={starting}
          onSubmit={handleAdd}
          onValidationError={setError}
          onBack={() => setView("home")}
        >
          {error && <p className="text-xs text-destructive">{error}</p>}
        </AddTunnelPage>
      )}

      {view === "edit" && editingTunnel && (
        <AddTunnelPage
          hosts={hosts}
          starting={starting}
          onSubmit={handleEdit}
          onValidationError={setError}
          onBack={() => {
            setView("home");
            setEditingId(null);
          }}
          title="Edit tunnel"
          submitLabel="Save changes"
          submittingLabel="Saving…"
          initial={{
            sshHost: editingTunnel.sshHost,
            localPort: editingTunnel.localPort,
            remoteHost: editingTunnel.remoteHost,
            remotePort: editingTunnel.remotePort,
          }}
        >
          {error && <p className="text-xs text-destructive">{error}</p>}
        </AddTunnelPage>
      )}

      {view === "settings" && (
        <SettingsPage
          onBack={() => setView("home")}
          onExport={exportToFile}
          onImport={importFromFile}
          canExport={tunnels.length > 0}
          onChangelog={() => setView("changelog")}
          error={error}
          {...preferences}
        />
      )}

      {view === "changelog" && <ChangelogPage onBack={() => setView("settings")} />}

      {view === "home" && (
        <>
          <OrphanBanner orphans={orphans} killing={killing} onKillAll={killAll} />
          <TunnelList
            tunnels={tunnels}
            onStart={startSaved}
            onStop={stop}
            onDelete={remove}
            onEdit={openEdit}
          />
        </>
      )}
    </div>
  );
}
