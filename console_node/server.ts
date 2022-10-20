import express = require("express");
const app = express();
const port = process.env.PORT || 3000;

app.get(["/", "/:name"], (req, res) => {
  const greeting = "<h1>Hello From Console Node!</h1>";
  const name = req.params["name"];
  if (name) {
    res.send(`${greeting} </br>and hello to ${name}`);
  } else {
    res.send(greeting);
  }
});

app.listen(port, () => {
  console.log(`console_node app listening on port ${port}!`);
});
