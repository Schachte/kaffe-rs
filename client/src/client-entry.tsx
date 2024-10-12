import * as React from "react";
import { BrowserRouter as Router } from "react-router-dom";
import { hydrateRoot } from "react-dom/client";
import Routing from "./Routing";

const container = document.getElementById("app");
if (container) {
  hydrateRoot(
    container,
    <React.StrictMode>
      <Router>
        <Routing />
      </Router>
    </React.StrictMode>
  );
} else {
  console.error("Container element with id 'app' not found");
}
