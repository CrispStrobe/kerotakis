import assert from "node:assert/strict";
import test from "node:test";
import { createHash } from "node:crypto";
import { mkdtemp, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { candidateClosure, inventory, licenceAllowed, validateEvidence, validateFixtures } from "./evidence.mjs";

test("SPDX policy handles conjunctions without accepting an unknown branch", () => {
  assert.equal(licenceAllowed("MIT AND Zlib"), true);
  assert.equal(licenceAllowed("MIT AND GPL-3.0"), false);
  assert.equal(licenceAllowed("GPL-3.0 OR MIT"), true);
});
test("the exact five fixture hashes validate", async () => assert.equal((await validateFixtures()).fixtures.length, 5));
test("tampered fixture evidence fails closed", async () => {
  const dir = await mkdtemp(join(tmpdir(), "brd080-fixture-"));
  const manifest = JSON.parse(await readFile(new URL("fixtures/manifest.json", import.meta.url), "utf8"));
  await writeFile(join(dir, "manifest.json"), JSON.stringify(manifest));
  for (const fixture of manifest.fixtures) await writeFile(join(dir, fixture.path), "tampered");
  await assert.rejects(validateFixtures(dir), /integrity mismatch/);
});
test("lock inventory and candidate closures are exact and licence-complete", async () => {
  const { lock, rows } = await inventory();
  assert.equal(rows.length, 222);
  assert.equal(candidateClosure(lock, "3dmol").length, 6);
  assert.equal(candidateClosure(lock, "molstar").length, 216);
});
test("closure traversal rejects a missing transitive package", () => {
  const lock = { packages: { "node_modules/a": { dependencies: { b: "1" } } } };
  assert.throws(() => candidateClosure(lock, "a"), /unresolved production dependency b/);
});
test("closure traversal skips absent optional dependencies and resolves peers from the containing tree", () => {
  const lock = { packages: {
    "node_modules/a": { optionalDependencies: { native: "1" }, peerDependencies: { peer: "1" } },
    "node_modules/peer": {},
    "node_modules/a/node_modules/peer": {},
  } };
  assert.deepEqual(candidateClosure(lock, "a"), ["node_modules/a", "node_modules/peer"]);
});
const validLock = { packages: {
  "node_modules/3dmol": { version: "2.5.5" },
  "node_modules/molstar": { version: "5.11.0" },
} };
const validFixtures = Array.from({ length: 5 }, (_, id) => ({ id }));
const validPackages = [
  { path: "node_modules/3dmol", version: "2.5.5", license: "BSD-3-Clause", integrity: "sha512-a" },
  { path: "node_modules/molstar", version: "5.11.0", license: "MIT", integrity: "sha512-b" },
];
const expected = { lock: validLock, lockSha256: "a".repeat(64), fixtures: validFixtures, packages: validPackages };
const validReport = () => ({
  schema: "kerotakis.brd080-evidence.v1",
  environment: { node: process.version },
  lockSha256: "a".repeat(64),
  fixtures: validFixtures,
  productionPackageCount: 2,
  packages: validPackages,
  candidates: ["3dmol", "molstar"].map((name) => ({
    name,
    packages: validPackages.filter(({ path }) => path === `node_modules/${name}`),
    artifacts: [{ path: "candidate.js", bytes: 10, gzipBytes: 8, sha256: "b".repeat(64) }],
    totals: { bytes: 10, gzipBytes: 8 },
  })),
});
test("complete evidence reports validate", () => assert.equal(validateEvidence(validReport(), expected).candidates.length, 2));
test("malformed and incomplete evidence reports fail closed", () => {
  const incomplete = validReport();
  incomplete.candidates.pop();
  assert.throws(() => validateEvidence(incomplete, expected), /both ordered candidates/);
  const malformed = validReport();
  malformed.candidates[0].totals.bytes++;
  assert.throws(() => validateEvidence(malformed, expected), /incorrect 3dmol totals/);
  const wrongLock = validReport();
  wrongLock.lockSha256 = "c".repeat(64);
  assert.throws(() => validateEvidence(wrongLock, expected), /canonical lock/);
  const missingClosure = validReport();
  missingClosure.candidates[0].packages = [];
  assert.throws(() => validateEvidence(missingClosure, expected), /closure or artifacts/);
  const wrongFixture = validReport();
  wrongFixture.fixtures = [...validFixtures.slice(0, 4), { id: "wrong" }];
  assert.throws(() => validateEvidence(wrongFixture, expected), /canonical lock/);
});
test("committed evidence is canonically bound to the current lock and fixtures", async () => {
  const report = JSON.parse(await readFile(new URL("evidence.json", import.meta.url), "utf8"));
  const lockBytes = await readFile(new URL("package-lock.json", import.meta.url));
  const { lock, rows: packages } = await inventory();
  const fixtures = (await validateFixtures()).fixtures;
  const lockSha256 = createHash("sha256").update(lockBytes).digest("hex");
  assert.equal(validateEvidence(report, { lock, lockSha256, fixtures, packages }), report);
});
