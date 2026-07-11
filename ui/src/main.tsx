import React from "react";
import ReactDOM from "react-dom/client";
import { App } from "./App";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { AgentSettingsProvider } from "./lib/agentSettingsStore";
import "./lib/fontawesome";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <AgentSettingsProvider>
        <App />
      </AgentSettingsProvider>
    </ErrorBoundary>
  </React.StrictMode>
);
