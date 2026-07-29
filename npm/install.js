#!/usr/bin/env node
// Downloads the correct prebuilt ailint binary from GitHub Releases into
// ./bin/ailint (or ailint.exe on Windows) at `npm install` time.
//
// TODO:
//   - map process.platform + process.arch to release asset names
//   - stream download with retries + progress
//   - verify checksum (release should ship .sha256 sidecars)
//   - support offline installs via AILINT_BINARY env override
//   - make executable (chmod +x) on POSIX

"use strict";

const fs = require("fs");
const path = require("path");

const VERSION = require("./package.json").version;
const BIN_DIR = path.join(__dirname, "bin");
const BIN_NAME = process.platform === "win32" ? "ailint.exe" : "ailint";

function target() {
  // TODO: complete matrix.
  const p = process.platform;
  const a = process.arch;
  if (p === "linux" && a === "x64") return "x86_64-unknown-linux-musl";
  if (p === "linux" && a === "arm64") return "aarch64-unknown-linux-musl";
  if (p === "darwin" && a === "x64") return "x86_64-apple-darwin";
  if (p === "darwin" && a === "arm64") return "aarch64-apple-darwin";
  if (p === "win32" && a === "x64") return "x86_64-pc-windows-msvc";
  throw new Error(`unsupported platform: ${p}/${a}`);
}

async function main() {
  fs.mkdirSync(BIN_DIR, { recursive: true });
  const t = target();
  console.log(`ailint: TODO download v${VERSION} for ${t}`);
  // Placeholder so `npm install` succeeds during scaffolding.
  fs.writeFileSync(
    path.join(BIN_DIR, BIN_NAME),
    "#!/bin/sh\necho 'ailint: binary not installed; TODO wire release download'\nexit 1\n",
    { mode: 0o755 },
  );
}

main().catch((err) => {
  console.error("ailint install failed:", err);
  process.exit(1);
});
