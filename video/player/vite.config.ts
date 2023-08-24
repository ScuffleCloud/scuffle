import { defineConfig } from "vite";
import path from "path";
import dts from "vite-plugin-dts";

export default defineConfig({
	plugins: [
		dts({
			outDir: ["dist"],
			insertTypesEntry: true,
		}),
	],
	optimizeDeps: {
		exclude: ["player-wasm"],
	},
	build: {
		minify: false,
		target: "esnext",
		outDir: "dist",
		lib: {
			entry: path.resolve(__dirname, "js/main.ts"),
			formats: ["es"],
			name: "Player",
			fileName: "player",
		},
		assetsInlineLimit: 0,
		rollupOptions: {
			external: [/pkg/],
		},
	},
});
