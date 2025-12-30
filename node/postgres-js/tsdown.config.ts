import { defineConfig } from "tsdown";

export default defineConfig({
  entry: ["src/index.ts"],
  format: ["cjs", "esm"],
  dts: true,
  publint: {
    strict: true,
  },
  attw: {
    profile: 'strict',
    level: 'error'
  },
});
