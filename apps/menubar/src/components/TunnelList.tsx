import type { TunnelInfo } from "../types";

interface Props {
  tunnels: TunnelInfo[];
  onStop: (id: string) => void;
}

export function TunnelList({ tunnels, onStop }: Props) {
  return (
    <section className="tunnels">
      <h2>Active ({tunnels.length})</h2>
      {tunnels.length === 0 && <p className="empty">No tunnels running.</p>}
      <ul>
        {tunnels.map((t) => (
          <li key={t.id}>
            <div className="tunnel-info">
              <strong>
                {t.localPort} → {t.remoteHost}:{t.remotePort}
              </strong>
              <span>via {t.sshHost}</span>
            </div>
            <button className="stop" onClick={() => onStop(t.id)}>
              Stop
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}
