#!/usr/bin/env python3
"""Every workflow must install what the scripts it reaches actually need.

Two requirements, both learned by a workflow failing the first time it
was ever run:

**Python.** `tools/build-shell-payload.sh` calls `tomllib`, which is 3.11
and newer.

`tomllib` is Python 3.11 and newer. The Linux half of the v0.1.0 release died
because `release.yml` ran `tools/build-shell-payload.sh` on `ubuntu-22.04`,
whose system python3 is 3.10 — and CI never saw it, because CI's own payload
build ran on `ubuntu-latest`.

Two things make this worth a script rather than a grep, and both were mistakes
made while writing it:

**The requirement is `tomllib`, not one script.** A first version scanned for
`tools/build-shell-payload.sh` and its callers. That misses `tools/build-web.sh`,
which does not call the payload script at all — it invokes the same four Python
tools itself. So the seed here is "imports tomllib, directly or through a module
that does", and the edges are followed out from there.

**Reach is indirect.** A workflow can arrive at that Python without naming
anything:

    appstore.yml
      -> tools/build-macos-appstore.sh
        -> npx tauri build
          -> beforeBuildCommand: npm run build:shell
            -> npm run payload
              -> tools/build-shell-payload.sh
                -> tools/lessons-index.py -> tools/lesson_prose.py -> tomllib

Comments are stripped before matching, or the lint reports its own prose as the
call chain: the comment this change added to `appstore.yml` named the payload
script, and `build-macos-appstore.sh` mentions `tauri build` twelve lines above
the real one.
"""
from __future__ import annotations
import argparse, json, pathlib, re, sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
WORKFLOWS = ROOT / ".github/workflows"


def uncommented(text: str) -> str:
    """Drop `#` comments, so a token is only found where it is actually run."""
    return "\n".join(line[:cut] if (cut := line.find("#")) >= 0 else line
                     for line in text.splitlines())


def python_tools_needing(module: str, subdirs: tuple[str, ...] = ()) -> set[str]:
    """tools/*.py that import `module`, directly or via a sibling that does.

    The transitive half is the point: `lessons-index.py` never says tomllib,
    `lesson_prose.py` does; no `tools/asc/*.py` says cryptography except
    `client.py`, and every one of them imports client.
    """
    globs = ["tools/*.py"] + [f"tools/{d}/*.py" for d in subdirs]
    sources = {p.name: uncommented(p.read_text(encoding="utf-8"))
               for g in globs for p in ROOT.glob(g)}
    needs = {name for name, src in sources.items()
             if re.search(rf"^\s*(import|from) {re.escape(module)}\b", src, re.M)}
    changed = True
    while changed:
        changed = False
        for name, src in sources.items():
            if name in needs:
                continue
            for other in needs:
                module = other[:-3]
                if re.search(rf"^\s*(import|from) {re.escape(module)}\b", src, re.M):
                    needs.add(name)
                    changed = True
                    break
    return needs


def npm_scripts_reaching(tokens: set[str]) -> set[str]:
    pkg = json.loads((ROOT / "web/app/package.json").read_text(encoding="utf-8"))
    scripts: dict[str, str] = pkg.get("scripts", {})
    reaching = {name for name, body in scripts.items()
                if any(tok in body for tok in tokens)}
    changed = True
    while changed:
        changed = False
        for name, body in scripts.items():
            if name in reaching:
                continue
            if any(re.search(rf"npm run {re.escape(dep)}\b", body) for dep in reaching):
                reaching.add(name)
                changed = True
    return reaching


def reaching_tokens() -> list[str]:
    """Anything whose presence in a workflow means a >=3.11 Python is needed.

    One interleaved fixed point, not three passes in sequence: an early version
    grew the shell scripts first and added `tauri build` afterwards, so
    `tools/build-macos-appstore.sh` — which reaches the payload only through
    `npx tauri build` — was never picked up, and `appstore.yml` silently
    dropped out of the report it had just been failing.
    """
    tokens = set(python_tools_needing("tomllib"))
    conf = json.loads((ROOT / "web/app/src-tauri/tauri.conf.json").read_text(encoding="utf-8"))
    before = conf.get("build", {}).get("beforeBuildCommand", "")

    while True:
        grew = False

        for script in sorted(ROOT.glob("tools/*.sh")):
            name = f"tools/{script.name}"
            if name in tokens:
                continue
            if any(tok in uncommented(script.read_text(encoding="utf-8"))
                   for tok in tokens):
                tokens.add(name)
                grew = True

        for name in npm_scripts_reaching(tokens):
            if f"npm run {name}" not in tokens:
                tokens.add(f"npm run {name}")
                grew = True
            # tauri runs beforeBuildCommand itself, so `tauri build` reaches
            # whichever npm script that config names.
            if name in before and "tauri build" not in tokens:
                tokens.add("tauri build")
                grew = True

        if not grew:
            return sorted(tokens)


