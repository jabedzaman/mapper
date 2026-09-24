import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import type { TunnelFailure } from "../types";

interface Props {
  failure: TunnelFailure;
  freeing: boolean;
  onFreeAndRetry: (failure: TunnelFailure) => void;
}

export function FailureBanner({ failure, freeing, onFreeAndRetry }: Props) {
  return (
    <Alert variant="destructive" className="gap-2">
      <AlertDescription className="break-words text-destructive">
        {failure.sshHost}:{failure.localPort} → {failure.message}
      </AlertDescription>
      {failure.portInUse && (
        <Button
          type="button"
          size="sm"
          variant="outline"
          disabled={freeing}
          className="w-fit border-amber-500/40 bg-amber-500/10 text-amber-600 hover:bg-amber-500/20 dark:text-amber-400"
          onClick={() => onFreeAndRetry(failure)}
        >
          {freeing ? "Freeing port…" : `Kill process on port ${failure.localPort} & retry`}
        </Button>
      )}
    </Alert>
  );
}
