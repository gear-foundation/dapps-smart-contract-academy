import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import path from "path";
import nodePolyfills from "vite-plugin-node-stdlib-browser";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

// https://vitejs.dev/config/
export default defineConfig({
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "src"),
      // "@/app": path.resolve(__dirname, "./src/app"),
      // "@/assets": path.resolve(__dirname, "./src/assets"),
      // "@/components": path.resolve(__dirname, "./src/components"),
    },
  },
  server: {
    port: 3000,
  },
  plugins: [
    // wasm(), topLevelAwait(),
    react(),
    nodePolyfills(),
  ],
  assetsInclude: ["**/*.wasm?inline"],
});
