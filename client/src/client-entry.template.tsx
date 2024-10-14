import React from "react";
import { hydrateRoot } from "react-dom/client";
const container = document.getElementById("root");

%{{ REPLACE_IMPORTS }}%
const components = %{{ REPLACE_COMPONENTS }}%;
if (container) {
  hydrateRoot(container, <React.Fragment>%{{ REPLACE_CONTENT }}%</React.Fragment>);
} else {
  console.error("Container element with id 'root' not found");
}
