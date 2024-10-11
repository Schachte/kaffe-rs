import ReactDOMServer from "react-dom/server";
import App from "./App";

const result = ReactDOMServer.renderToString(<App />);
console.log(result);
globalThis.ssrResult = result;
