#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(import.meta.dirname, "..");
const cssPath = path.join(root, "web/app/src/app.css");
const css = fs.readFileSync(cssPath, "utf8");

const themes = new Map();
const blockPattern = /:root(?:\[data-theme="([^"]+)"\])?\s*\{([^}]+)\}/g;
for (const match of css.matchAll(blockPattern)) {
  const name = match[1] ?? "light";
  const tokens = new Map();
  for (const declaration of match[2].matchAll(/(--[\w-]+)\s*:\s*([^;]+);/g)) {
    tokens.set(declaration[1], declaration[2].trim());
  }
  themes.set(name, tokens);
}

const requiredThemes = ["light", "dark", "contrast"];
const surfaces = ["--lab-bg", "--surface", "--surface-raised", "--bench"];
const accents = [
  "--primary",
  "--instrument",
  "--action",
  "--discovery",
  "--success",
  "--warning",
  "--danger",
];

function rgb(hex) {
  const normalized = hex.toLowerCase();
  const short = /^#([0-9a-f]{3})$/.exec(normalized);
  const long = /^#([0-9a-f]{6})$/.exec(normalized);
  if (!short && !long) throw new Error(`expected a literal hex colour, got ${hex}`);
  const digits = short
    ? [...short[1]].map((digit) => `${digit}${digit}`).join("")
    : long[1];
  return [0, 2, 4].map((offset) => Number.parseInt(digits.slice(offset, offset + 2), 16));
}

function luminance(hex) {
  const channels = rgb(hex).map((channel) => {
    const value = channel / 255;
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  return channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722;
}

function ratio(foreground, background) {
  const [lighter, darker] = [luminance(foreground), luminance(background)].sort(
    (left, right) => right - left,
  );
  return (lighter + 0.05) / (darker + 0.05);
}

const failures = [];
function requireRatio(theme, foregroundName, backgroundName, minimum) {
  const foreground = theme.get(foregroundName);
  const background = theme.get(backgroundName);
  if (!foreground || !background) {
    failures.push(`missing ${foregroundName} or ${backgroundName}`);
    return;
  }
  const actual = ratio(foreground, background);
  if (actual < minimum) {
    failures.push(
      `${foregroundName} ${foreground} on ${backgroundName} ${background}: ` +
        `${actual.toFixed(2)}:1 (needs ${minimum}:1)`,
    );
  }
}

for (const name of requiredThemes) {
  const theme = themes.get(name);
  if (!theme) {
    failures.push(`missing ${name} theme`);
    continue;
  }
  for (const surface of surfaces) {
    requireRatio(theme, "--ink", surface, 4.5);
    requireRatio(theme, "--ink-muted", surface, 4.5);
  }
  for (const accent of accents) requireRatio(theme, "--on-accent", accent, 4.5);
}

const componentRoot = path.join(root, "web/app/src/lib/components");
for (const entry of fs.readdirSync(componentRoot, { recursive: true, withFileTypes: true })) {
  if (!entry.isFile() || !entry.name.endsWith(".svelte")) continue;
  const file = path.join(entry.parentPath, entry.name);
  const source = fs.readFileSync(file, "utf8");
  if (/color\s*:\s*(?:white|#fff(?:fff)?)(?:\s*;)/i.test(source)) {
    failures.push(`${path.relative(root, file)} bypasses --on-accent with fixed white text`);
  }
}

if (failures.length > 0) {
  console.error("Theme contrast gate failed:\n");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}

console.log("Theme contrast gate passed: light, dark, and high-contrast token pairs meet WCAG AA.");
