#!/usr/bin/env python3
"""A prompt that distinguishes N things must produce N different answers.

The curiosity corpus classifies a prompt by the events its script emits, so
a script that runs, emits events and earns a disposition looks answered.
It is not necessarily answered. `mat-012` asked "How can density
distinguish copper, zinc, and aluminium pieces?" and weighed five grams of
each on a balance: three vessels, three identical readings, and a row that
matched its own `expected` and appeared in no mismatch list for as long as
the corpus existed.

That is the one class of corpus defect that never shows up as a failure — a
PASSING row whose script cannot reach its own question — and it needs no
vocabulary to detect. If a script fills two or more vessels with different
things, and the bench says the same thing about all of them, the script
cannot answer any question that distinguishes them.

Two rules make it precise:

  * Setup echoes are not answers. `v1: +0.0787 mol copper` differs from
    `v2: +0.0765 mol zinc` and tells you only what you already typed. What
    counts is what the bench says BACK.
  * A refusal repeated is not this defect. Two vessels that both reach
    `not yet modelled` are an engine gap, already counted as `missing`;
    the script is not at fault and fixing it would change nothing.

Validated against the known instance: mat-012's pre-fix script scores
3 vessels -> 1 answer, and its fixed script scores 3 -> 3.

Usage: tools/curiosity-answer-invariance.py [--kero PATH]
Exits non-zero if any prompt is invariant over its own subjects.
"""
from __future__ import annotations
import argparse, collections, pathlib, re, subprocess, sys, tempfile, tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent
CORPUS = ROOT / "tests/coverage/curiosity-v1"

VESSEL = re.compile(r"\bv(\d+)\b")
# What you put in, and what shape the glassware is, are not answers.
SETUP = re.compile(r":\s*\+|new vessel|pressure controlled|sealed at|swept at")
# An honest refusal is an engine gap, not a script that misses its question.
REFUSAL = re.compile(r"not yet modelled|isn't awake yet|cannot say")


def load_prompts() -> list[dict]:
    prompts = []
    for shard in sorted(CORPUS.glob("*.toml")):
        if shard.name in ("baseline.toml", "manifest.toml"):
            continue
        doc = tomllib.loads(shard.read_text(encoding="utf-8"))
        prompts.extend(doc.get("prompt", []))
    return prompts


def vessel_contents(script: list[str]) -> dict[str, frozenset[str]]:
    per: dict[str, set[str]] = collections.defaultdict(set)
    for line in script:
        match = re.match(r"\s*add\s+(v\d+)\s+(\S+)", line)
        if match:
            per[match.group(1)].add(match.group(2))
    return {vessel: frozenset(items) for vessel, items in per.items()}


def answers(kero: str, script: list[str]) -> dict[str, tuple[str, ...]] | None:
    with tempfile.NamedTemporaryFile("w", suffix=".lab", delete=False) as handle:
        handle.write("register lv2\n" + "\n".join(script) + "\n")
        path = handle.name
    try:
        out = subprocess.run(
            [kero, "run", path], capture_output=True, text=True, timeout=180
        ).stdout
    except subprocess.TimeoutExpired:
        return None
    finally:
        pathlib.Path(path).unlink(missing_ok=True)
    grouped: dict[str, list[str]] = collections.defaultdict(list)
    for line in out.splitlines():
        match = VESSEL.search(line)
        if not match or SETUP.search(line):
            continue
        grouped["v" + match.group(1)].append(VESSEL.sub("vN", line).strip())
    return {vessel: tuple(lines) for vessel, lines in grouped.items()}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--kero", default=str(ROOT / "target/debug/kero"))
    args = parser.parse_args()
    if not pathlib.Path(args.kero).exists():
        print(f"no kero binary at {args.kero}; pass --kero PATH", file=sys.stderr)
        return 2

    problems, checked = [], 0
    for prompt in load_prompts():
        if prompt.get("expected") == "boundary" or prompt.get("parse_boundary"):
            continue
        script = prompt.get("script", [])
        contents = vessel_contents(script)
        if len(contents) < 2 or len(set(contents.values())) < 2:
            continue  # not a comparison: nothing claims to be distinguished
        checked += 1
        said = answers(args.kero, script)
        if said is None:
            continue
        said = {vessel: lines for vessel, lines in said.items() if vessel in contents}
        if len(said) < 2:
            continue
        if all(all(REFUSAL.search(line) for line in lines) for lines in said.values()):
            continue  # every vessel refused: an engine gap, not a script gap
        if len(set(said.values())) < len(said):
            problems.append((prompt, said))

    for prompt, said in problems:
        print(f"{prompt['id']}: {len(said)} vessels, "
              f"{len(set(said.values()))} distinct answers")
        print(f"  question: {prompt['question']}")
        print(f"  script:   {prompt['script']}")
        for vessel, lines in sorted(said.items()):
            print(f"    {vessel}: {list(lines)}")
    print(f"\n{checked} comparison prompts run; {len(problems)} cannot tell "
          f"their own subjects apart")
    return 1 if problems else 0


if __name__ == "__main__":
    raise SystemExit(main())
