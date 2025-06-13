import { defineConfig } from "vitest/config";
import { lunas } from "vite-plugin-lunas";
export default defineConfig({
  plugins: [lunas()],
  test: {
    environment: "jsdom",
  },
});
