import "@fontsource-variable/geist";
import "@fontsource-variable/geist-mono";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { App } from "./App";
import "./i18n";
import "./theme/global.css";
import { applyTokens } from "./theme/tokens";

const root = document.getElementById("root");
if (!root) throw new Error("missing #root");

// before first paint, so nothing flashes unstyled
applyTokens(document.documentElement);

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
