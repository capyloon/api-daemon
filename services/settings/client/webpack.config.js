const path = require("path");

module.exports = {
  entry: ["./generated/settings_service.js"],
  output: {
    filename: "service.js",
    library: "lib_settings",
    libraryTarget: "umd",
    umdNamedDefine: true,
    path: path.resolve(__dirname, "dist")
  }
};
