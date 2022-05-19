const path = require("path");

module.exports = {
  entry: ["./generated/dweb_service.js"],
  output: {
    filename: "service.js",
    library: "lib_dweb",
    libraryTarget: "umd",
    umdNamedDefine: true,
    path: path.resolve(__dirname, "dist")
  }
};
