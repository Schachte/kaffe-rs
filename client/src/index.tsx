import React from "react";
import { hydrateRoot } from "react-dom/client";
import App from "./App";

const Application = (
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
const container = document.getElementById("app")!;
hydrateRoot(container, Application);
