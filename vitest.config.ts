import { defineConfig } from "vitest/config";
import path from "path";

export default defineConfig({
  test: {
    environment: "jsdom",
    globals: true,
    include: ["src/**/__tests__/**/*.test.ts"],
    alias: {
      "$lib": path.resolve("./src/lib"),
      "@tauri-apps/api/event": path.resolve("./src/lib/__mocks__/tauri.ts"),
      "@tauri-apps/api/core": path.resolve("./src/lib/__mocks__/tauri.ts"),
    },
  },
});
