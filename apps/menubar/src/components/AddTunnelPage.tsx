import { ChevronLeft } from "lucide-react";
import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { TunnelForm } from "./TunnelForm";
import type { SshHost } from "../types";
import type { StartTunnelSpec } from "../lib/api";

interface Props {
  hosts: SshHost[];
  starting: boolean;
  onSubmit: (spec: StartTunnelSpec) => Promise<boolean>;
  onValidationError: (message: string) => void;
  onBack: () => void;
  children?: ReactNode;
}

export function AddTunnelPage({
  hosts,
  starting,
  onSubmit,
  onValidationError,
  onBack,
  children,
}: Props) {
  return (
    <div className="flex flex-1 flex-col">
      <div className="flex items-center gap-1.5 border-b border-border px-2 py-2.5">
        <Button variant="ghost" size="icon-sm" title="Back" onClick={onBack}>
          <ChevronLeft className="size-4" />
        </Button>
        <span className="text-xs font-semibold">New tunnel</span>
      </div>

      <TunnelForm
        hosts={hosts}
        starting={starting}
        onSubmit={onSubmit}
        onValidationError={onValidationError}
      >
        {children}
      </TunnelForm>
    </div>
  );
}
