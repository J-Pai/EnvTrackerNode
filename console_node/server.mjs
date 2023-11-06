import * as build from "./build/index.js";
import express from "express";
import compression from "compression";
import fs from "fs";
import https from "https";
import remix from "@remix-run/express";
import { broadcastDevReady } from "@remix-run/node";

const app = express();

app.use(compression());

app.use(express.static("public"));

app.all(
    "*",
    remix.createRequestHandler({
        build: build,
        getLoadContext() {}
    }),
)

const server = https.createServer(
    {
        key: fs.readFileSync("certificates/console-node.key"),
        cert: fs.readFileSync("certificates/console-node.crt"),
    },
    app,
);

let port = process.env.PORT || 3443;
server.listen(port, () => {
    if (process.env.NODE_ENV === "development") {
      broadcastDevReady(build);
    }
    console.log(`[server.js]: https://localhost:${port}`);
});
