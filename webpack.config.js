const path = require('path');

module.exports = {
    devtool: "source-map",
    entry: "./index.js",
    mode: "development",
    output: {
        path: path.join(__dirname, "dist"),
        filename: "bundle.js",
    },
    devServer: {
        watchOptions: {
            ignored: [
                path.join(__dirname, 'dist'),
                path.join(__dirname, 'node_modules'),
                path.join(__dirname, 'src'),
                path.join(__dirname, 'target'),
            ]
        }
    }
};
