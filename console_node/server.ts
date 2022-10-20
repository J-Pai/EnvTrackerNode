import express = require("express");
const app = express();
const port = process.env.PORT || 3000;

app.use("/", express.static("frontend/dist"))

app.listen(port, () => {
  console.log(`console_node app listening on port ${port}!`);
});
