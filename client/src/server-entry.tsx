import React from "react";
import "fast-text-encoding";
import ReactDOMServer from "react-dom/server";
import { StaticRouter } from "react-router-dom/server";
import Routing from "./Routing";

export const renderSSR = (location: string = "/") => {
  const result = ReactDOMServer.renderToString(
    <StaticRouter location={location}>
      <Routing />
    </StaticRouter>
  );
  globalThis.ssrResult = result;
  return result;
};

globalThis.renderSSR = renderSSR;
