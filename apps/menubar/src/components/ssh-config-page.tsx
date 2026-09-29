import { useEffect, useState } from "react";
import { ChevronLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { SshHostList } from "./ssh-host-list";
import { SshHostForm } from "./ssh-host-form";
import { api } from "../lib/api";
import type { ConfigDoc, HostBlock } from "../types";

interface Props {
  onBack: () => void;
  onSaved: () => void;
}

type SubView = "list" | "edit" | "raw";

export function SshConfigPage({ onBack, onSaved }: Props) {
  const [doc, setDoc] = useState<ConfigDoc | null>(null);
  const [raw, setRaw] = useState<string | null>(null);
  const [subview, setSubview] = useState<SubView>("list");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  function load() {
    Promise.all([api.getSshConfigDoc(), api.getSshConfig()])
      .then(([d, r]) => {
        setDoc(d);
        setRaw(r);
      })
      .catch((e) => setError(String(e)));
  }

  useEffect(load, []);

  const editingHost = editingId ? (doc?.hosts.find((h) => h.id === editingId) ?? null) : null;

  async function saveHost(host: HostBlock) {
    if (!doc) return;
    setSaving(true);
    setError(null);
    const hosts = editingId
      ? doc.hosts.map((h) => (h.id === editingId ? host : h))
      : [...doc.hosts, { ...host, id: crypto.randomUUID() }];
    const next = { ...doc, hosts };
    try {
      await api.saveSshConfigDoc(next);
      setDoc(next);
      const r = await api.getSshConfig();
      setRaw(r);
      onSaved();
      setSubview("list");
      setEditingId(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  }

  async function deleteHost(id: string) {
    if (!doc) return;
    const next = { ...doc, hosts: doc.hosts.filter((h) => h.id !== id) };
    setError(null);
    try {
      await api.saveSshConfigDoc(next);
      setDoc(next);
      const r = await api.getSshConfig();
      setRaw(r);
      onSaved();
    } catch (e) {
      setError(String(e));
    }
  }

  async function saveRaw(contents: string) {
    setSaving(true);
    setError(null);
    try {
      await api.saveSshConfig(contents);
      setRaw(contents);
      const d = await api.getSshConfigDoc();
      setDoc(d);
      onSaved();
      setSubview("list");
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  }

  if (subview === "edit") {
    return (
      <SshHostForm
        initial={editingHost}
        saving={saving}
        error={error}
        onSubmit={saveHost}
        onBack={() => {
          setSubview("list");
          setEditingId(null);
          setError(null);
        }}
      />
    );
  }

  if (subview === "raw") {
    return (
      <RawEditor
        initial={raw ?? ""}
        saving={saving}
        error={error}
        onSave={saveRaw}
        onBack={() => {
          setSubview("list");
          setError(null);
        }}
      />
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center justify-between gap-1 px-2 py-2.5">
        <div className="flex items-center gap-1">
          <Button variant="ghost" size="icon-sm" title="Back" onClick={onBack}>
            <ChevronLeft className="size-4" />
          </Button>
          <span className="text-xs font-medium">SSH config</span>
        </div>
        <Button variant="ghost" size="sm" className="text-[11px]" onClick={() => setSubview("raw")}>
          Edit raw
        </Button>
      </div>

      <div className="flex min-h-0 flex-1 flex-col gap-2.5 overflow-y-auto px-4 pb-4">
        {error && <p className="text-xs text-destructive">{error}</p>}

        {doc === null && !error && <p className="text-xs text-muted-foreground">Loading…</p>}

        {doc !== null && (
          <>
            <SshHostList
              hosts={doc.hosts}
              onEdit={(id) => {
                setEditingId(id);
                setSubview("edit");
              }}
              onDelete={deleteHost}
              onAdd={() => {
                setEditingId(null);
                setSubview("edit");
              }}
            />
          </>
        )}
      </div>
    </div>
  );
}

function RawEditor({
  initial,
  saving,
  error,
  onSave,
  onBack,
}: {
  initial: string;
  saving: boolean;
  error: string | null;
  onSave: (contents: string) => void;
  onBack: () => void;
}) {
  const [text, setText] = useState(initial);
  const dirty = text !== initial;

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center justify-between gap-1 px-2 py-2.5">
        <div className="flex items-center gap-1">
          <Button variant="ghost" size="icon-sm" title="Back" onClick={onBack}>
            <ChevronLeft className="size-4" />
          </Button>
          <span className="text-xs font-medium">Raw SSH config</span>
        </div>
        <Button size="sm" onClick={() => onSave(text)} disabled={!dirty || saving}>
          {saving ? "Saving…" : "Save"}
        </Button>
      </div>

      <div className="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto px-4 pb-4">
        {error && <p className="text-xs text-destructive">{error}</p>}
        <textarea
          className="min-h-0 flex-1 resize-none rounded-lg border border-border bg-card p-2.5 font-mono text-[11px] leading-relaxed outline-none focus:border-ring"
          spellCheck={false}
          value={text}
          onChange={(e) => setText(e.target.value)}
        />
      </div>
    </div>
  );
}
