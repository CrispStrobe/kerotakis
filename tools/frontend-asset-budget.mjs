#!/usr/bin/env node

import { gzipSync } from 'node:zlib';
import { readFile, readdir } from 'node:fs/promises';
import { resolve, relative, sep } from 'node:path';
import { pathToFileURL } from 'node:url';

const KINDS = Object.freeze({ '.js': 'javascript', '.css': 'css' });

async function assetPaths(root, directory = root) {
  const entries = await readdir(directory, { withFileTypes: true });
  const paths = [];
  for (const entry of entries.sort((a, b) => a.name < b.name ? -1 : a.name > b.name ? 1 : 0)) {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) paths.push(...await assetPaths(root, path));
    else if (entry.isFile() && Object.hasOwn(KINDS, entry.name.slice(entry.name.lastIndexOf('.')))) paths.push(path);
  }
  return paths;
}

function emptySizes() {
  return { rawBytes: 0, gzipBytes: 0 };
}

export async function measureFrontendAssets(directory) {
  const root = resolve(directory);
  const files = [];
  for (const path of await assetPaths(root)) {
    const bytes = await readFile(path);
    files.push({
      path: relative(root, path).split(sep).join('/'),
      kind: KINDS[path.slice(path.lastIndexOf('.'))],
      rawBytes: bytes.byteLength,
      // level 9 and mtime 0 make the byte count independent of host and clock.
      gzipBytes: gzipSync(bytes, { level: 9, mtime: 0 }).byteLength,
    });
  }
  if (files.length === 0) throw new Error(`no .js or .css assets found in ${root}`);

  const totals = { javascript: emptySizes(), css: emptySizes(), all: emptySizes() };
  for (const file of files) {
    for (const key of [file.kind, 'all']) {
      totals[key].rawBytes += file.rawBytes;
      totals[key].gzipBytes += file.gzipBytes;
    }
  }
  return { version: 1, files, totals };
}

export function compareBudget(report, budget) {
  if (budget.version !== 1 || !budget.limits || typeof budget.limits !== 'object') {
    throw new Error('budget must have version 1 and a limits object');
  }
  const failures = [];
  for (const [kind, limits] of Object.entries(budget.limits)) {
    if (!Object.hasOwn(report.totals, kind)) throw new Error(`unknown budget kind: ${kind}`);
    for (const [metric, limit] of Object.entries(limits)) {
      if (!['rawBytes', 'gzipBytes'].includes(metric) || !Number.isSafeInteger(limit) || limit < 0) {
        throw new Error(`invalid limit: ${kind}.${metric}`);
      }
      const actual = report.totals[kind][metric];
      if (actual > limit) failures.push({ kind, metric, actual, limit, overBy: actual - limit });
    }
  }
  return failures;
}

function usage() {
  return 'usage: node tools/frontend-asset-budget.mjs --dir PATH [--budget FILE] [--json]';
}

async function main(argv) {
  let directory;
  let budgetPath;
  let json = false;
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === '--dir') directory = argv[++i];
    else if (argv[i] === '--budget') budgetPath = argv[++i];
    else if (argv[i] === '--json') json = true;
    else throw new Error(`${usage()}\nunknown argument: ${argv[i]}`);
  }
  if (!directory) throw new Error(usage());

  const report = await measureFrontendAssets(directory);
  const failures = budgetPath
    ? compareBudget(report, JSON.parse(await readFile(resolve(budgetPath), 'utf8')))
    : [];
  const result = { ...report, budget: budgetPath ?? null, failures };
  if (json) process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
  else {
    for (const kind of ['javascript', 'css', 'all']) {
      const value = report.totals[kind];
      process.stdout.write(`${kind}: ${value.rawBytes} raw bytes, ${value.gzipBytes} gzip bytes\n`);
    }
    for (const failure of failures) {
      process.stderr.write(`budget exceeded: ${failure.kind}.${failure.metric} ${failure.actual} > ${failure.limit} (+${failure.overBy})\n`);
    }
  }
  if (failures.length) process.exitCode = 1;
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  main(process.argv.slice(2)).catch((error) => {
    process.stderr.write(`${error.message}\n`);
    process.exitCode = 2;
  });
}
