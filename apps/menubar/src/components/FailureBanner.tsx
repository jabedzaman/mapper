import type { TunnelFailure } from "../types";

interface Props {
  failure: TunnelFailure;
  freeing: boolean;
  onFreeAndRetry: (failure: TunnelFailure) => void;
}

export function FailureBanner({ failure, freeing, onFreeAndRetry }: Props) {
  return (
    <div className="failure">
      <p className="error">
        {failure.sshHost}:{failure.localPort} → {failure.message}
      </p>
      {failure.portInUse && (
        <button
          type="button"
          className="retry"
          disabled={freeing}
          onClick={() => onFreeAndRetry(failure)}
        >
          {freeing ? "Freeing port…" : `Kill process on port ${failure.localPort} & retry`}
        </button>
      )}
    </div>
  );
}
