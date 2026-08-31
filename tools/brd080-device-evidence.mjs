#!/usr/bin/env node
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { dirname, isAbsolute, relative, resolve, sep } from "node:path";
import { pathToFileURL } from "node:url";

export const SCHEMA = "kerotakis.brd080-device-evidence.v1";
export const PLATFORMS = ["android", "ios"];
export const CANDIDATE = { name: "3dmol", version: "2.5.5" };
export const SOURCE_COMMIT = "c26e390544b6388f86e50387cd4565759b4da0df";
export const MAX_CANVAS_PIXELS = 1280 * 960 * 4;
export const REQUIRED_INTERACTIONS = ["load", "select", "labels", "resize", "reduced-motion", "dispose"];

const text = (value) => typeof value === "string" && value.trim().length > 0;
const integer = (value) => Number.isSafeInteger(value) && value >= 0;
const finite = (value) => typeof value === "number" && Number.isFinite(value) && value >= 0;
const digest = (value) => /^[a-f0-9]{64}$/.test(value ?? "");
const instant = (value) => text(value) && !Number.isNaN(Date.parse(value));

function requireText(errors, value, path) {
  if (!text(value)) errors.push(`${path} must be explicit non-empty text`);
}

function artifactErrors(errors, artifact, path) {
  if (!artifact || typeof artifact !== "object" || Array.isArray(artifact)) {
    errors.push(`${path} must be an artifact object`);
    return;
  }
  requireText(errors, artifact.path, `${path}.path`);
  if (!digest(artifact.sha256)) errors.push(`${path}.sha256 must be 64 lowercase hex characters`);
}

