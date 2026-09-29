import { useState } from "react";
import { ChevronDown, ChevronLeft, ChevronUp } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { HostBlock } from "../types";

function blankHost(): HostBlock {
  return { id: "", patterns: "", hostname: "", user: "", port: "", proxyJump: "", identityFiles: "", extra: "" };
}

interface Props {
  /** Pre-fills the form for editing an existing host instead of creating one. */
  initial: HostBlock | null;
  saving: boolean;
  error: string | null;
  onSubmit: (host: HostBlock) => void;
  onBack: () => void;
}

export function SshHostForm({ initial, saving, error, onSubmit, onBack }: Props) {
  const [host, setHost] = useState<HostBlock>(initial ?? blankHost());
  const [validationError, setValidationError] = useState<string | null>(null);
  const [advancedOpen, setAdvancedOpen] = useState(
    () => !!(initial?.proxyJump || initial?.identityFiles || initial?.extra),
  );

  function set<K extends keyof HostBlock>(key: K, value: HostBlock[K]) {
    setHost((h) => ({ ...h, [key]: value }));
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!host.patterns.trim()) {
      setValidationError("give this host an alias");
      return;
    }
    setValidationError(null);
    onSubmit(host);
  }

  return (
    <div className="flex flex-1 flex-col">
      <div className="flex items-center gap-1.5 border-b border-border px-2 py-2.5">
        <Button variant="ghost" size="icon-sm" title="Back" onClick={onBack}>
          <ChevronLeft className="size-4" />
        </Button>
        <span className="text-xs font-semibold">{initial ? "Edit host" : "New host"}</span>
      </div>

      <form onSubmit={handleSubmit} className="flex flex-col gap-3 overflow-y-auto p-4">
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="host-patterns" className="text-xs text-muted-foreground">
            Alias
          </Label>
          <Input
            id="host-patterns"
            placeholder="myserver"
            value={host.patterns}
            onChange={(e) => set("patterns", e.target.value)}
          />
        </div>

        <div className="flex flex-col gap-1.5">
          <Label htmlFor="host-hostname" className="text-xs text-muted-foreground">
            HostName
          </Label>
          <Input
            id="host-hostname"
            placeholder="1.2.3.4 or real.host.name"
            value={host.hostname}
            onChange={(e) => set("hostname", e.target.value)}
          />
        </div>

        <div className="flex gap-2.5">
          <div className="flex flex-1 flex-col gap-1.5">
            <Label htmlFor="host-user" className="text-xs text-muted-foreground">
              User
            </Label>
            <Input id="host-user" value={host.user} onChange={(e) => set("user", e.target.value)} />
          </div>
          <div className="flex flex-1 flex-col gap-1.5">
            <Label htmlFor="host-port" className="text-xs text-muted-foreground">
              Port
            </Label>
            <Input
              id="host-port"
              inputMode="numeric"
              placeholder="22"
              value={host.port}
              onChange={(e) => set("port", e.target.value)}
            />
          </div>
        </div>

        <button
          type="button"
          className="flex items-center gap-1 self-start text-[11px] text-muted-foreground hover:text-foreground"
          onClick={() => setAdvancedOpen((v) => !v)}
        >
          {advancedOpen ? <ChevronUp className="size-3" /> : <ChevronDown className="size-3" />}
          Advanced
        </button>

        {advancedOpen && (
          <>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="host-proxy-jump" className="text-xs text-muted-foreground">
                ProxyJump
              </Label>
              <Input
                id="host-proxy-jump"
                placeholder="bastion"
                value={host.proxyJump}
                onChange={(e) => set("proxyJump", e.target.value)}
              />
            </div>

            <div className="flex flex-col gap-1.5">
              <Label htmlFor="host-identity-files" className="text-xs text-muted-foreground">
                IdentityFile (one per line)
              </Label>
              <textarea
                id="host-identity-files"
                className="min-h-[3.5rem] resize-none rounded-lg border border-input bg-transparent px-2.5 py-1.5 font-mono text-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
                spellCheck={false}
                value={host.identityFiles}
                onChange={(e) => set("identityFiles", e.target.value)}
              />
            </div>

            <div className="flex flex-col gap-1.5">
              <Label htmlFor="host-extra" className="text-xs text-muted-foreground">
                Other lines (comments, advanced options)
              </Label>
              <textarea
                id="host-extra"
                className="min-h-[4rem] resize-y rounded-lg border border-input bg-transparent px-2.5 py-1.5 font-mono text-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
                spellCheck={false}
                placeholder="# anything the form above doesn't cover — kept as raw lines"
                value={host.extra}
                onChange={(e) => set("extra", e.target.value)}
              />
            </div>
          </>
        )}

        {(validationError || error) && (
          <p className="text-xs text-destructive">{validationError ?? error}</p>
        )}

        <Button type="submit" disabled={saving} className="mt-1 w-full">
          {saving ? "Saving…" : initial ? "Save changes" : "Add host"}
        </Button>
      </form>
    </div>
  );
}
