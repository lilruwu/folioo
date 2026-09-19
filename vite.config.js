import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri expects a fixed dev port and serves the production build from ./dist.
export default defineConfig({
  plugins: [react()],
  // Prevent Vite from obscuring Rust errors.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // Don't watch the Rust source tree, nor the Flatpak build artifacts:
      // flatpak-build/ mirrors parts of a root filesystem and contains symlink
      // loops (var/run/udev/watch/...) that crash the watcher with ELOOP.
      ignored: [
        "**/src-tauri/**",
        "**/flatpak-build/**",
        "**/flatpak-repo/**",
        "**/.flatpak-builder/**",
      ],
    },
  },
  build: {
    target: "es2021",
    outDir: "dist",
    emptyOutDir: true,
  },
});
