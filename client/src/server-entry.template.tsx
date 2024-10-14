import React from "react";
import "fast-text-encoding";
import ReactDOMServer from "react-dom/server";

%{{ REPLACE_IMPORTS }}%

(globalThis as any).renderToString = (location = "/") => {
  return ReactDOMServer.renderToString(
    const components = %{{ REPLACE_COMPONENTS }}%;
    <div id="root">%{{ REPLACE_CONTENT }}%</div>
  );
};
