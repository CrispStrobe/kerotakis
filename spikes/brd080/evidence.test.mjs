import assert from "node:assert/strict";
import test from "node:test";
import { mkdtemp, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { candidateClosure, inventory, licenceAllowed, validateFixtures } from "./evidence.mjs";

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
