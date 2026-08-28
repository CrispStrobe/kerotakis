#!/usr/bin/env node
// NOTICE is the legal source of truth for what this app embeds. The About
// dialog has to say the same thing, and a hand-kept second list would
// drift — silently, because nobody diffs a credits screen against a
// licence file.
//
// So the dialog reads a generated file, and `aboutNotice.test.ts` fails if
// regenerating would change it. Adding a component to NOTICE and
// forgetting the dialog is a red test rather than a quiet omission.
//
// The bullets are carried across verbatim. An earlier version tried to
// split each one into {name, licence, detail} and got "Apache-2.0" wrong
// (truncated at the dot), called serde a licence, and turned two prose
// paragraphs into the names of imaginary components. NOTICE is prose with
// bullets, not a table; pretending otherwise produced a credits screen
// that stated licences the file never claimed. Verbatim cannot be wrong.
//
//   node tools/about-notice.mjs          # check (exit 1 if stale)
//   node tools/about-notice.mjs --write  # regenerate
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const NOTICE = join(ROOT, "NOTICE");
const OUT = join(ROOT, "web/app/src/lib/generated/notice.json");

function parse(text) {
  const lines = text.split("\n");
  const sections = [];
  let current = null;

  const nextMeaningful = (i) => {
    for (let j = i + 1; j < lines.length; j++) {
      if (lines[j].trim()) return lines[j];
    }
    return "";
  };

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (!line.trim()) continue;

    // A heading is a short line that introduces bullets, either underlined
    // with a rule or immediately followed by one.
    const underlined = /^-{3,}$/.test((lines[i + 1] ?? "").trim());
    const introducesBullets = nextMeaningful(i).startsWith("- ");
    if (!line.startsWith("- ") && !/^\s/.test(line) && (underlined || introducesBullets)) {
      if (underlined && !nextMeaningfulAfterRule(lines, i)) continue;
      current = { title: line.trim().replace(/:$/, ""), entries: [] };
      sections.push(current);
      if (underlined) i++;
      continue;
    }

    if (current && line.startsWith("- ")) {
      let body = line.slice(2).trim();
      while (/^\s{2,}\S/.test(lines[i + 1] ?? "")) {
        body += " " + lines[++i].trim();
      }
      current.entries.push(body);
    }
  }
  return sections.filter((s) => s.entries.length > 0);
}

/** A rule under a title is only a heading if bullets follow it. */
function nextMeaningfulAfterRule(lines, i) {
  for (let j = i + 2; j < lines.length; j++) {
    if (!lines[j].trim()) continue;
    return lines[j].startsWith("- ") || /^[A-Z]/.test(lines[j]);
  }
  return false;
}

const sections = parse(readFileSync(NOTICE, "utf8"));
const total = sections.reduce((n, s) => n + s.entries.length, 0);
if (total < 10) {
  console.error(`only ${total} entries parsed from NOTICE — the parser is broken, not the file`);
  process.exit(1);
}
const json = JSON.stringify({ sections }, null, 1) + "\n";

if (process.argv.includes("--write")) {
  mkdirSync(dirname(OUT), { recursive: true });
  writeFileSync(OUT, json);
  console.log(`notice.json: ${sections.length} sections, ${total} entries`);
} else {
  let existing = "";
  try {
    existing = readFileSync(OUT, "utf8");
  } catch {
    /* not generated yet */
  }
  if (existing !== json) {
    console.error("notice.json is stale — run: node tools/about-notice.mjs --write");
    process.exit(1);
  }
  console.log(`notice.json is current: ${sections.length} sections, ${total} entries`);
}
