// src/ssr.tsx
import ReactDOMServer from "react-dom/server";

// src/App.tsx
import { jsx } from "react/jsx-runtime";
var App = () => {
  return /* @__PURE__ */ jsx("div", { id: "app", children: /* @__PURE__ */ jsx("h1", { children: "Hello from React SSR with Rust and TypeScript!" }) });
};
var App_default = App;

// src/ssr.tsx
import { jsx as jsx2 } from "react/jsx-runtime";
var result = ReactDOMServer.renderToString(/* @__PURE__ */ jsx2(App_default, {}));
globalThis.ssrResult = result;
