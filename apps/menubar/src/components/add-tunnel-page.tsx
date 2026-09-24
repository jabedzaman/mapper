import { ChevronLeft } from "lucide-react";
import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { TunnelForm } from "./tunnel-form";
import type { SshHost } from "../types";
import type { StartTunnelSpec } from "../lib/api";

interface Props {
  hosts: SshHost[];
  starting: boolean;
  onSubmit: (spec: StartTunnelSpec) => Promise<boolean>;
  onValidationError: (message: string) => void;
  onBack: () => void;
  /** Pre-fills the form for editing an existing tunnel instead of creating one. */
  initial?: StartTunnelSpec;
  title?: string;
  submitLabel?: string;
  submittingLabel?: string;
  children?: ReactNode;
}

export function AddTunnelPage({
  hosts,
  starting,
  onSubmit,
  onValidationError,
  onBack,
  initial,
  title = "New tunnel",
  submitLabel,
  submittingLabel,
  children,
}: Props) {
  return (
    <div className="flex flex-1 flex-col">
      <div className="flex items-center gap-1.5 border-b border-border px-2 py-2.5">
        <Button variant="ghost" size="icon-sm" title="Back" onClick={onBack}>
          <ChevronLeft className="size-4" />
        </Button>
        <span className="text-xs font-semibold">{title}</span>
      </div>

      <TunnelForm
        hosts={hosts}
        starting={starting}
        onSubmit={onSubmit}
        onValidationError={onValidationError}
        initial={initial}
        submitLabel={submitLabel}
        submittingLabel={submittingLabel}
      >
        {children}
      </TunnelForm>
    </div>
  );
}
