#!/usr/bin/env node
// The npm face of the licence bar (ROADMAP-Webapp.md allowlist; the cargo
// side is cargo-deny). Every installed package must declare a licence the
// direct-inclusion allowlist accepts. Dual-licensed expressions pass if any
// branch is allowlisted — the permissive branch is the one we take.
import { readdirSync, readFileSync, existsSync } from "node:fs";
import { join } from "node:path";

const ALLOWED = new Set([
  "MIT",
  "Apache-2.0",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "ISC",
  "0BSD",
  "Zlib",
  "Unlicense",
  "CC0-1.0",
  "BlueOak-1.0.0",
  "Python-2.0",
]);

function licenceOk(expr) {
  if (typeof expr !== "string") return false;
  const cleaned = expr.replace(/[()]/g, "");
  if (cleaned.includes(" OR "))
    return cleaned.split(" OR ").some((part) => licenceOk(part.trim()));
  if (cleaned.includes(" AND "))
    return cleaned.split(" AND ").every((part) => licenceOk(part.trim()));
  return ALLOWED.has(cleaned.trim());
}

function* packages(dir) {
  if (!existsSync(dir)) return;
  for (const entry of readdirSync(dir)) {
    if (entry.startsWith(".")) continue;
    if (entry.startsWith("@")) {
      yield* packages(join(dir, entry));
      continue;
    }
    const manifest = join(dir, entry, "package.json");
    if (existsSync(manifest)) {
      yield JSON.parse(readFileSync(manifest, "utf8"));
      yield* packages(join(dir, entry, "node_modules"));
    }
  }
}

const root = new URL("../node_modules", import.meta.url).pathname;
const bad = [];
let count = 0;
for (const pkg of packages(root)) {
  count += 1;
  if (!licenceOk(pkg.license)) {
    bad.push(`${pkg.name}@${pkg.version}: ${pkg.license ?? "UNDECLARED"}`);
  }
}

if (count === 0) {
  console.error("no node_modules found — run npm install first");
  process.exit(2);
}
if (bad.length > 0) {
  console.error(`licence bar failed for ${bad.length} package(s):`);
  for (const line of bad) console.error(`  ${line}`);
  process.exit(1);
}
console.log(`licence bar clean: ${count} packages, all allowlisted`);
