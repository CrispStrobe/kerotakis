import assert from 'node:assert/strict';
import { mkdtemp, mkdir, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { compareBudget, measureFrontendAssets } from './frontend-asset-budget.mjs';

test('measures only JS and CSS in stable path order', async (t) => {
  const root = await mkdtemp(join(tmpdir(), 'kerotakis-assets-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  await mkdir(join(root, 'assets'));
  await writeFile(join(root, 'assets', 'z.css'), 'body{}\n');
  await writeFile(join(root, 'a.js'), 'export{}\n');
  await writeFile(join(root, 'ignored.map'), '{}');

  const first = await measureFrontendAssets(root);
  const second = await measureFrontendAssets(root);
  assert.deepEqual(first, second);
  assert.deepEqual(first.files.map(({ path }) => path), ['a.js', 'assets/z.css']);
  assert.equal(first.totals.all.rawBytes, first.totals.javascript.rawBytes + first.totals.css.rawBytes);
  assert.equal(first.totals.all.gzipBytes, first.totals.javascript.gzipBytes + first.totals.css.gzipBytes);
});

test('reports every exceeded limit and accepts an exact baseline', () => {
  const report = { totals: {
    javascript: { rawBytes: 100, gzipBytes: 50 },
    css: { rawBytes: 20, gzipBytes: 10 },
    all: { rawBytes: 120, gzipBytes: 60 },
  } };
  assert.deepEqual(compareBudget(report, { version: 1, limits: { all: { rawBytes: 120, gzipBytes: 60 } } }), []);
  assert.deepEqual(compareBudget(report, { version: 1, limits: { javascript: { gzipBytes: 49 }, all: { rawBytes: 119 } } }), [
    { kind: 'javascript', metric: 'gzipBytes', actual: 50, limit: 49, overBy: 1 },
    { kind: 'all', metric: 'rawBytes', actual: 120, limit: 119, overBy: 1 },
  ]);
});

test('rejects malformed and unknown limits', () => {
  const report = { totals: { javascript: {}, css: {}, all: {} } };
  assert.throws(() => compareBudget(report, { version: 1, limits: { images: { rawBytes: 1 } } }), /unknown budget kind/);
  assert.throws(() => compareBudget(report, { version: 1, limits: { all: { gzipBytes: -1 } } }), /invalid limit/);
});

test('rejects an empty or wrong build directory', async (t) => {
  const root = await mkdtemp(join(tmpdir(), 'kerotakis-empty-assets-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(join(root, 'index.html'), '<!doctype html>');
  await assert.rejects(measureFrontendAssets(root), /no \.js or \.css assets found/);
});
