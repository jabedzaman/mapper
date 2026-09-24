import { useEffect, useState } from "react";
import { ChevronLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { api } from "../lib/api";

interface Props {
  onBack: () => void;
}

/** Renders semantic-release's generated Markdown as plain styled blocks — no markdown-parser dependency needed for this narrow, predictable output shape. */
function renderLine(line: string, key: number) {
  const heading = line.match(/^(#{2,3})\s+(.*)$/);
  if (heading) {
    const [, hashes, text] = heading;
    const stripped = text.replace(/\[([^\]]+)\]\([^)]+\)/g, "$1");
    return (
      <p key={key} className={hashes.length === 2 ? "mt-3 text-xs font-semibold first:mt-0" : "mt-2 text-[11px] font-medium text-muted-foreground"}>
        {stripped}
      </p>
    );
  }

  const bullet = line.match(/^[*-]\s+(.*)$/);
  if (bullet) {
    const stripped = bullet[1]
      .replace(/\[([a-f0-9]{7,40})\]\([^)]+\)/g, "")
      .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
      .trim();
    return (
      <p key={key} className="pl-3 text-[11px] text-foreground/90">
        {"– "}
        {stripped}
      </p>
    );
  }

  if (!line.trim()) return null;
  return (
    <p key={key} className="text-[11px] text-muted-foreground">
      {line}
    </p>
  );
}

export function ChangelogPage({ onBack }: Props) {
  const [text, setText] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api
      .getChangelog()
      .then(setText)
      .catch((e) => setError(String(e)));
  }, []);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-1 px-2 py-2.5">
        <Button variant="ghost" size="icon-sm" title="Back" onClick={onBack}>
          <ChevronLeft className="size-4" />
        </Button>
        <span className="text-xs font-medium">Changelog</span>
      </div>

      <div className="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto px-4 pb-4">
        {error && <p className="text-xs text-destructive">{error}</p>}
        {!error && text === null && <p className="text-xs text-muted-foreground">Loading…</p>}
        {text?.split("\n").map((line, i) => renderLine(line, i))}
      </div>
    </div>
  );
}
