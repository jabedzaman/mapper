import { useEffect, useState } from "react";
import { ChevronLeft } from "lucide-react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Button } from "@/components/ui/button";
import { api } from "../lib/api";

interface Props {
  onBack: () => void;
}

const components = {
  h1: ({ children }: { children?: React.ReactNode }) => (
    <p className="mt-3 text-xs font-semibold first:mt-0">{children}</p>
  ),
  h2: ({ children }: { children?: React.ReactNode }) => (
    <p className="mt-3 text-xs font-semibold first:mt-0">{children}</p>
  ),
  h3: ({ children }: { children?: React.ReactNode }) => (
    <p className="mt-2 text-[11px] font-medium text-muted-foreground">{children}</p>
  ),
  ul: ({ children }: { children?: React.ReactNode }) => <div className="pl-3">{children}</div>,
  li: ({ children }: { children?: React.ReactNode }) => (
    <p className="text-[11px] text-foreground/90">
      {"– "}
      {children}
    </p>
  ),
  p: ({ children }: { children?: React.ReactNode }) => (
    <p className="text-[11px] text-muted-foreground">{children}</p>
  ),
  a: ({ href, children }: { href?: string; children?: React.ReactNode }) => (
    <a href={href} target="_blank" rel="noreferrer" className="underline hover:text-foreground">
      {children}
    </a>
  ),
  code: ({ children }: { children?: React.ReactNode }) => (
    <code className="rounded bg-muted px-1 py-0.5 text-[10px]">{children}</code>
  ),
  strong: ({ children }: { children?: React.ReactNode }) => (
    <strong className="font-medium text-foreground">{children}</strong>
  ),
};

export function ChangelogPage({ onBack }: Props) {
  const [text, setText] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api
      .getChangelog()
      .then((full) => setText(full.replace(/\n# Changelog\b[\s\S]*$/, "")))
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
        {text && (
          <ReactMarkdown remarkPlugins={[remarkGfm]} components={components}>
            {text}
          </ReactMarkdown>
        )}
      </div>
    </div>
  );
}
