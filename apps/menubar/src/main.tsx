import React from "react";
import ReactDOM from "react-dom/client";
import App from "./app";
import "./index.css";

// shadcn's dark tokens are gated behind a `.dark` class rather than
// prefers-color-scheme, so sync it with the system theme ourselves.
const darkMedia = window.matchMedia("(prefers-color-scheme: dark)");
const syncColorScheme = () => document.documentElement.classList.toggle("dark", darkMedia.matches);
syncColorScheme();
darkMedia.addEventListener("change", syncColorScheme);

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
