import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  server: {
    port: 5173,
    strictPort: true, // fail loudly if port is taken
  },
  build: {
    outDir: "dist",
  },
});
