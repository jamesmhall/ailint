#!/usr/bin/env node
// Downloads the correct prebuilt ailint binary from GitHub Releases into
// ./bin/ailint (or ailint.exe on Windows) at `npm install` time.
//
// Set AILINT_BINARY=/path/to/ailint for offline installs.

"use strict";

const fs = require("fs");
const os = require("os");
const path = require("path");
const https = require("https");
const crypto = require("crypto");
const { execFileSync } = require("child_process");

const VERSION = require("./package.json").version;
const REPO = "jamesmhall/ailint";
const BIN_DIR = path.join(__dirname, "bin");
const BIN_NAME = process.platform === "win32" ? "ailint.exe" : "ailint";
const BIN_PATH = path.join(BIN_DIR, BIN_NAME);

function target() {
  const p = process.platform;
  const a = process.arch;
  if (p === "linux" && a === "x64") return "x86_64-unknown-linux-musl";
  if (p === "linux" && a === "arm64") return "aarch64-unknown-linux-musl";
  if (p === "darwin" && a === "x64") return "x86_64-apple-darwin";
  if (p === "darwin" && a === "arm64") return "aarch64-apple-darwin";
  if (p === "win32" && a === "x64") return "x86_64-pc-windows-msvc";
  throw new Error(
    `ailint: unsupported platform ${p}/${a}. Set AILINT_BINARY to a local ailint binary.`,
  );
}

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const req = https.get(url, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        res.resume();
        download(res.headers.location, dest).then(resolve, reject);
        return;
      }
      if (res.statusCode !== 200) {
        res.resume();
        reject(new Error(`HTTP ${res.statusCode} for ${url}`));
        return;
      }
      const total = parseInt(res.headers["content-length"] || "0", 10);
      let got = 0;
      const out = fs.createWriteStream(dest);
      res.on("data", (chunk) => {
        got += chunk.length;
        if (total > 0) {
          process.stderr.write(
            `\railint: downloading ${got}/${total} bytes`,
          );
        }
      });
      res.on("end", () => process.stderr.write("\n"));
      res.pipe(out);
      out.on("finish", () => out.close(resolve));
      out.on("error", reject);
    });
    req.on("error", reject);
  });
}

async function downloadWithRetries(url, dest) {
  const delays = [1000, 2000, 4000];
  let lastErr;
  for (let i = 0; i < 3; i++) {
    try {
      await download(url, dest);
      return;
    } catch (err) {
      lastErr = err;
      if (i < 2) {
        process.stderr.write(
          `ailint: download failed (${err.message}); retrying in ${delays[i]}ms\n`,
        );
        await sleep(delays[i]);
      }
    }
  }
  throw new Error(`download failed after 3 attempts: ${lastErr.message}`);
}

function sha256(file) {
  const h = crypto.createHash("sha256");
  h.update(fs.readFileSync(file));
  return h.digest("hex");
}

function verifyChecksum(archive, sumFile) {
  const expected = fs
    .readFileSync(sumFile, "utf8")
    .trim()
    .split(/\s+/)[0]
    .toLowerCase();
  const actual = sha256(archive).toLowerCase();
  if (expected !== actual) {
    throw new Error(
      `checksum mismatch: expected ${expected}, got ${actual}`,
    );
  }
}

function extract(archive, ext, destDir) {
  if (ext === "zip") {
    execFileSync(
      "powershell",
      [
        "-NoProfile",
        "-Command",
        `Expand-Archive -Force -Path '${archive}' -DestinationPath '${destDir}'`,
      ],
      { stdio: "inherit" },
    );
  } else {
    execFileSync("tar", ["-xzf", archive, "-C", destDir], { stdio: "inherit" });
  }
}

async function main() {
  fs.mkdirSync(BIN_DIR, { recursive: true });

  if (process.env.AILINT_BINARY) {
    fs.copyFileSync(process.env.AILINT_BINARY, BIN_PATH);
    if (process.platform !== "win32") fs.chmodSync(BIN_PATH, 0o755);
    process.stderr.write(`ailint: installed from AILINT_BINARY\n`);
    return;
  }

  const t = target();
  const ext = process.platform === "win32" ? "zip" : "tar.gz";
  const asset = `ailint-v${VERSION}-${t}.${ext}`;
  const baseUrl = `https://github.com/${REPO}/releases/download/v${VERSION}`;
  const url = `${baseUrl}/${asset}`;
  const sumUrl = `${url}.sha256`;

  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "ailint-"));
  const archive = path.join(tmp, asset);
  const sumFile = `${archive}.sha256`;

  try {
    process.stderr.write(`ailint: fetching ${url}\n`);
    await downloadWithRetries(sumUrl, sumFile);
    await downloadWithRetries(url, archive);
    verifyChecksum(archive, sumFile);
    extract(archive, ext, BIN_DIR);
    if (!fs.existsSync(BIN_PATH)) {
      throw new Error(`extracted archive did not contain ${BIN_NAME}`);
    }
    if (process.platform !== "win32") fs.chmodSync(BIN_PATH, 0o755);
    process.stderr.write(`ailint: installed v${VERSION} for ${t}\n`);
  } catch (err) {
    try {
      fs.rmSync(BIN_PATH, { force: true });
    } catch (_) {}
    throw err;
  } finally {
    fs.rmSync(tmp, { recursive: true, force: true });
  }
}

main().catch((err) => {
  process.stderr.write(`ailint install failed: ${err.message}\n`);
  process.exit(1);
});
