const { main } = require("./cli.js");

main().catch((e) => {
  console.log(e);
  process.exit(1);
});
