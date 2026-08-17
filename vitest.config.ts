import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte()],
  resolve: { alias: { "$lib": new URL("./src/lib", import.meta.url).pathname } },
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.ts"],
    clearMocks: true,
  },
});
