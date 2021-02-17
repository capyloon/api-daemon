const path = require("path");

module.exports = {
  entry: ["./generated/contentmanager_service.js"],
  output: {
    filename: "service.js",
    library: "lib_contentmanager",
    libraryTarget: "umd",
    umdNamedDefine: true,
    path: path.resolve(__dirname, "dist")
  }
};
