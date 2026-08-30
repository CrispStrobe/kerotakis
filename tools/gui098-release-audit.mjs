#!/usr/bin/env node
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

export const REQUIRED_PLATFORMS = ["web", "android", "ios", "macos", "windows"];
export const CPU_FRAME_P95_LIMIT_MS = 9;
export const GZIP_DELTA_LIMIT_BYTES = 64 * 1024;

const present = (value) => typeof value === "string" && value.trim().length > 0;
const finite = (value) => typeof value === "number" && Number.isFinite(value) && value >= 0;
const integer = (value) => Number.isSafeInteger(value) && value >= 0;
const date = (value) => present(value) && /^\d{4}-\d{2}-\d{2}$/.test(value) && !Number.isNaN(Date.parse(`${value}T00:00:00Z`));

function requireText(errors, value, path) {
  if (!present(value)) errors.push(`${path} must be explicit non-empty text`);
}

function validateArtifact(errors, artifact, path) {
  if (!artifact || typeof artifact !== "object") return errors.push(`${path} must be an artifact object`);
  requireText(errors, artifact.path, `${path}.path`);
  if (!/^[a-f0-9]{64}$/.test(artifact.sha256 ?? "")) errors.push(`${path}.sha256 must be 64 lowercase hex characters`);
}

function validateTiming(errors, timing, path, includeMax, expectedSamples) {
  if (!timing || typeof timing !== "object") return errors.push(`${path} is required`);
  if (!Number.isSafeInteger(timing.samples) || timing.samples < 1) errors.push(`${path}.samples must be a positive integer`);
  else if (timing.samples !== expectedSamples) errors.push(`${path}.samples must equal ${expectedSamples}`);
  for (const field of includeMax ? ["p50", "p95", "max"] : ["p50", "p95"]) {
    if (!finite(timing[field])) errors.push(`${path}.${field} must be a finite non-negative number`);
  }
  if (finite(timing.p50) && finite(timing.p95) && timing.p50 > timing.p95) errors.push(`${path} must satisfy p50 <= p95`);
  if (includeMax && finite(timing.p95) && finite(timing.max) && timing.p95 > timing.max) errors.push(`${path} must satisfy p95 <= max`);
}

function validateHost(errors, host, index) {
  const root = `hosts[${index}]`;
  if (!host || typeof host !== "object") return errors.push(`${root} must be an object`);
  const textFields = [
    [host.device?.manufacturer, "device.manufacturer"], [host.device?.model, "device.model"],
    [host.os?.name, "os.name"], [host.os?.version, "os.version"], [host.os?.build, "os.build"],
    [host.runtime?.kind, "runtime.kind"], [host.runtime?.name, "runtime.name"], [host.runtime?.version, "runtime.version"],
    [host.gpu?.vendor, "gpu.vendor"], [host.gpu?.model, "gpu.model"], [host.gpu?.driver, "gpu.driver"],
    [host.power?.source, "power.source"], [host.power?.mode, "power.mode"], [host.reviewer, "reviewer"],
  ];
  for (const [value, path] of textFields) requireText(errors, value, `${root}.${path}`);
  if (host.device?.physical !== true) errors.push(`${root}.device.physical must be true; inferred/emulated rows are incomplete`);
  for (const field of ["rawBytes", "gzipBytes", "baselineGzipBytes"]) {
    if (!integer(host.payload?.[field])) errors.push(`${root}.payload.${field} must be a non-negative integer`);
  }
  if (integer(host.payload?.rawBytes) && integer(host.payload?.gzipBytes) && host.payload.gzipBytes > host.payload.rawBytes) {
    errors.push(`${root}.payload.gzipBytes cannot exceed rawBytes`);
  }
  if (integer(host.payload?.gzipBytes) && integer(host.payload?.baselineGzipBytes)
      && host.payload.gzipBytes - host.payload.baselineGzipBytes > GZIP_DELTA_LIMIT_BYTES) {
    errors.push(`${root}.payload gzip delta exceeds ${GZIP_DELTA_LIMIT_BYTES} bytes`);
  }
  validateTiming(errors, host.coldStartupMs, `${root}.coldStartupMs`, false, 10);
  validateTiming(errors, host.cpuFrameMs, `${root}.cpuFrameMs`, true, 1800);
  if (finite(host.cpuFrameMs?.p95) && host.cpuFrameMs.p95 > CPU_FRAME_P95_LIMIT_MS) {
    errors.push(`${root}.cpuFrameMs.p95 exceeds the ${CPU_FRAME_P95_LIMIT_MS} ms BRD-072 governor`);
  }
  for (const field of ["frames", "missedFrames", "longFrames"]) {
    if (!integer(host.rafJank?.[field])) errors.push(`${root}.rafJank.${field} must be a non-negative integer`);
  }
  if (integer(host.rafJank?.frames)) {
    if (host.rafJank.frames !== 1800) errors.push(`${root}.rafJank.frames must equal 1800`);
    for (const field of ["missedFrames", "longFrames"]) {
      if (integer(host.rafJank?.[field]) && host.rafJank[field] > host.rafJank.frames) errors.push(`${root}.rafJank.${field} cannot exceed frames`);
    }
  }
  for (const scenario of ["webGpuAbsent", "compileFailure", "deviceLoss", "headless", "background", "reducedMotion"]) {
    if (host.fallback?.[scenario] !== true) errors.push(`${root}.fallback.${scenario} must explicitly pass`);
  }
  if (!Array.isArray(host.artifacts) || host.artifacts.length === 0) errors.push(`${root}.artifacts must contain raw evidence`);
  else host.artifacts.forEach((artifact, artifactIndex) => validateArtifact(errors, artifact, `${root}.artifacts[${artifactIndex}]`));
  if (!date(host.reviewedAt)) errors.push(`${root}.reviewedAt must be an ISO date`);
}

