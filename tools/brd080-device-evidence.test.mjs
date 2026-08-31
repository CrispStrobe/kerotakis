import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdtemp, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { validateDeviceEvidence, validateDeviceEvidenceFile } from "./brd080-device-evidence.mjs";

const sha = (value) => createHash("sha256").update(value).digest("hex");
const artifact = { path: "raw.txt", sha256: sha("raw") };
const row = (platform) => ({
  platform, physical: true,
  device: { manufacturer: "Lab", model: `${platform}-phone`, hardwareIdHash: sha(`${platform}-id`) },
  os: { name: platform === "ios" ? "iOS" : "Android", version: "1", build: "build" },
  runtime: { name: platform === "ios" ? "Mobile Safari" : "Chrome", version: "1", engine: platform === "ios" ? "WebKit" : "Blink" },
  gpu: { vendor: "vendor", renderer: "renderer", webglVersion: "WebGL 2", maxTextureSize: 4096, maxRenderbufferSize: 4096 },
  viewport: { cssWidth: 390, cssHeight: 844, dpr: 3 },
  network: { origin: "https://example.vercel.app", externalRequests: 0 },
  workload: { cycles: 3, paths: 5, successfulPaths: 5, interactions: ["load", "select", "labels", "resize", "reduced-motion", "dispose"], maxCanvasPixels: 1000, contextLosses: 0, errors: [] },
  memory: { method: "OS process sampler", baselineBytes: 100, peakBytes: 180, settledBytes: 110, deltaPeakBytes: 80, deltaSettledBytes: 10, samplesArtifact: artifact },
  artifacts: [artifact], reviewer: "Release Lab", measuredAt: "2026-08-31T12:00:00Z",
});
const evidence = () => ({
  schema: "kerotakis.brd080-device-evidence.v1",
  candidate: { name: "3dmol", version: "2.5.5", sourceCommit: "c26e390544b6388f86e50387cd4565759b4da0df", routeCommit: "a".repeat(40), routeArtifact: artifact },
  measuredAt: "2026-08-31T12:00:00Z", reviewer: "Release Lab", rows: [row("android"), row("ios")],
});

test("accepts a complete two-device envelope", () => assert.deepEqual(validateDeviceEvidence(evidence()), []));

test("rejects nonphysical, incomplete and unsafe measurements", () => {
  const value = evidence();
  value.rows[0].physical = false;
  value.rows[0].runtime.engine = "headless SwiftShader emulator";
  value.rows[0].network.externalRequests = 1;
  value.rows[0].workload.interactions.pop();
  value.rows[0].workload.maxCanvasPixels = 99_000_000;
  value.rows[0].workload.contextLosses = 1;
  value.rows[0].memory.deltaPeakBytes = 3;
  const errors = validateDeviceEvidence(value).join("\n");
  for (const marker of ["physical must be true", "physical-device browser", "externalRequests", "dispose", "maxCanvasPixels", "contextLosses", "deltaPeakBytes"]) assert.match(errors, new RegExp(marker));
});

test("requires exactly one Android and iOS row and the exact candidate", () => {
  const value = evidence();
  value.candidate.version = "latest";
  value.rows = [value.rows[0], structuredClone(value.rows[0])];
  const errors = validateDeviceEvidence(value).join("\n");
  assert.match(errors, /candidate must be 3dmol 2\.5\.5/);
  assert.match(errors, /exactly one physical android row \(found 2\)/);
  assert.match(errors, /exactly one physical ios row \(found 0\)/);
});

test("malformed row collections fail closed without crashing file validation", async () => {
  const directory = await mkdtemp(join(tmpdir(), "brd080-device-malformed-"));
  await writeFile(join(directory, "raw.txt"), "raw");
  const value = evidence(); value.rows = {};
  const path = join(directory, "evidence.json");
  await writeFile(path, JSON.stringify(value));
  assert.match((await validateDeviceEvidenceFile(path)).join("\n"), /physical android row/);
});

test("checks artifact confinement and content hashes", async () => {
  const directory = await mkdtemp(join(tmpdir(), "brd080-device-"));
  await writeFile(join(directory, "raw.txt"), "raw");
  const path = join(directory, "evidence.json");
  await writeFile(path, JSON.stringify(evidence()));
  assert.deepEqual(await validateDeviceEvidenceFile(path), []);
  const tampered = evidence(); tampered.rows[0].artifacts[0] = { path: "../outside", sha256: sha("raw") };
  await writeFile(path, JSON.stringify(tampered));
  assert.match((await validateDeviceEvidenceFile(path)).join("\n"), /escapes the evidence directory/);
  const mismatch = evidence(); mismatch.candidate.routeArtifact.sha256 = "0".repeat(64);
  await writeFile(path, JSON.stringify(mismatch));
  assert.match((await validateDeviceEvidenceFile(path)).join("\n"), /sha256 mismatch/);
});
