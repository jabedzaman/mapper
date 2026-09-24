import { Header } from "./components/Header";
import { TunnelForm } from "./components/TunnelForm";
import { FailureBanner } from "./components/FailureBanner";
import { TunnelList } from "./components/TunnelList";
import { useSshHosts } from "./hooks/useSshHosts";
import { useTunnels } from "./hooks/useTunnels";

export default function App() {
  const hosts = useSshHosts();
  const { tunnels, error, setError, failure, starting, freeing, start, stop, freeAndRetry } =
    useTunnels();

  return (
    <div className="app">
      <Header />

      <TunnelForm hosts={hosts} starting={starting} onSubmit={start} onValidationError={setError}>
        {error && <p className="error">{error}</p>}
        {failure && (
          <FailureBanner failure={failure} freeing={freeing} onFreeAndRetry={freeAndRetry} />
        )}
      </TunnelForm>

      <TunnelList tunnels={tunnels} onStop={stop} />
    </div>
  );
}
