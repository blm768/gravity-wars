const path = require('path');

module.exports = {
    devtool: "source-map",
    entry: "./index.js",
    mode: "development",
    output: {
        path: path.join(__dirname, "dist"),
        filename: "bundle.js",
    }
};
