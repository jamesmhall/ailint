#!/usr/bin/env node
// Thin shim that execs the downloaded ailint binary with the given args.

"use strict";

const { spawnSync } = require("child_process");
const path = require("path");

const bin = path.join(
  __dirname,
  "bin",
  process.platform === "win32" ? "ailint.exe" : "ailint",
);

const result = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
process.exit(result.status ?? 1);
