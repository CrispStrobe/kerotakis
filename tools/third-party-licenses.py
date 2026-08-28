#!/usr/bin/env python3
"""Generate the legal document shown by About -> Open-source licences.

The committed HTML is part of every web/native payload.  Package names and
licence expressions come from the checked-in Rust inventory and npm lockfile;
authors, repositories and verbatim notice files come from the package sources.
No code is compiled by this generator.
"""

from __future__ import annotations

import argparse
import hashlib
import html
import json
import os
from pathlib import Path
import tomllib


ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "web/app/public/legal/third-party-licenses.html"
NOTICE_NAMES = ("license", "licence", "copying", "notice", "unlicense")


def esc(value: object) -> str:
    return html.escape(str(value), quote=True)


def notice_files(directory: Path, declared: str | None = None) -> list[Path]:
    found: list[Path] = []
    if declared:
        candidate = directory / declared
        if candidate.is_file():
            found.append(candidate)
    if directory.is_dir():
        for candidate in sorted(directory.iterdir(), key=lambda p: p.name.lower()):
            if candidate.is_file() and candidate.name.lower().startswith(NOTICE_NAMES):
                if candidate not in found:
                    found.append(candidate)
    return found


def readable(path: Path) -> str:
    # Preserve wording exactly while dropping line-ending whitespace that has
    # no legal meaning and would make the committed generated file fail the
    # repository's whitespace check.
    return "\n".join(
        line.rstrip()
        for line in path.read_text(encoding="utf-8", errors="replace").strip().splitlines()
    )


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def notice_references(files: list[Path], catalogue: dict[str, tuple[str, str]]) -> str:
    links: list[str] = []
    for path in files:
        body = readable(path)
        digest = hashlib.sha256(body.encode()).hexdigest()
        catalogue.setdefault(digest, (path.name, body))
        links.append(f'<li><a href="#notice-{digest}">{esc(path.name)}</a></li>')
    return f"<ul>{''.join(links)}</ul>" if links else ""


def package_block(
    name: str,
    version: str,
    licence: str,
    authors: list[str],
    repository: str,
    files: list[Path],
    catalogue: dict[str, tuple[str, str]],
) -> str:
    identity = f"{name} {version}".strip()
    metadata = [f"Declared licence / Angegebene Lizenz: {licence or 'not declared / nicht angegeben'}"]
    if authors:
        metadata.append("Authors/maintainers / Urheber und Maintainer: " + "; ".join(authors))
    if repository:
        metadata.append("Upstream / Quelle: " + repository)
    notices = notice_references(files, catalogue)
    if not notices:
        notices = (
            "<p class=\"missing\">The published package contains no separate licence file; "
            "its manifest declaration is reproduced above. / Das veröffentlichte Paket enthält "
            "keine separate Lizenzdatei; die Angabe aus seinem Manifest steht oben.</p>"
        )
    return (
        f'<details data-package="{esc(name)}" data-version="{esc(version)}">'
        f"<summary>{esc(identity)} — {esc(licence or 'licence not declared')}</summary>"
        f"<p>{'<br>'.join(esc(line) for line in metadata)}</p>{notices}</details>"
    )


