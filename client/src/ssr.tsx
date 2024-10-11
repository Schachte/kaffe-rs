import "fast-text-encoding"; // Mandatory for React18
import ReactDOMServer from "react-dom/server";
import App from "./App";

const result = ReactDOMServer.renderToString(<App />);
globalThis.ssrResult = result;
