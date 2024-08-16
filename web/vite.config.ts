import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";
import path from "node:path";

// https://vitejs.dev/config/
export default defineConfig({
	resolve: {
		alias: {
			"@": path.resolve(__dirname, "./src"),
			"@assets": path.resolve(__dirname, "./src/assets"),
			"@lib": path.resolve(__dirname, "./src/lib"),
		},
	},
	plugins: [svelte()],
});
