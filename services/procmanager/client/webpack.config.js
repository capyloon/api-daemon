const path = require("path");

module.exports = {
  entry: ["./generated/procmanager_service.js"],
  output: {
    filename: "service.js",
    library: "lib_procmanager",
    libraryTarget: "umd",
    umdNamedDefine: true,
    path: path.resolve(__dirname, "dist")
  }
};
