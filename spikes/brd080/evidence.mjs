#!/usr/bin/env node
import { createHash } from "node:crypto";
import { mkdtemp, readFile, readdir, rm, stat } from "node:fs/promises";
import { gzipSync } from "node:zlib";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { build } from "vite";

const root = new URL(".", import.meta.url).pathname;
const sha = (data) => createHash("sha256").update(data).digest("hex");
const allowed = new Set(["MIT", "ISC", "Apache-2.0", "BSD-3-Clause", "0BSD", "Python-2.0", "Zlib"]);
export function licenceAllowed(value) {
  if (typeof value !== "string") return false;
  const clean = value.replace(/[()]/g, "");
  if (clean.includes(" OR ")) return clean.split(" OR ").some(licenceAllowed);
  if (clean.includes(" AND ")) return clean.split(" AND ").every(licenceAllowed);
  return allowed.has(clean.trim());
}
export async function validateFixtures(directory = join(root, "fixtures")) {
  const manifest = JSON.parse(await readFile(join(directory, "manifest.json"), "utf8"));
  const expected = ["molecule", "crystal", "protein", "orbital", "trajectory"];
  if (manifest.schemaVersion !== 1 || manifest.fixtures?.map((x) => x.kind).join() !== expected.join()) throw new Error("fixture manifest must contain the five ordered BRD-080 kinds");
  for (const item of manifest.fixtures) {
    const data = await readFile(join(directory, item.path));
    if (data.length !== item.bytes || sha(data) !== item.sha256) throw new Error(`fixture integrity mismatch: ${item.path}`);
  }
  return manifest;
}
function resolveLocked(packages, parent, name) {
  let cursor = parent;
  while (cursor) {
    const nested = `${cursor}/node_modules/${name}`;
    if (packages[nested]) return nested;
    cursor = cursor.replace(/(?:^|\/)node_modules\/(?:@[^/]+\/)?[^/]+$/, "");
  }
  const top = `node_modules/${name}`;
  return packages[top] ? top : undefined;
}
export function candidateClosure(lock, rootName) {
  const start = `node_modules/${rootName}`;
  if (!lock.packages?.[start]) throw new Error(`missing candidate package: ${rootName}`);
  const seen = new Set(), queue = [start];
  while (queue.length) {
    const path = queue.shift();
    if (seen.has(path)) continue;
    seen.add(path);
    const entry = lock.packages[path];
    const names = new Set([...Object.keys(entry.dependencies ?? {}), ...Object.keys(entry.optionalDependencies ?? {}), ...Object.keys(entry.peerDependencies ?? {})]);
    for (const name of [...names].sort()) {
      if (entry.peerDependenciesMeta?.[name]?.optional && !resolveLocked(lock.packages, path, name)) continue;
      const resolved = resolveLocked(lock.packages, path, name);
      if (!resolved) throw new Error(`${path} has unresolved production dependency ${name}`);
      queue.push(resolved);
    }
  }
  return [...seen].sort();
}
export async function inventory(lockPath = join(root, "package-lock.json")) {
  const lock = JSON.parse(await readFile(lockPath, "utf8"));
  if (lock.lockfileVersion !== 3) throw new Error("BRD-080 requires npm lockfileVersion 3");
  for (const [name, version] of [["3dmol", "2.5.5"], ["molstar", "5.11.0"]]) {
    if (lock.packages?.[""]?.dependencies?.[name] !== version || lock.packages?.[`node_modules/${name}`]?.version !== version) throw new Error(`${name} must be exactly pinned to ${version}`);
  }
  const rows = [];
  for (const [path, entry] of Object.entries(lock.packages).sort(([a], [b]) => a.localeCompare(b))) {
    if (!path || entry.dev) continue;
    if (!entry.version || !entry.integrity) throw new Error(`incomplete locked production package: ${path}`);
    const manifest = JSON.parse(await readFile(join(root, path, "package.json"), "utf8"));
    if (manifest.version !== entry.version || !licenceAllowed(manifest.license)) throw new Error(`installed version or licence is disallowed: ${path}`);
    rows.push({ path, version: entry.version, license: manifest.license, integrity: entry.integrity });
  }
  return { lock, rows };
}
async function files(directory) {
  const result = [];
  for (const name of (await readdir(directory)).sort()) {
    const path = join(directory, name), info = await stat(path);
    if (info.isDirectory()) result.push(...await files(path));
    else { const data = await readFile(path); result.push({ path: path.slice(directory.length + 1), bytes: data.length, gzipBytes: gzipSync(data, { level: 9 }).length, sha256: sha(data) }); }
  }
  return result;
}
export async function collect() {
  const fixtures = await validateFixtures();
  const { lock, rows: packages } = await inventory();
  const lockBytes = await readFile(join(root, "package-lock.json"));
  const temporary = await mkdtemp(join(tmpdir(), "kerotakis-brd080-"));
  try {
    const candidates = [];
    for (const name of ["3dmol", "molstar"]) {
      const outDir = join(temporary, name);
      await build({ root, configFile: false, logLevel: "silent", build: { outDir, emptyOutDir: true, sourcemap: false, rollupOptions: { input: resolve(root, `src/measure-${name}.ts`), output: { entryFileNames: "candidate.js", chunkFileNames: "chunk-[hash].js", assetFileNames: "asset-[hash][extname]" } } } });
      const artifacts = await files(outDir);
      const closure = new Set(candidateClosure(lock, name));
      candidates.push({ name, packages: packages.filter((row) => closure.has(row.path)), artifacts, totals: artifacts.reduce((a, x) => ({ bytes: a.bytes + x.bytes, gzipBytes: a.gzipBytes + x.gzipBytes }), { bytes: 0, gzipBytes: 0 }) });
    }
    return {
      schema: "kerotakis.brd080-evidence.v1",
      environment: { node: process.version, molstarRequiredNode: ">=22.0.0" },
      lockSha256: sha(lockBytes),
      fixtures: fixtures.fixtures,
      productionPackageCount: packages.length,
      packages,
      candidates,
    };
  } finally { await rm(temporary, { recursive: true, force: true }); }
}
if (import.meta.url === new URL(process.argv[1], "file:").href) console.log(JSON.stringify(await collect(), null, 2));
