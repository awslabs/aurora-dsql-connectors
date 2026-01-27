import { defineConfig } from "tsdown";
import { readFileSync } from "fs";

const packageJson = JSON.parse(readFileSync("./package.json", "utf-8"));

export default defineConfig({
  entry: ["src/index.ts"],
  format: ["cjs", "esm"],
  dts: true,
  define: {
    __VERSION__: JSON.stringify(packageJson.version),
  },
  publint: {
    strict: true,
  },
  attw: {
    profile: 'strict',
    level: 'error'
  },
});
