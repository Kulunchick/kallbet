import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// `npm run dev` serves the SPA with hot reload and proxies the API to the
// Rust binary (cargo run, default 127.0.0.1:8095). `npm run build` writes
// dist/, which the binary serves itself.
export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    proxy: {
      "/api": "http://127.0.0.1:8095",
    },
  },
});
