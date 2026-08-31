import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { validateReleaseEvidence, validateShaderProvenance } from "../gui098-release-audit.mjs";

const artifact = { path: "artifacts/run.json", sha256: "a".repeat(64) };
const host = (platform) => ({
  platform,
  device: { manufacturer: "Lab", model: `${platform}-device`, physical: true },
  os: { name: platform, version: "1", build: "1A" },
  runtime: { kind: platform === "web" ? "browser" : "WebView", name: "runtime", version: "1" },
  gpu: { vendor: "vendor", model: "gpu", driver: "driver-1" },
  power: { source: "AC", mode: "balanced" },
  payload: { rawBytes: 1000, gzipBytes: 500, baselineGzipBytes: 450 },
  coldStartupMs: { samples: 10, p50: 100, p95: 140 },
  cpuFrameMs: { samples: 1800, p50: 3, p95: 8.5, max: 12 },
  rafJank: { frames: 1800, missedFrames: 2, longFrames: 1 },
  fallback: { webGpuAbsent: true, compileFailure: true, deviceLoss: true, headless: true, background: true, reducedMotion: true },
  artifacts: [artifact], reviewer: "Reviewer", reviewedAt: "2026-08-30",
});
const complete = () => ({
  schemaVersion: 1,
  candidate: { commit: "abcdef123", buildArtifact: "release.zip", measuredAt: "2026-08-30T20:00:00Z" },
  hosts: ["web", "android", "ios", "macos", "windows"].map(host),
  similarityReview: { method: "side-by-side source review", result: "independent", reviewer: "Reviewer", reviewedAt: "2026-08-30", artifacts: [artifact] },
});

test("complete physical five-host evidence passes", () => {
  assert.deepEqual(validateReleaseEvidence(complete()), []);
});

test("missing, inferred, and over-budget rows remain incomplete", () => {
  const evidence = complete();
  evidence.hosts.pop();
  evidence.hosts[0].device.physical = false;
  evidence.hosts[1].cpuFrameMs.p95 = 9.1;
  evidence.hosts[2].fallback.deviceLoss = null;
  const errors = validateReleaseEvidence(evidence).join("\n");
  assert.match(errors, /exactly one physical windows row/);
  assert.match(errors, /inferred\/emulated rows are incomplete/);
  assert.match(errors, /exceeds the 9 ms/);
  assert.match(errors, /deviceLoss must explicitly pass/);
});

test("the committed null template intentionally fails", () => {
  const template = JSON.parse(readFileSync(new URL("../../docs/gui098-gpu-release-evidence.template.json", import.meta.url)));
  assert.ok(validateReleaseEvidence(template).length > 20);
});

test("current shader has DOI and independent provenance", () => {
  const source = readFileSync(new URL("../../web/app/src/lib/ignitionFlameShader.ts", import.meta.url), "utf8");
  assert.deepEqual(validateShaderProvenance(source), []);
});

test("copied or dependency-marked WGSL fails provenance review", () => {
  const bad = `Nguyen, Fedkiw & Jensen DOI 10.1145/566654.566643 independent no source or equations were copied\nexport const IGNITION_FLAME_WGSL = \`// Adapted from dependency: github.com/example/code\``;
  const errors = validateShaderProvenance(bad).join("\n");
  assert.match(errors, /adapted from/i);
  assert.match(errors, /github/i);
  assert.match(errors, /dependency/i);
});