def cargo_packages(
    registry_root: Path, catalogue: dict[str, tuple[str, str]]
) -> list[str]:
    inventory = json.loads((ROOT / "data/inventory.json").read_text())
    source_dirs: dict[str, Path] = {}
    if registry_root.is_dir():
        for index in registry_root.iterdir():
            if not index.is_dir():
                continue
            for package in index.iterdir():
                if package.is_dir():
                    source_dirs.setdefault(package.name, package)

    packages: dict[tuple[str, str], dict[str, str]] = {
        (item["name"], item["version"]): item
        for item in inventory["external_dependencies"]
    }
    # The Tauri shell is deliberately a separate Cargo workspace. Its lockfile
    # is therefore an independent legal input, not a subset of the root
    # inventory. Registry packages have their licence metadata recovered from
    # the exact source directory below.
    tauri_lock = ROOT / "web/app/src-tauri/Cargo.lock"
    if tauri_lock.is_file():
        for item in tomllib.loads(readable(tauri_lock)).get("package", []):
            if str(item.get("source", "")).startswith("registry+"):
                packages.setdefault(
                    (str(item["name"]), str(item["version"])),
                    {"name": str(item["name"]), "version": str(item["version"]), "license": ""},
                )

    blocks: list[str] = []
    missing: list[str] = []
    for item in sorted(
        packages.values(), key=lambda p: (p["name"], p["version"])
    ):
        key = f'{item["name"]}-{item["version"]}'
        directory = source_dirs.get(key)
        authors: list[str] = []
        repository = ""
        declared_file: str | None = None
        licence = item.get("license") or ""
        if directory:
            manifest_path = directory / "Cargo.toml"
            if manifest_path.is_file():
                manifest = tomllib.loads(readable(manifest_path)).get("package", {})
                authors = [str(a) for a in manifest.get("authors", [])]
                repository = str(manifest.get("repository", ""))
                declared_file = manifest.get("license-file")
                licence = str(manifest.get("license", licence))
        else:
            missing.append(key)
        blocks.append(
            package_block(
                item["name"],
                item["version"],
                licence,
                authors,
                repository,
                notice_files(directory, declared_file) if directory else [],
                catalogue,
            )
        )
    if missing:
        raise SystemExit(
            "Cargo source cache is missing: " + ", ".join(missing[:12])
            + (" …" if len(missing) > 12 else "")
        )
    return blocks


def npm_packages(node_modules: Path, catalogue: dict[str, tuple[str, str]]) -> list[str]:
    lock = json.loads((ROOT / "web/app/package-lock.json").read_text())
    blocks: list[str] = []
    missing: list[str] = []
    for key, item in sorted(lock["packages"].items()):
        if not key.startswith("node_modules/") or item.get("dev") is True:
            continue
        name = key.removeprefix("node_modules/")
        directory = node_modules / name
        manifest_path = directory / "package.json"
        if not manifest_path.is_file():
            missing.append(name)
            continue
        manifest = json.loads(manifest_path.read_text())
        authors: list[str] = []
        author = manifest.get("author")
        if isinstance(author, str):
            authors.append(author)
        elif isinstance(author, dict) and author.get("name"):
            authors.append(str(author["name"]))
        for contributor in manifest.get("contributors", []) or []:
            if isinstance(contributor, str):
                authors.append(contributor)
            elif isinstance(contributor, dict) and contributor.get("name"):
                authors.append(str(contributor["name"]))
        repository_value = manifest.get("repository", "")
        if isinstance(repository_value, dict):
            repository = str(repository_value.get("url", ""))
        else:
            repository = str(repository_value)
        blocks.append(
            package_block(
                name,
                str(item.get("version", "")),
                str(item.get("license", manifest.get("license", ""))),
                authors,
                repository,
                notice_files(directory),
                catalogue,
            )
        )
    if missing:
        raise SystemExit("npm source tree is missing: " + ", ".join(missing))
    return blocks


def manual_block(
    title: str,
    lane: str,
    licence: str,
    files: list[Path],
    catalogue: dict[str, tuple[str, str]],
) -> str:
    missing = [str(path) for path in files if not path.is_file()]
    if missing:
        raise SystemExit("required notice file is missing: " + ", ".join(missing))
    notices = notice_references(files, catalogue)
    return (
        f'<details data-component="{esc(title)}"><summary>{esc(title)} — {esc(licence)}</summary>'
        f"<p>Distribution lane / Verteilungsweg: {esc(lane)}</p>{notices}</details>"
    )


