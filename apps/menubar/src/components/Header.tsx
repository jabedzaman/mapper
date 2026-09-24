import { api } from "../lib/api";

export function Header() {
  return (
    <header>
      <span className="subtitle">SSH port forwarding</span>
      <button className="quit" onClick={() => api.quitApp()}>
        Quit
      </button>
    </header>
  );
}