function validateRow(errors, row, index) {
  const root = `rows[${index}]`;
  if (!row || typeof row !== "object" || Array.isArray(row)) return errors.push(`${root} must be an object`);
  if (!PLATFORMS.includes(row.platform)) errors.push(`${root}.platform is unsupported`);
  if (row.physical !== true) errors.push(`${root}.physical must be true; simulators, emulators and viewport emulation are incomplete evidence`);
  for (const [value, path] of [
    [row.device?.manufacturer, "device.manufacturer"], [row.device?.model, "device.model"],
    [row.device?.hardwareIdHash, "device.hardwareIdHash"], [row.os?.name, "os.name"],
    [row.os?.version, "os.version"], [row.os?.build, "os.build"], [row.runtime?.name, "runtime.name"],
    [row.runtime?.version, "runtime.version"], [row.runtime?.engine, "runtime.engine"],
    [row.gpu?.vendor, "gpu.vendor"], [row.gpu?.renderer, "gpu.renderer"],
    [row.gpu?.webglVersion, "gpu.webglVersion"], [row.memory?.method, "memory.method"],
    [row.reviewer, "reviewer"],
  ]) requireText(errors, value, `${root}.${path}`);
  if (!digest(row.device?.hardwareIdHash)) errors.push(`${root}.device.hardwareIdHash must be a privacy-preserving SHA-256`);
  const runtime = `${row.device?.manufacturer ?? ""} ${row.device?.model ?? ""} ${row.runtime?.name ?? ""} ${row.runtime?.engine ?? ""}`;
  if (/simulator|emulator|headless|swiftshader|\bgeneric\b|\bsdk(?:_|\b)/i.test(runtime)) errors.push(`${root}.runtime must identify a physical-device browser engine`);
  for (const field of ["cssWidth", "cssHeight", "dpr"]) if (!finite(row.viewport?.[field]) || row.viewport[field] === 0) errors.push(`${root}.viewport.${field} must be finite and positive`);
  for (const field of ["maxTextureSize", "maxRenderbufferSize"]) if (!integer(row.gpu?.[field]) || row.gpu[field] === 0) errors.push(`${root}.gpu.${field} must be a positive integer`);
  if (!text(row.network?.origin) || !/^https:\/\//.test(row.network.origin)) errors.push(`${root}.network.origin must be an HTTPS deployment origin`);
  if (row.network?.externalRequests !== 0) errors.push(`${root}.network.externalRequests must equal 0`);
  if (row.workload?.cycles !== 3) errors.push(`${root}.workload.cycles must equal 3`);
  if (row.workload?.paths !== 5 || row.workload?.successfulPaths !== 5) errors.push(`${root}.workload must complete all 5 selected-candidate paths`);
  for (const interaction of REQUIRED_INTERACTIONS) if (!row.workload?.interactions?.includes(interaction)) errors.push(`${root}.workload.interactions must include ${interaction}`);
  if (!integer(row.workload?.maxCanvasPixels) || row.workload.maxCanvasPixels > MAX_CANVAS_PIXELS) errors.push(`${root}.workload.maxCanvasPixels must be an integer <= ${MAX_CANVAS_PIXELS}`);
  if (row.workload?.contextLosses !== 0) errors.push(`${root}.workload.contextLosses must equal 0`);
  if (!Array.isArray(row.workload?.errors) || row.workload.errors.length !== 0) errors.push(`${root}.workload.errors must be an empty array`);
  for (const field of ["baselineBytes", "peakBytes", "settledBytes", "deltaPeakBytes", "deltaSettledBytes"]) if (!integer(row.memory?.[field])) errors.push(`${root}.memory.${field} must be a non-negative integer`);
  if (integer(row.memory?.baselineBytes) && integer(row.memory?.peakBytes) && row.memory.peakBytes < row.memory.baselineBytes) errors.push(`${root}.memory.peakBytes cannot be below baselineBytes`);
  if (integer(row.memory?.settledBytes) && integer(row.memory?.peakBytes) && row.memory.settledBytes > row.memory.peakBytes) errors.push(`${root}.memory.settledBytes cannot exceed peakBytes`);
  if (integer(row.memory?.baselineBytes) && integer(row.memory?.peakBytes) && integer(row.memory?.deltaPeakBytes) && row.memory.deltaPeakBytes !== row.memory.peakBytes - row.memory.baselineBytes) errors.push(`${root}.memory.deltaPeakBytes does not match peakBytes - baselineBytes`);
  if (integer(row.memory?.baselineBytes) && integer(row.memory?.settledBytes) && integer(row.memory?.deltaSettledBytes) && row.memory.deltaSettledBytes !== Math.max(0, row.memory.settledBytes - row.memory.baselineBytes)) errors.push(`${root}.memory.deltaSettledBytes does not match settledBytes - baselineBytes`);
  artifactErrors(errors, row.memory?.samplesArtifact, `${root}.memory.samplesArtifact`);
  if (!Array.isArray(row.artifacts) || row.artifacts.length === 0) errors.push(`${root}.artifacts must contain raw evidence`);
  else row.artifacts.forEach((artifact, artifactIndex) => artifactErrors(errors, artifact, `${root}.artifacts[${artifactIndex}]`));
  if (!instant(row.measuredAt)) errors.push(`${root}.measuredAt must be an ISO date-time`);
}

export function validateDeviceEvidence(value) {
  const errors = [];
  if (!value || typeof value !== "object" || Array.isArray(value)) return ["evidence must be a JSON object"];
  if (value.schema !== SCHEMA) errors.push(`schema must be ${SCHEMA}`);
  if (value.candidate?.name !== CANDIDATE.name || value.candidate?.version !== CANDIDATE.version) errors.push(`candidate must be ${CANDIDATE.name} ${CANDIDATE.version}`);
  if (value.candidate?.sourceCommit !== SOURCE_COMMIT) errors.push(`candidate.sourceCommit must equal ${SOURCE_COMMIT}`);
  if (!/^[a-f0-9]{40}$/.test(value.candidate?.routeCommit ?? "")) errors.push("candidate.routeCommit must be an exact 40-character repository Git SHA");
  artifactErrors(errors, value.candidate?.routeArtifact, "candidate.routeArtifact");
  requireText(errors, value.reviewer, "reviewer");
  if (!instant(value.measuredAt)) errors.push("measuredAt must be an ISO date-time");
  const rows = Array.isArray(value.rows) ? value.rows : [];
  for (const platform of PLATFORMS) {
    const count = rows.filter((row) => row?.platform === platform).length;
    if (count !== 1) errors.push(`rows must contain exactly one physical ${platform} row (found ${count})`);
  }
  rows.forEach((row, index) => validateRow(errors, row, index));
  return errors;
}

async function verifyArtifact(root, artifact, label) {
  if (!artifact || !text(artifact.path) || !digest(artifact.sha256)) return [];
  const path = resolve(root, artifact.path);
  const local = relative(root, path);
  if (isAbsolute(local) || local === ".." || local.startsWith(`..${sep}`)) return [`${label}.path escapes the evidence directory`];
  try {
    const bytes = await readFile(path);
    return createHash("sha256").update(bytes).digest("hex") === artifact.sha256 ? [] : [`${label}.sha256 mismatch`];
  } catch (error) {
    return [`${label} cannot be read: ${error instanceof Error ? error.message : String(error)}`];
  }
}

export async function validateDeviceEvidenceFile(path) {
  const absolute = resolve(path);
  const value = JSON.parse(await readFile(absolute, "utf8"));
  const errors = validateDeviceEvidence(value);
  const root = dirname(absolute);
  errors.push(...await verifyArtifact(root, value.candidate?.routeArtifact, "candidate.routeArtifact"));
  for (const [index, row] of (Array.isArray(value.rows) ? value.rows : []).entries()) {
    errors.push(...await verifyArtifact(root, row.memory?.samplesArtifact, `rows[${index}].memory.samplesArtifact`));
    for (const [artifactIndex, artifact] of (Array.isArray(row.artifacts) ? row.artifacts : []).entries()) errors.push(...await verifyArtifact(root, artifact, `rows[${index}].artifacts[${artifactIndex}]`));
  }
  return errors;
}

async function main() {
  if (process.argv.length !== 3) throw new Error("usage: node tools/brd080-device-evidence.mjs <evidence.json>");
  const errors = await validateDeviceEvidenceFile(process.argv[2]);
  if (errors.length) {
    process.stderr.write(`BRD-080 physical-device evidence incomplete (${errors.length} issue${errors.length === 1 ? "" : "s"}):\n${errors.map((error) => `- ${error}`).join("\n")}\n`);
    process.exitCode = 1;
  } else process.stdout.write("BRD-080 physical-device evidence complete: Android and iOS physical runs and artifact hashes pass\n");
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main().catch((error) => { process.stderr.write(`${error.message}\n`); process.exitCode = 2; });