def generate(registry_root: Path, node_modules: Path, iphreeqc_notice: Path) -> str:
    catalogue: dict[str, tuple[str, str]] = {}
    shipped = [
        manual_block(
            "IPhreeqc / PHREEQC",
            "runtime",
            "USGS User Rights Notice",
            [iphreeqc_notice],
            catalogue,
        ),
        manual_block(
            "MY-BASIC", "runtime", "MIT", [ROOT / "vendor/my-basic/LICENSE"], catalogue
        ),
        manual_block(
            "NASA CEA",
            "runtime",
            "Apache-2.0",
            [ROOT / "vendor/nasa-cea/NOTICE.txt", ROOT / "vendor/nasa-cea/LICENSE.txt"],
            catalogue,
        ),
    ]
    rust = cargo_packages(registry_root, catalogue)
    npm = npm_packages(node_modules, catalogue)
    notice_texts = "".join(
        f'<section id="notice-{digest}"><h3>{esc(name)} — SHA-256 {digest}</h3>'
        f"<pre>{esc(body)}</pre></section>"
        for digest, (name, body) in catalogue.items()
    )
    cargo_lock = ROOT / "web/app/src-tauri/Cargo.lock"
    return f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width">
<meta name="kerotakis-rust-inventory-sha256" content="{sha256(ROOT / 'data/inventory.json')}">
<meta name="kerotakis-tauri-lock-sha256" content="{sha256(cargo_lock)}">
<meta name="kerotakis-npm-lock-sha256" content="{sha256(ROOT / 'web/app/package-lock.json')}">
<title>Kerotakis open-source licences / Open-Source-Lizenzen</title>
<style>
:root{{color-scheme:light dark;font-family:system-ui,sans-serif}}body{{max-width:76rem;margin:auto;padding:1.25rem;line-height:1.45}}
h1,h2{{line-height:1.15}}details{{border:1px solid #7894aa;border-radius:.65rem;margin:.55rem 0;padding:.65rem}}
summary{{cursor:pointer;font-weight:700}}pre{{white-space:pre-wrap;overflow-wrap:anywhere;background:#0001;padding:.75rem;border-radius:.4rem}}
.missing{{border-left:.25rem solid #c97500;padding-left:.65rem}}a{{color:inherit}}
</style></head><body>
<h1>Kerotakis open-source licences / Open-Source-Lizenzen</h1>
<p lang="en">This document is bundled with the app. It records the package versions used by the audited source tree and reproduces every available licence or notice file from those packages. A manifest declaration is shown explicitly where an upstream package contains no separate licence file.</p>
<p lang="de">Dieses Dokument ist in der App enthalten. Es nennt die im geprüften Quellbaum verwendeten Paketversionen und gibt jede dort verfügbare Lizenz- oder Hinweisedatei wieder. Wenn ein Upstream-Paket keine separate Lizenzdatei enthält, wird die Angabe aus seinem Manifest ausdrücklich angezeigt.</p>
<h2>Vendored runtime components / Eingebundene Laufzeitkomponenten</h2>{''.join(shipped)}
<h2>Rust dependency inventory / Rust-Abhängigkeiten ({len(rust)})</h2>{''.join(rust)}
<h2>Web runtime dependency inventory / Web-Laufzeitabhängigkeiten ({len(npm)})</h2>{''.join(npm)}
<h2>Licence and notice texts / Lizenz- und Hinweistexte ({len(catalogue)})</h2>{notice_texts}
</body></html>"""


def main() -> None:
    parser = argparse.ArgumentParser()
    default_cargo = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo")) / "registry/src"
    parser.add_argument("--cargo-registry-root", type=Path, default=default_cargo)
    parser.add_argument("--node-modules", type=Path, default=ROOT / "web/app/node_modules")
    parser.add_argument(
        "--iphreeqc-notice", type=Path, default=ROOT / "vendor/iphreeqc/doc/NOTICE"
    )
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    generated = generate(args.cargo_registry_root, args.node_modules, args.iphreeqc_notice)
    if args.check:
        if not OUT.is_file() or OUT.read_text() != generated:
            raise SystemExit("third-party licence bundle is stale; run tools/third-party-licenses.py")
        return
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(generated)
    print(f"wrote {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
