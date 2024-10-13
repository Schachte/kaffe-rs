import React from "react";
        import "fast-text-encoding";
        import ReactDOMServer from "react-dom/server";
        import Home from "./components/Home";

        // Expose the renderToString function globally
        (globalThis as any).renderToString = (location = "/") => {
        return ReactDOMServer.renderToString(
            <React.Fragment>
            <Home />
            </React.Fragment>
        );
        };
        