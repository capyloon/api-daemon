const path = require("path");

module.exports = {
  entry: ["./generated/devicecapability_service.js"],
  output: {
    filename: "service.js",
    library: "lib_devicecapability",
    libraryTarget: "umd",
    umdNamedDefine: true,
    path: path.resolve(__dirname, "dist")
  }
};