def wasm_building_scripts() -> list[str]:
    """tools/*.sh that compile for wasm, plus their callers."""
    marks = ("wasm32-unknown-unknown", "wasm-bindgen")
    found: list[str] = []
    while True:
        grew = False
        for script in sorted(ROOT.glob("tools/*.sh")):
            name = f"tools/{script.name}"
            if name in found:
                continue
            body = uncommented(script.read_text(encoding="utf-8"))
            if any(m in body for m in marks) or any(tok in body for tok in found):
                found.append(name)
                grew = True
        if not grew:
            return found


def wasm_offenders() -> list[str]:
    """Workflows that build wasm without installing the target or bindgen.

    `vercel.yml` reached `tools/build-web.sh`, which compiles kerotakis-wasm,
    with a bare `dtolnay/rust-toolchain@stable`. It failed on its first real
    run with "the `wasm32-unknown-unknown` target may not be installed" — the
    workflow had existed for weeks and had never been dispatched.
    """
    scripts = wasm_building_scripts()
    bad = []
    for wf in sorted(WORKFLOWS.glob("*.yml")):
        text = uncommented(wf.read_text(encoding="utf-8"))
        builds_wasm = ("wasm32-unknown-unknown" in text
                       or any(s in text for s in scripts))
        if not builds_wasm:
            continue
        missing = []
        if "targets: wasm32-unknown-unknown" not in text:
            missing.append("targets: wasm32-unknown-unknown")
        # Only scripts that run wasm-bindgen itself need the CLI; a workflow
        # that merely `cargo build`s for the target does not.
        needs_bindgen = any(
            "wasm-bindgen" in uncommented((ROOT / s).read_text(encoding="utf-8"))
            for s in scripts if s in text and (ROOT / s).exists()
        )
        if needs_bindgen and "wasm-bindgen-cli" not in text:
            missing.append("cargo install wasm-bindgen-cli")
        if missing:
            bad.append(f"{wf.name}: builds wasm but has no {' and no '.join(missing)}")
    return bad


def cryptography_offenders() -> list[str]:
    """Workflows that run an ASC script without installing cryptography.

    `tools/asc/client.py` signs the App Store Connect JWT with it, and every
    other script in that directory imports client. `appstore.yml` got away
    without it for as long as it used the runner's own python3 — macos-latest
    ships cryptography preinstalled — and broke the moment a `setup-python`
    pin replaced that interpreter with a clean one. The fix for one problem
    uncovered the next, which is only visible if the requirement is stated
    rather than inherited.
    """
    needing = python_tools_needing("cryptography", subdirs=("asc",))
    if not needing:
        return ["payload-toolchain-lint: nothing imports cryptography — "
                "the scan is broken, not the repository"]
    bad = []
    for wf in sorted(WORKFLOWS.glob("*.yml")):
        text = uncommented(wf.read_text(encoding="utf-8"))
        if not any(f"tools/asc/{name}" in text for name in needing | {"client.py"}) \
                and "tools/asc/" not in text:
            continue
        if "pip install" in text and "cryptography" in text:
            continue
        bad.append(f"{wf.name}: runs a tools/asc script but never installs "
                   "cryptography, which client.py signs the JWT with")
    return bad


def offenders(reach: list[str]) -> tuple[list[str], list[tuple[str, str, bool]]]:
    rows, bad = [], []
    for wf in sorted(WORKFLOWS.glob("*.yml")):
        text = uncommented(wf.read_text(encoding="utf-8"))
        hit = next((tok for tok in reach if tok in text), None)
        if not hit:
            continue
        pinned = "actions/setup-python" in text
        rows.append((wf.name, hit, pinned))
        if not pinned:
            bad.append(wf.name)
    return bad, rows


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true", help="exit non-zero on a finding")
    ap.add_argument("--verbose", action="store_true", help="list every reaching token")
    args = ap.parse_args()

    reach = reaching_tokens()
    if not reach:
        print("payload-toolchain-lint: found nothing needing tomllib — "
              "that is not a pass, it is a broken scan", file=sys.stderr)
        return 1
    print(f"payload-toolchain-lint: {len(reach)} way(s) to reach a tomllib tool")
    if args.verbose:
        for tok in reach:
            print(f"  via {tok}")

    if not list(WORKFLOWS.glob("*.yml")):
        print("payload-toolchain-lint: no workflows found — not a pass", file=sys.stderr)
        return 1

    bad, rows = offenders(reach)
    for name, hit, pinned in rows:
        print(f"  {name}: reaches it via {hit!r} — "
              f"{'pins python' if pinned else 'DOES NOT PIN PYTHON'}")

    crypto_bad = cryptography_offenders()
    for row in crypto_bad:
        print(f"  {row}")
    if crypto_bad and args.check:
        print("::error::" + "; ".join(crypto_bad), file=sys.stderr)
        return 1

    wasm_bad = wasm_offenders()
    for row in wasm_bad:
        print(f"  {row}")
    if wasm_bad and args.check:
        print("::error::" + "; ".join(wasm_bad), file=sys.stderr)
        return 1

    if bad and args.check:
        print(f"::error::{', '.join(bad)} run tools that import tomllib without "
              "actions/setup-python. 3.11 is the floor and ubuntu-22.04 ships "
              "3.10.", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
