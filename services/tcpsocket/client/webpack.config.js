const path = require("path");

module.exports = {
  entry: ["./generated/tcpsocket_service.js"],
  output: {
    filename: "service.js",
    library: "lib_tcpsocket",
    libraryTarget: "umd",
    umdNamedDefine: true,
    path: path.resolve(__dirname, "dist")
  }
};
