#!/usr/bin/env python3
"""Mutation sensitivity: perturb the engine, ask which tests notice.

The instrument this repository did not have. `tools/curiosity-answer-invariance.py`
asks whether a corpus row's answer moves across vessels; `tests/perturbation.rs`
asks whether a quantity moves when its stated cause moves. Both mutate *inputs*.
This mutates the *engine* and asks whether anything downstream complains — which
is the only one of the three that can tell you an assertion is aimed at code that
cannot break.

## Why a schema, and not a recompile per mutant

The textbook harness edits one line, rebuilds, runs the suite, and repeats. In
this workspace `kerotakis-core` is 71 000 lines and a cold rebuild of it plus the
CLI test binaries is minutes, so a few hundred mutants is a day of compute on a
four-core box that is shared. `cargo-mutants` is the obvious off-the-shelf tool
and it works exactly that way.

So this harness uses *mutant schemata* instead: every mutation site is rewritten
once into a call that consults `KERO_MUTANT` at run time and returns either the
original value or the perturbed one. One build, N runs. A mutant is then selected
by an environment variable, which the CLI integration tests inherit for free when
they spawn `kero`.

Three consequences worth stating, because two of them are advantages and the
third is a limit:

* **No mutant can be "killed" by a compile error.** Every mutant in the
  catalogue compiles by construction, so the score cannot be inflated by the
  type checker doing the test suite's job.
* **Cost collapses** from N builds to one build plus N test runs.
* **Only expressions can be schema'd.** A number inside a `const` table is
  evaluated at compile time and cannot consult an environment variable, so
  those sites are catalogued separately as `table` mutants and must be run the
  slow way, one rebuild each.

## Operators

`branch`   negate the condition of an `if`.
`constant` move an in-code f64 literal by +25 % (0.0 becomes 1.0, since
           0.0 x 1.25 is the same number and a mutant that changes nothing is
           not evidence about anything).
`table`    the same perturbation applied to a literal inside a `const`/`static`
           item — catalogued here, executed by rebuild rather than by schema.

Usage:
    mutate.py catalogue FILE...        list the sites, write catalogue.json
    mutate.py instrument               rewrite the surface, back up the originals
    mutate.py restore                  put the originals back
    mutate.py run [--ceiling SECONDS]  execute the catalogue, write results.json
    mutate.py report                   render results.json as markdown
    mutate.py recover                  undo a falsified table literal a
                                       killed run left in the working tree
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
STATE = Path(os.environ.get("KERO_MUTATION_STATE", REPO / ".mutation-state"))
CATALOGUE = STATE / "catalogue.json"
RESULTS = STATE / "results.json"
# The crash-safety marker. A `table` mutant falsifies a literal IN THE WORKING
# TREE and relies on a `finally` to put it back; a SIGKILL — which on this box
# arrives from the kernel's memory-pressure sweep, twice on 2026-09-16 — skips
# `finally` and leaves a polynomial coefficient 25 % wrong on disk, where the
# next commit picks it up. So the intent is written down BEFORE the edit and
# removed after the restore, and every later invocation of this script puts
# back anything the marker still describes. See `recover_live`.
LIVE = STATE / "table-live.json"

RUNTIME_MODULE = """
// --- injected by tools/mutation/mutate.py; `mutate.py restore` removes it ---
#[doc(hidden)]
#[allow(dead_code)]
pub mod __mutation {
    use std::sync::OnceLock;
    static ACTIVE: OnceLock<u32> = OnceLock::new();
    #[inline]
    fn selected() -> u32 {
        *ACTIVE.get_or_init(|| {
            std::env::var("KERO_MUTANT")
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(u32::MAX)
        })
    }
    /// Branch operator: the condition reads backwards when this mutant is live.
    #[inline]
    pub fn cond(id: u32, value: bool) -> bool {
        if selected() == id {
            !value
        } else {
            value
        }
    }
    /// Constant operator: the number is a quarter larger when this mutant is
    /// live. Zero becomes one, because zero times anything is still zero and a
    /// mutant that changes no value proves nothing about the tests.
    #[inline]
    pub fn num(id: u32, value: f64) -> f64 {
        if selected() == id {
            if value == 0.0 {
                1.0
            } else {
                value * 1.25
            }
        } else {
            value
        }
    }
}
"""

# --------------------------------------------------------------------------
# A Rust-aware-enough scanner.
#
# Not a parser. It masks comments, strings and char literals to spaces so that
# offsets are preserved, then works on the mask. Everything it cannot be sure
# about, it skips: the cost of a missed site is a smaller catalogue, and the
# cost of a wrong site is a build that does not compile.
# --------------------------------------------------------------------------


def mask(text: str) -> str:
    """Blank out comments, string and char literals, preserving offsets."""
    out = list(text)
    i, n = 0, len(text)

    def blank(a: int, b: int) -> None:
        for k in range(a, b):
            if out[k] != "\n":
                out[k] = " "

    while i < n:
        c = text[i]
        if c == "/" and i + 1 < n and text[i + 1] == "/":
            j = text.find("\n", i)
            j = n if j < 0 else j
            blank(i, j)
            i = j
        elif c == "/" and i + 1 < n and text[i + 1] == "*":
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth += 1
                    j += 2
                elif text.startswith("*/", j):
                    depth -= 1
                    j += 2
                else:
                    j += 1
            blank(i, j)
            i = j
        elif c == "r" and (m := re.match(r'r(#*)"', text[i:])):
            hashes = m.group(1)
            close = '"' + hashes
            j = text.find(close, i + m.end() - m.start())
            j = n if j < 0 else j + len(close)
            blank(i, j)
            i = j
        elif c == '"':
            j = i + 1
            while j < n:
                if text[j] == "\\":
                    j += 2
                    continue
                if text[j] == '"':
                    j += 1
                    break
                j += 1
            blank(i, j)
            i = j
        elif c == "'":
            # A char literal, or a lifetime. Only the former is masked.
            m = re.match(r"'(\\.|[^\\'])'", text[i:])
            if m:
                blank(i, i + m.end())
                i += m.end()
            else:
                i += 1
        else:
            i += 1
    return "".join(out)


def match_brace(masked: str, open_at: int) -> int:
    """Index just past the `}` matching the `{` at open_at."""
    depth, i, n = 0, open_at, len(masked)
    while i < n:
        if masked[i] == "{":
            depth += 1
        elif masked[i] == "}":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return n


def excluded_spans(masked: str) -> list[tuple[int, int]]:
    """Regions no mutation may touch: test modules, const fns, attributes."""
    spans: list[tuple[int, int]] = []
    for m in re.finditer(r"#\[cfg\(test\)\]", masked):
        brace = masked.find("{", m.end())
        if brace >= 0:
            spans.append((m.start(), match_brace(masked, brace)))
    for m in re.finditer(r"\bconst\s+fn\b", masked):
        brace = masked.find("{", m.end())
        if brace >= 0:
            spans.append((m.start(), match_brace(masked, brace)))
    for m in re.finditer(r"#!?\[", masked):
        depth, i, n = 0, m.end() - 1, len(masked)
        while i < n:
            if masked[i] == "[":
                depth += 1
            elif masked[i] == "]":
                depth -= 1
                if depth == 0:
                    spans.append((m.start(), i + 1))
                    break
            i += 1
    for m in re.finditer(r"\bmacro_rules!", masked):
        brace = masked.find("{", m.end())
        if brace >= 0:
            spans.append((m.start(), match_brace(masked, brace)))
    return spans


def const_spans(masked: str) -> list[tuple[int, int]]:
    """Regions evaluated at compile time: schemata cannot reach them."""
    spans = []
    for m in re.finditer(r"\b(?:const|static)\s+(?:mut\s+)?[A-Z_][A-Za-z0-9_]*\s*:", masked):
        depth, i, n = 0, m.end(), len(masked)
        while i < n:
            ch = masked[i]
            if ch in "([{":
                depth += 1
            elif ch in ")]}":
                depth -= 1
            elif ch == ";" and depth == 0:
                spans.append((m.start(), i + 1))
                break
            i += 1
    return spans


def inside(spans: list[tuple[int, int]], a: int, b: int) -> bool:
    return any(s <= a and b <= e for s, e in spans)


FLOAT = re.compile(
    r"(?<![A-Za-z0-9_.])"
    r"(\d[\d_]*\.\d[\d_]*(?:[eE][-+]?\d+)?|\d[\d_]*[eE][-+]?\d+)"
    r"(?![A-Za-z0-9_.])"
)


def line_of(text: str, pos: int) -> int:
    return text.count("\n", 0, pos) + 1


def source_line(text: str, pos: int) -> str:
    a = text.rfind("\n", 0, pos) + 1
    b = text.find("\n", pos)
    return " ".join(text[a : b if b > 0 else len(text)].split())[:90]


def collect(path: Path) -> list[dict]:
    text = path.read_text()
    m = mask(text)
    excl = excluded_spans(m)
    consts = const_spans(m)
    sites: list[dict] = []

    # --- branch operator: negate an `if` condition -------------------------
    for hit in re.finditer(r"\bif\b", m):
        start = hit.end()
        if re.match(r"\s+let\b", m[start:]):
            continue  # `if let` binds a pattern; there is no bool to flip
        if inside(excl, hit.start(), hit.end()) or inside(consts, hit.start(), hit.end()):
            continue
        depth, i, n, brace = 0, start, len(m), -1
        guard = False
        while i < n:
            ch = m[i]
            if ch in "([":
                depth += 1
            elif ch in ")]":
                depth -= 1
            elif depth == 0 and m.startswith("=>", i):
                guard = True  # a match guard: the next `{` is an arm body
                break
            elif ch == "{" and depth == 0:
                brace = i
                break
            elif ch == ";" and depth == 0:
                break
            i += 1
        if guard or brace < 0 or brace - start > 400:
            continue
        cond = text[start:brace].strip()
        if not cond:
            continue
        sites.append(
            {
                "kind": "branch",
                "file": str(path.relative_to(REPO)),
                "line": line_of(text, start),
                "start": start,
                "end": brace,
                "original": cond,
                "describes": " ".join(cond.split())[:90],
            }
        )

    # --- constant operator: move an f64 literal ----------------------------
    for hit in FLOAT.finditer(m):
        a, b = hit.start(), hit.end()
        if inside(excl, a, b):
            continue
        if m[max(0, a - 2) : a] == ".." or m[b : b + 2] == "..":
            continue  # a range bound may be a pattern, and patterns are const
        in_const = inside(consts, a, b)
        sites.append(
            {
                "kind": "table" if in_const else "constant",
                "file": str(path.relative_to(REPO)),
                "line": line_of(text, a),
                "start": a,
                "end": b,
                "original": text[a:b],
                "describes": source_line(text, a),
            }
        )

    sites.sort(key=lambda s: s["start"])
    return sites


# --------------------------------------------------------------------------
# instrument / restore
# --------------------------------------------------------------------------


def do_catalogue(files: list[str]) -> list[dict]:
    STATE.mkdir(parents=True, exist_ok=True)
    sites: list[dict] = []
    for f in files:
        sites.extend(collect(Path(f).resolve()))
    for n, s in enumerate(sites):
        s["id"] = n
    CATALOGUE.write_text(json.dumps(sites, indent=1))
    by_kind: dict[str, int] = {}
    for s in sites:
        by_kind[s["kind"]] = by_kind.get(s["kind"], 0) + 1
    print(f"{len(sites)} sites: " + ", ".join(f"{k}={v}" for k, v in sorted(by_kind.items())))
    return sites


def do_instrument(skip: set[int]) -> None:
    sites = json.loads(CATALOGUE.read_text())
    backups = STATE / "orig"
    backups.mkdir(parents=True, exist_ok=True)
    by_file: dict[str, list[dict]] = {}
    for s in sites:
        if s["kind"] == "table" or s["id"] in skip:
            continue
        by_file.setdefault(s["file"], []).append(s)

    for rel, group in by_file.items():
        path = REPO / rel
        bak = backups / rel.replace("/", "__")
        if not bak.exists():
            shutil.copy2(path, bak)
        text = bak.read_text()
        # Every edit is expressed as a pair of INSERTIONS rather than a
        # replacement, because the sites nest: a literal often sits inside the
        # `if` condition that is itself a site, and a replacement of the outer
        # span would be computed against offsets the inner edit had moved.
        inserts: list[tuple[int, int, str]] = []
        for s in group:
            if s["kind"] == "branch":
                inserts.append((s["start"], 0, f" crate::__mutation::cond({s['id']}, ("))
                inserts.append((s["end"], 1, ")) "))
            else:
                inserts.append((s["start"], 0, f"crate::__mutation::num({s['id']}, "))
                inserts.append((s["end"], 1, ")"))
        # Descending position; at equal positions an opening insert must come
        # before any other opening, and a closing insert after any other close.
        inserts.sort(key=lambda t: (t[0], t[1]), reverse=True)
        for pos, _, frag in inserts:
            text = text[:pos] + frag + text[pos:]
        path.write_text(text)

    lib = REPO / "crates/kerotakis-core/src/lib.rs"
    libbak = backups / "lib_rs_root"
    if not libbak.exists():
        shutil.copy2(lib, libbak)
    if "__mutation" not in lib.read_text():
        lib.write_text(libbak.read_text() + RUNTIME_MODULE)
    print(f"instrumented {len(by_file)} file(s), {sum(len(g) for g in by_file.values())} sites")


def do_restore() -> None:
    backups = STATE / "orig"
    if not backups.exists():
        print("nothing to restore")
        return
    for bak in backups.iterdir():
        if bak.name == "lib_rs_root":
            shutil.copy2(bak, REPO / "crates/kerotakis-core/src/lib.rs")
        else:
            shutil.copy2(bak, REPO / bak.name.replace("__", "/"))
    shutil.rmtree(backups)
    print("restored")


# --------------------------------------------------------------------------
# run
# --------------------------------------------------------------------------

# A tier is a test selection plus a judgement about what killing a mutant here
# is worth. `strength` is the deliberate part of the score: see docs.
TIERS = [
    {
        # Cheapest, and first, so that most mutants never pay for a CLI build.
        "name": "core-unit",
        "strength": "unit",
        "cmd": ["cargo", "test", "-p", "kerotakis-core", "--lib", "--", "-q"],
        "timeout": 1200,
    },
    {
        # `--lib` runs ONLY the tests inside src/. kerotakis-core also has 116
        # integration binaries under tests/, and the first run of this harness
        # silently excluded every one of them — including tests/buoyancy.rs,
        # which is the dedicated suite for a module this surface contains. The
        # selection below is by name against the surface, which is deliberately
        # GENEROUS: it gives the suite its best shot before anything is called
        # a survivor.
        "name": "core-integration",
        "strength": "integration",
        "cmd": [
            "cargo", "test", "-p", "kerotakis-core",
            "--test", "buoyancy", "--test", "float_or_sink", "--test", "density",
            "--test", "resistivity", "--test", "plastics", "--test", "surface",
            "--test", "instrument_oracle", "--test", "one_value",
            "--test", "heat_capacity_curves",
            # Added 2026-09-17 with the file itself: the λ° table's external
            # corroboration lives here, and a tier that does not run it cannot
            # see the 23 const-table survivors it was written for.
            "--test", "conductivity_sources",
        ],
        "timeout": 120,
    },
    {
        # The newest instruments in the tree (#613, and its older sibling).
        # These assert relations — a quantity must MOVE when its stated cause
        # moves — rather than values, which is the strongest evidence a test
        # can give that it is aimed at code that could break.
        "name": "cli-property",
        "strength": "property",
        "cmd": [
            "cargo", "test", "-p", "kerotakis-cli",
            "--test", "perturbation", "--test", "metamorphic",
        ],
        "timeout": 2400,
    },
    {
        # Last on purpose. The curiosity corpus grades answers against blessed
        # values, so it notices ANY digit that moves, including a correction.
        # Being last means a mutant labelled `golden` is one that NOTHING but a
        # blessed value noticed — which is the fact worth reporting.
        "name": "cli-corpus",
        "strength": "golden",
        "cmd": ["cargo", "test", "-p", "kerotakis-cli", "--test", "curiosity"],
        "timeout": 2400,
    },
]

FAILED_LINE = re.compile(r"^\s{4}(\S+)\s*$", re.M)


def failing_tests(output: str) -> list[str]:
    names = []
    for block in re.findall(r"\nfailures:\n(.*?)\n\n", output, re.S):
        names.extend(n for n in FAILED_LINE.findall(block) if not n.startswith("-"))
    return sorted(set(names))


def run_tier(tier: dict, mutant: int | None, env_extra: dict | None = None) -> dict:
    env = dict(os.environ)
    env["RUSTC_WRAPPER"] = ""
    env["TMPDIR"] = os.environ.get("TMPDIR", "/mnt/volume1/tmp-overflow/kero-build")
    if mutant is not None:
        env["KERO_MUTANT"] = str(mutant)
    env.update(env_extra or {})
    t0 = time.time()
    # A mutant can stop a loop converging, so every run is a process GROUP that
    # can be killed whole: `cargo test` spawns the test binary, which spawns
    # `kero`, and killing only the first would leave the last one running.
    proc = subprocess.Popen(
        tier["cmd"], cwd=REPO, env=env, stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT, text=True, start_new_session=True,
    )
    try:
        out, _ = proc.communicate(timeout=tier["timeout"])
        return {
            "tier": tier["name"],
            "ok": proc.returncode == 0,
            "timeout": False,
            "seconds": round(time.time() - t0, 1),
            "failing": failing_tests(out or ""),
        }
    except subprocess.TimeoutExpired:
        try:
            os.killpg(os.getpgid(proc.pid), 9)
        except ProcessLookupError:
            pass
        proc.communicate()
        return {
            "tier": tier["name"], "ok": False, "timeout": True,
            "seconds": round(time.time() - t0, 1), "failing": [],
        }


def do_run(ceiling: float, only: str | None, ids: list[int] | None) -> None:
    sites = json.loads(CATALOGUE.read_text())
    live = [s for s in sites if s["kind"] != "table"]
    if only:
        live = [s for s in live if s["kind"] == only]
    if ids:
        live = [s for s in live if s["id"] in ids]

    results = json.loads(RESULTS.read_text()) if RESULTS.exists() else {}
    started = time.time()

    if "baseline" not in results:
        base = [run_tier(t, None) for t in TIERS]
        results["baseline"] = base
    # A mutant that stops a solver converging must not cost twenty minutes.
    # The ceiling for each rung is eight times what that rung takes clean,
    # never less than a minute, which is generous enough that a slow machine
    # is not mistaken for a hang.
    for tier, b in zip(TIERS, results["baseline"]):
        tier["timeout"] = max(60.0, 8.0 * b["seconds"])
    RESULTS.write_text(json.dumps(results, indent=1))
    for b in results["baseline"]:
        print(f"baseline {b['tier']}: ok={b['ok']} {b['seconds']}s")
    if not all(b["ok"] for b in results["baseline"]):
        print("BASELINE IS RED — the instrumented tree must pass before any "
              "mutant means anything. Stopping.")
        return

    for s in live:
        key = str(s["id"])
        if key in results:
            continue
        if time.time() - started > ceiling:
            print(f"wall-clock ceiling {ceiling}s reached; {len(live)} planned, stopping")
            break
        record = {"site": s, "tiers": []}
        for tier in TIERS:
            r = run_tier(tier, s["id"])
            record["tiers"].append(r)
            if not r["ok"]:
                record["verdict"] = "timeout" if r["timeout"] else "caught"
                record["caught_by"] = tier["name"]
                record["strength"] = "hang" if r["timeout"] else tier["strength"]
                break
        else:
            record["verdict"] = "survived"
            record["strength"] = None
        results[key] = record
        RESULTS.write_text(json.dumps(results, indent=1))
        print(f"#{s['id']:>3} {s['kind']:<8} {s['file'].split('/')[-1]}:{s['line']:<5} "
              f"{record['verdict']:<9} {record.get('caught_by', '')}")


# --------------------------------------------------------------------------
# table mutants: the slow way, because a const cannot read an environment
# --------------------------------------------------------------------------

# The same selections as the rungs, compiled but not run: a table mutant has
# to be BUILT before it can be judged, and that build is what makes it slow.
BUILD = [
    [a for a in tier["cmd"] if a not in ("--", "-q")] + ["--no-run"]
    for tier in TIERS
]


def perturbed(literal: str) -> str:
    v = float(literal.replace("_", ""))
    return "1.0" if v == 0.0 else repr(v * 1.25)


def table_site_column(site: dict) -> tuple[Path, int, int, str]:
    """Where the literal sits, in line/column terms.

    Instrumentation only ever INSERTS text within a line, so line numbers are
    identical between the original and the instrumented file, and a `const`
    item is never instrumented at all — so its columns are unchanged too.
    """
    orig = STATE / "orig" / site["file"].replace("/", "__")
    if not orig.exists():
        # `run-table` needs a PRISTINE copy of the file only to turn a recorded
        # byte offset into a line and a column; it never reads it for content.
        # `instrument` is what normally writes one, and a table-only run never
        # instruments — so on a fresh checkout (CI, or a worktree where the
        # state directory was cleaned) there is nothing there. Recreate it from
        # the working tree, but ONLY when git agrees the file is unmodified:
        # seeding the reference from an already-falsified file would silently
        # rebase every offset on the mutation.
        dirty = subprocess.run(
            ["git", "status", "--porcelain", "--", site["file"]],
            cwd=REPO,
            capture_output=True,
            text=True,
        )
        if dirty.stdout.strip():
            raise SystemExit(
                f"{site['file']} has uncommitted changes and "
                f"{orig} does not exist. The pristine copy cannot be recreated "
                "from a modified file — commit or stash first, or restore "
                f"{STATE / 'orig'} from a clean tree."
            )
        orig.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(REPO / site["file"], orig)
        print(f"seeded {orig.name} from the clean working tree")
    text = orig.read_text()
    line_start = text.rfind("\n", 0, site["start"]) + 1
    return REPO / site["file"], site["line"], site["start"] - line_start, site["original"]


def mark_live(path: Path, line_no: int, original_line: str, mutant: int) -> None:
    """Record the falsified line before writing it, so a kill is recoverable."""
    LIVE.parent.mkdir(parents=True, exist_ok=True)
    LIVE.write_text(
        json.dumps(
            {
                "file": str(path.relative_to(REPO)),
                "line": line_no,
                "original": original_line,
                "mutant": mutant,
            }
        )
    )


def clear_live() -> None:
    LIVE.unlink(missing_ok=True)


def recover_live() -> bool:
    """Put back a table literal a killed run left falsified. Returns whether it did.

    Called at the top of EVERY subcommand, because the run that needs this is by
    definition the run that is no longer executing. Restores the one recorded
    line rather than the whole file from `orig`, so that edits made to the file
    since — a test added next to the constant, say — are not silently reverted
    along with the mutation.
    """
    if not LIVE.exists():
        return False
    mark = json.loads(LIVE.read_text())
    path = REPO / mark["file"]
    lines = path.read_text().splitlines(keepends=True)
    current = lines[mark["line"] - 1]
    if current == mark["original"]:
        print(f"note: {mark['file']}:{mark['line']} was already clean "
              f"(mutant #{mark['mutant']}); marker cleared")
    else:
        lines[mark["line"] - 1] = mark["original"]
        path.write_text("".join(lines))
        print(
            "RECOVERED a falsified constant a killed run left on disk:\n"
            f"  {mark['file']}:{mark['line']} (mutant #{mark['mutant']})\n"
            f"  was: {current.strip()}\n"
            f"  now: {mark['original'].strip()}\n"
            "  Run `git status` and `git diff` before committing anything."
        )
    clear_live()
    return True


def install_restore_signals() -> None:
    """Turn the catchable kill signals into a normal unwind.

    SIGTERM and SIGHUP otherwise bypass `finally` exactly as SIGKILL does, and
    those two ARE catchable: raising SystemExit from the handler runs the
    `finally` that restores the line. SIGKILL still cannot be caught, which is
    what `recover_live` is for.
    """
    import signal

    def bail(signum: int, _frame: object) -> None:
        raise SystemExit(f"killed by signal {signum}; restoring sources")

    for sig in (signal.SIGTERM, signal.SIGHUP, signal.SIGINT):
        signal.signal(sig, bail)


def do_run_table(ids: list[int], ceiling: float) -> None:
    sites = {s["id"]: s for s in json.loads(CATALOGUE.read_text())}
    results = json.loads(RESULTS.read_text()) if RESULTS.exists() else {}
    for tier, b in zip(TIERS, results.get("baseline", [])):
        tier["timeout"] = max(60.0, 8.0 * b["seconds"])
    started = time.time()
    for mid in ids:
        site = sites[mid]
        if site["kind"] != "table" or str(mid) in results:
            continue
        if time.time() - started > ceiling:
            print("ceiling reached")
            break
        path, line_no, col, literal = table_site_column(site)
        lines = path.read_text().splitlines(keepends=True)
        line = lines[line_no - 1]
        assert line[col : col + len(literal)] == literal, (
            f"#{mid}: expected {literal!r} at {path}:{line_no}:{col}, found "
            f"{line[col : col + len(literal)]!r}"
        )
        original_line = line
        lines[line_no - 1] = line[:col] + perturbed(literal) + line[col + len(literal) :]
        mark_live(path, line_no, original_line, mid)
        path.write_text("".join(lines))
        try:
            t0 = time.time()
            env = dict(os.environ)
            env["RUSTC_WRAPPER"] = ""
            built = all(
                subprocess.run(cmd, cwd=REPO, env=env, capture_output=True).returncode == 0
                for cmd in BUILD
            )
            build_s = round(time.time() - t0, 1)
            if not built:
                print(f"#{mid} build failed after {build_s}s — excluded")
                results[str(mid)] = {"site": site, "verdict": "uncompilable",
                                     "build_seconds": build_s}
                RESULTS.write_text(json.dumps(results, indent=1))
                continue
            record = {"site": site, "tiers": [], "build_seconds": build_s}
            for tier in TIERS:
                r = run_tier(tier, None)
                record["tiers"].append(r)
                if not r["ok"]:
                    record["verdict"] = "timeout" if r["timeout"] else "caught"
                    record["caught_by"] = tier["name"]
                    record["strength"] = "hang" if r["timeout"] else tier["strength"]
                    break
            else:
                record["verdict"] = "survived"
                record["strength"] = None
            results[str(mid)] = record
            RESULTS.write_text(json.dumps(results, indent=1))
            print(f"#{mid:>3} table    {site['file'].split('/')[-1]}:{site['line']:<5} "
                  f"{record['verdict']:<9} {record.get('caught_by', '')} "
                  f"(build {build_s}s)")
        finally:
            lines = path.read_text().splitlines(keepends=True)
            lines[line_no - 1] = original_line
            path.write_text("".join(lines))
            clear_live()


# --------------------------------------------------------------------------
# report
# --------------------------------------------------------------------------


def do_report() -> None:
    results = json.loads(RESULTS.read_text())
    rows = [(int(k), v) for k, v in results.items() if k != "baseline"]
    rows.sort()
    counts: dict[str, int] = {}
    for _, r in rows:
        key = r["strength"] or "survived"
        counts[key] = counts.get(key, 0) + 1
    total = len(rows)
    print(f"| outcome | mutants | share |")
    print(f"|---|---:|---:|")
    for k in ("property", "integration", "unit", "golden", "hang", "survived"):
        if k in counts:
            print(f"| {k} | {counts[k]} | {100 * counts[k] / total:.0f} % |")
    print(f"| **total** | **{total}** | |")
    print()
    print("## Survivors")
    print()
    print("| id | kind | site | what the mutant does |")
    print("|---|---|---|---|")
    for i, r in rows:
        if r["verdict"] == "survived":
            s = r["site"]
            print(f"| {i} | {s['kind']} | `{s['file'].split('/')[-1]}:{s['line']}` | "
                  f"`{s['describes'][:70]}` |")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    sub = ap.add_subparsers(dest="cmd", required=True)
    c = sub.add_parser("catalogue")
    c.add_argument("files", nargs="+")
    i = sub.add_parser("instrument")
    i.add_argument("--skip", default="", help="comma-separated ids to leave alone")
    sub.add_parser("restore")
    r = sub.add_parser("run")
    r.add_argument("--ceiling", type=float, default=3600.0)
    r.add_argument("--only", default=None)
    r.add_argument("--ids", default=None)
    rt = sub.add_parser("run-table")
    rt.add_argument("--ids", required=True)
    rt.add_argument("--ceiling", type=float, default=3600.0)
    sub.add_parser("report")
    sub.add_parser("recover")
    a = ap.parse_args()

    # Before anything else, and for every subcommand: a previous run may have
    # been killed with a constant falsified on disk.
    recovered = recover_live()
    install_restore_signals()

    if a.cmd == "recover":
        if not recovered:
            print("nothing to recover")
    elif a.cmd == "catalogue":
        do_catalogue(a.files)
    elif a.cmd == "instrument":
        do_instrument({int(x) for x in a.skip.split(",") if x.strip()})
    elif a.cmd == "restore":
        do_restore()
    elif a.cmd == "run":
        do_run(a.ceiling, a.only, [int(x) for x in a.ids.split(",")] if a.ids else None)
    elif a.cmd == "run-table":
        do_run_table([int(x) for x in a.ids.split(",")], a.ceiling)
    elif a.cmd == "report":
        do_report()


if __name__ == "__main__":
    main()
