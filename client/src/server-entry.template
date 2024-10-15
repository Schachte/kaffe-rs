import React from "react";
import "fast-text-encoding";
import ReactDOMServer from "react-dom/server";

%{{ REPLACE_IMPORTS }}%

(globalThis as any).renderToString = (location = "/") => {
  const components = %{{ REPLACE_COMPONENTS }}%;
  return ReactDOMServer.renderToString(
    <div id="root">%{{ REPLACE_CONTENT }}%</div>
  );
};
