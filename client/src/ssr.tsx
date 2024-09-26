import React from "react";
import ReactDOMServer from "react-dom/server";
import App from "./App";

const result = ReactDOMServer.renderToString(<App />);
(globalThis as any).ssrResult = result;