export function validateReleaseEvidence(value) {
  const errors = [];
  if (!value || typeof value !== "object") return ["evidence must be a JSON object"];
  if (value.schemaVersion !== 1) errors.push("schemaVersion must be 1");
  requireText(errors, value.candidate?.commit, "candidate.commit");
  requireText(errors, value.candidate?.buildArtifact, "candidate.buildArtifact");
  if (!present(value.candidate?.measuredAt) || Number.isNaN(Date.parse(value.candidate.measuredAt))) errors.push("candidate.measuredAt must be an ISO date-time");
  const hosts = Array.isArray(value.hosts) ? value.hosts : [];
  hosts.forEach((host, index) => {
    if (!REQUIRED_PLATFORMS.includes(host?.platform)) errors.push(`hosts[${index}].platform is unsupported`);
  });
  for (const platform of REQUIRED_PLATFORMS) {
    const count = hosts.filter((host) => host?.platform === platform).length;
    if (count !== 1) errors.push(`hosts must contain exactly one physical ${platform} row (found ${count})`);
  }
  hosts.forEach((host, index) => validateHost(errors, host, index));
  const review = value.similarityReview;
  requireText(errors, review?.method, "similarityReview.method");
  if (review?.result !== "independent") errors.push('similarityReview.result must be "independent"');
  requireText(errors, review?.reviewer, "similarityReview.reviewer");
  if (!date(review?.reviewedAt)) errors.push("similarityReview.reviewedAt must be an ISO date");
  if (!Array.isArray(review?.artifacts) || review.artifacts.length === 0) errors.push("similarityReview.artifacts must contain review evidence");
  else review.artifacts.forEach((artifact, index) => validateArtifact(errors, artifact, `similarityReview.artifacts[${index}]`));
  return errors;
}

export function validateShaderProvenance(source) {
  const errors = [];
  if (!/Nguyen,\s*Fedkiw\s*&\s*Jensen/i.test(source)) errors.push("shader provenance must name Nguyen, Fedkiw & Jensen");
  if (!/DOI\s+10\.1145\/566654\.566643/i.test(source)) errors.push("shader provenance DOI is missing");
  if (!/independent/i.test(source) || !/no source(?: code)? or equations were\s+copied/i.test(source)) {
    errors.push("shader must carry an explicit independent/no-copied-source notice");
  }
  const wgsl = source.match(/IGNITION_FLAME_WGSL[^`]*`([\s\S]*?)`/)?.[1] ?? "";
  if (!wgsl) errors.push("IGNITION_FLAME_WGSL source was not found");
  const forbidden = [/@import\b/i, /#include\b/i, /\brequire\s*\(/i, /\badapted from\b/i, /\bported from\b/i, /\bcopyright\b/i, /github\.com\//i, /dependency\s*:/i];
  for (const marker of forbidden) if (marker.test(wgsl)) errors.push(`WGSL contains forbidden copied/dependency marker ${marker}`);
  return errors;
}

function main() {
  if (process.argv.length !== 4) {
    console.error("usage: node tools/gui098-release-audit.mjs <evidence.json> <ignitionFlameShader.ts>");
    process.exit(2);
  }
  let evidence;
  try { evidence = JSON.parse(readFileSync(process.argv[2], "utf8")); }
  catch (error) { console.error(`invalid evidence JSON: ${error.message}`); process.exit(2); }
  const errors = [
    ...validateReleaseEvidence(evidence),
    ...validateShaderProvenance(readFileSync(process.argv[3], "utf8")),
  ];
  if (errors.length > 0) {
    console.error(`GUI-098 release evidence incomplete (${errors.length} issue${errors.length === 1 ? "" : "s"}):`);
    for (const error of errors) console.error(`- ${error}`);
    process.exit(1);
  }
  console.log("GUI-098 release evidence complete: five physical hosts, performance/fallback and provenance gates pass");
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
