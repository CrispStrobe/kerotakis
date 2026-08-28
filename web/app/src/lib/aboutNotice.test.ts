/** The credits screen says what NOTICE says.
 *
 * NOTICE is the legal source of truth for what this app embeds. The About
 * dialog is the only place a reader will ever see that list, and a
 * hand-kept copy of it would drift — silently, because nobody diffs a
 * credits screen against a licence file.
 *
 * So the dialog reads a generated file, and this fails when regenerating
 * would change it. Adding a component to NOTICE and forgetting the dialog
 * is a red test rather than a quiet omission.
 */
import { describe, expect, it } from "vitest";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { COPYRIGHT, NOTICE_SECTIONS, THIRD_PARTY_LICENSES } from "./about";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "../../../..");

describe("the About dialog's third-party list", () => {
  it("is current with NOTICE", () => {
    // The generator exits non-zero when the committed file is stale, and
    // prints the command that fixes it.
    expect(() =>
      execFileSync("node", [join(ROOT, "tools/about-notice.mjs")], { cwd: ROOT }),
    ).not.toThrow();
  });

  it("carries every component NOTICE lists", () => {
    // Counted from NOTICE directly rather than from the generated file:
    // a generator that silently dropped a section would otherwise agree
    // with itself.
    const notice = readFileSync(join(ROOT, "NOTICE"), "utf8");
    const bullets = notice.split("\n").filter((l) => l.startsWith("- ")).length;
    const shown = NOTICE_SECTIONS.reduce((n, s) => n + s.entries.length, 0);
    expect(shown).toBe(bullets);
  });

  it("names the things a reader would look for", () => {
    const all = NOTICE_SECTIONS.flatMap((s) => s.entries).join("\n");
    // IPhreeqc is the one that does the chemistry, and its licence is the
    // unusual one — a USGS notice rather than an SPDX identifier.
    expect(all).toContain("IPhreeqc");
    expect(all).toContain("USGS User Rights Notice");
    for (const name of ["MY-BASIC", "NASA CEA", "diffsol", "chematic", "Svelte"]) {
      expect(all).toContain(name);
    }
    expect(all).not.toContain("CoolProp");
    expect(all).not.toContain("xtb / CREST");
  });

  it("identifies the copyright holder and bundled licence document", () => {
    expect(COPYRIGHT).toContain("Christian Ströbele");
    expect(THIRD_PARTY_LICENSES).toContain("legal/third-party-licenses.html");
  });

  it("bundles every locked runtime dependency and principal vendored notice", () => {
    const bundled = readFileSync(
      join(ROOT, "web/app/public/legal/third-party-licenses.html"),
      "utf8",
    );
    const packageKeys = new Set(
      [...bundled.matchAll(/data-package="([^"]+)" data-version="([^"]+)"/g)].map(
        ([, name, version]) => `${name}@${version}`,
      ),
    );
    const inventory = JSON.parse(readFileSync(join(ROOT, "data/inventory.json"), "utf8"));
    for (const item of inventory.external_dependencies) {
      expect(packageKeys.has(`${item.name}@${item.version}`)).toBe(true);
    }

    const npmLock = JSON.parse(readFileSync(join(ROOT, "web/app/package-lock.json"), "utf8"));
    for (const [path, item] of Object.entries<any>(npmLock.packages)) {
      if (!path.startsWith("node_modules/") || item.dev === true) continue;
      const name = path.replace(/^node_modules\//, "");
      expect(packageKeys.has(`${name}@${item.version}`)).toBe(true);
    }

    const tauriLock = readFileSync(join(ROOT, "web/app/src-tauri/Cargo.lock"), "utf8");
    for (const block of tauriLock.split("[[package]]").slice(1)) {
      if (!/^\s*source = "registry\+/m.test(block)) continue;
      const name = block.match(/^name = "([^"]+)"/m)?.[1];
      const version = block.match(/^version = "([^"]+)"/m)?.[1];
      expect(name && version && packageKeys.has(`${name}@${version}`)).toBe(true);
    }

    for (const component of ["IPhreeqc / PHREEQC", "MY-BASIC", "NASA CEA"]) {
      expect(bundled).toContain(`data-component="${component}"`);
    }
  });

  it("records the exact dependency inputs used to generate the bundle", () => {
    const bundled = readFileSync(
      join(ROOT, "web/app/public/legal/third-party-licenses.html"),
      "utf8",
    );
    for (const [marker, path] of [
      ["rust-inventory", "data/inventory.json"],
      ["tauri-lock", "web/app/src-tauri/Cargo.lock"],
      ["npm-lock", "web/app/package-lock.json"],
      ["project-license", "LICENSE"],
      ["project-notice", "NOTICE"],
      ["iphreeqc-notice", "vendor/iphreeqc/doc/NOTICE"],
      ["my-basic-license", "vendor/my-basic/LICENSE"],
      ["nasa-cea-notice", "vendor/nasa-cea/NOTICE.txt"],
      ["nasa-cea-license", "vendor/nasa-cea/LICENSE.txt"],
    ]) {
      const hash = createHash("sha256").update(readFileSync(join(ROOT, path))).digest("hex");
      expect(bundled).toContain(`name="kerotakis-${marker}-sha256" content="${hash}"`);
    }
    expect(bundled).toContain('id="kerotakis-license"');
    expect(bundled).toContain('data-component="Kerotakis"');
  });

  it("names the principal upstream authors, roles and sources", () => {
    const bundled = readFileSync(
      join(ROOT, "web/app/public/legal/third-party-licenses.html"),
      "utf8",
    );
    for (const expected of [
      "S.R. Charlton and D.L. Parkhurst",
      "D.L. Parkhurst and C.A.J. Appelo",
      "Copyright © 2011–2026 Tony Wang",
      "Administrator of the National Aeronautics and Space Administration",
      "https://github.com/CrispStrobe/iphreeqc",
      "https://github.com/paladin-t/my_basic",
      "https://github.com/nasa/cea",
      "Role in Kerotakis / Aufgabe in Kerotakis",
    ]) {
      expect(bundled).toContain(expected);
    }
  });

  it("puts the AGPL notice in the interactive UI in both languages", () => {
    const dialog = readFileSync(
      join(ROOT, "web/app/src/lib/components/AboutDialog.svelte"),
      "utf8",
    );
    const de = readFileSync(join(ROOT, "web/app/src/locales/de.json"), "utf8");
    for (const phrase of ["no warranty", "use, share and modify", "corresponding source"]) {
      expect(dialog).toContain(phrase);
    }
    for (const phrase of ["ohne Gewährleistung", "teilen und verändern", "zugehörigen Quellcode"]) {
      expect(de).toContain(phrase);
    }
  });

  it("keeps both privacy documents honest about the macOS entitlement", () => {
    const en = readFileSync(join(ROOT, "web/privacy.html"), "utf8");
    const de = readFileSync(join(ROOT, "web/privacy.de.html"), "utf8");
    expect(en).toContain("com.apple.security.network.client");
    expect(de).toContain("com.apple.security.network.client");
    expect(de).not.toContain("besitzt nicht einmal eine Netzwerkberechtigung");
  });

  it("does not invent licences the file never claims", () => {
    // An earlier generator split each bullet into {name, licence} and got
    // "Apache-2.0" wrong, called serde a licence, and turned two prose
    // paragraphs into components. Entries are verbatim now; this pins that.
    const notice = readFileSync(join(ROOT, "NOTICE"), "utf8").replace(/\s+/g, " ");
    for (const entry of NOTICE_SECTIONS.flatMap((s) => s.entries)) {
      expect(notice).toContain(entry.replace(/\s+/g, " "));
    }
  });
});
