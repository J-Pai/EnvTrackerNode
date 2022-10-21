export {}

const express = require("express");
const app = express();
const port = process.env.PORT || 3000;

// Serve statically generated frontend
app.use("/", express.static("dist"));

app.listen(port, () => {
  console.log(`console_node app listening on port ${port}!`);
});
