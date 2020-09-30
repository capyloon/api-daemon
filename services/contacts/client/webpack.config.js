const path = require("path");

module.exports = {
  entry: ["./generated/contacts_service.js"],
  output: {
    filename: "service.js",
    library: "lib_contacts",
    libraryTarget: "umd",
    umdNamedDefine: true,
    path: path.resolve(__dirname, "dist")
  }
};
