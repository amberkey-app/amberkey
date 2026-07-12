import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./test",
  timeout: 60000,
  use: {
    // The whole point: the tool must work with zero network access.
    offline: true,
  },
});
