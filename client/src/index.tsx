import React from "react";
import { hydrateRoot } from "react-dom/client";
import App from "./App";

const container = document.getElementById("app");

if (container) {
  hydrateRoot(
    container,
    <React.StrictMode>
      <div className="app">
        <App />
      </div>
    </React.StrictMode>
  );
} else {
  console.error("Container element with id 'app' not found");
}
