const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
	mode: "production",
	entry: {
		index: "./js/index.js"
	},
	output: {
		path: dist,
		filename: "[name].js"
	},
	devServer: {
		contentBase: dist,
		host: process.env.HTTP_HOST || "localhost",
		port: parseInt(process.env.PORT || 8080),
	},
	experiments: {
		asyncWebAssembly: true,
	},
	plugins: [
		new CopyPlugin({
			patterns: [
				{from: path.resolve(__dirname, "static")},
				{from: path.resolve(__dirname, "gen")},
			]
		}),

		new WasmPackPlugin({
			crateDirectory: __dirname,
		}),
	]
};
