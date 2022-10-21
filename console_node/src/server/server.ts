export {}

import express from "express";
import type { Response } from "express";
const app = express();
const port = process.env.PORT || 3000;

// Serve statically generated frontend
app.use("/", express.static("dist"));

app.get("/v1/hello", (_, res: Response<string>) => {
  res.send("Hello World");
});

app.listen(port, () => {
  console.log(`console_node app listening on http://localhost:${port}!`);
});
