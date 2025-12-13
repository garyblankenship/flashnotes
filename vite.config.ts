import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST || "127.0.0.1";

export default defineConfig(async () => ({
  plugins: [sveltekit()],

  // Required for CodeMirror 6 with Vite
  optimizeDeps: {
    exclude: [
      '@codemirror/state',
      '@codemirror/view',
      '@codemirror/commands',
      '@codemirror/language',
      '@codemirror/autocomplete',
      '@codemirror/search',
    ],
  },

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host,
    hmr: {
      protocol: "ws",
      host,
      port: 1421,
    },
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
