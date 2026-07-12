#!/usr/bin/env node
// Build dist/recover.html: one self-contained file, WASM inlined as base64.
// Reproducible: fixed rustflags remap, no timestamps, deterministic inputs only.
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repo = join(here, "..", "..");
const wasmOut = process.env.WASM_OUT || join(here, "wasm");
const distDir = process.env.DIST_DIR || join(here, "dist");
const version = JSON.parse(
  execFileSync("cargo", ["metadata", "--format-version", "1", "--no-deps"], { cwd: repo, encoding: "utf8" })
).packages.find((p) => p.name === "amberkey-core").version;

execFileSync(
  "wasm-pack",
  // -j1: fat LTO merges modules in completion order, so parallel builds are
  // NOT byte-reproducible. Single-threaded is ~20s slower and deterministic.
  ["build", "crates/core", "--target", "no-modules", "--release", "--out-dir", wasmOut, "--", "--features", "wasm", "-j1"],
  {
    cwd: repo,
    stdio: "inherit",
    env: {
      ...process.env,
      // Fat LTO's module merge is not byte-deterministic (parallel completion
      // order leaks into the binary). LTO off keeps the trust anchor
      // byte-reproducible; size cost is a few percent.
      CARGO_PROFILE_RELEASE_LTO: "off",
      // Strip machine-specific paths for cross-machine reproducibility.
      // CARGO_HOME matters: rust docker images use /usr/local/cargo, dev boxes ~/.cargo.
      RUSTFLAGS: `${process.env.RUSTFLAGS || ""} --remap-path-prefix=${repo}=/build --remap-path-prefix=${process.env.CARGO_HOME || `${process.env.HOME}/.cargo`}=/cargo --remap-path-prefix=${process.env.HOME}=/home`.trim(),
    },
  }
);

const glue = readFileSync(join(wasmOut, "amberkey_core.js"), "utf8");
const wasm = readFileSync(join(wasmOut, "amberkey_core_bg.wasm"));
const template = readFileSync(join(here, "template.html"), "utf8");

const html = template
  .replace("__VERSION__", version)
  .replace("__GLUE_JS__", () => glue)
  .replace("__WASM_B64__", () => wasm.toString("base64"));

mkdirSync(distDir, { recursive: true });
const out = join(distDir, "recover.html");
writeFileSync(out, html);
const hash = createHash("sha256").update(html).digest("hex");
writeFileSync(join(distDir, "recover.html.sha256"), `${hash}  recover.html\n`);
console.log(`${out}\nsha256: ${hash}  (${(html.length / 1024).toFixed(0)} KiB)`);
