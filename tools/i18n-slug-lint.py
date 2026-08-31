#!/usr/bin/env python3
"""I18N-2 coverage: the map screen's own vocabulary.

Neither a concept nor an experiment has a label field. `ConceptMap.svelte`
and `ExperimentCatalog.svelte` take the IDENTIFIER, replace its hyphens
with spaces and hand the result to the interface dictionary:

    tSlug("activation-energy")  ->  t("activation energy")  ->  "Aktivierungsenergie"

So the German for a concept node is not in the catalogue and not in the
engine: it is a `messages` entry in `web/app/src/locales/<code>.json`,
keyed by the de-slugged English identifier. Nothing connects the two ends,
which is why this lint exists — `t()` falls back to its argument, so a
concept nobody translated renders `heterogeneous catalysis` inside a German
map and fails nothing. Three concepts and one experiment name had done
exactly that since `rates.toml` gained an entry.

The key set is derived here rather than listed, from the same fields the
components read, so adding a codex entry moves the requirement.

    python3 tools/i18n-slug-lint.py           # report
    python3 tools/i18n-slug-lint.py --check   # non-zero if German has a hole

German alone is fatal, for the reason `localeBundles.test.ts` gives: a
half-translated language is the intended state — every string falls back
on its own — and German is the one that is complete, so it is the honest
floor. A new language is reported, never failed.
"""

from __future__ import annotations

import json
import pathlib
import re
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent
CODEX = ROOT / "codex"
LOCALES = ROOT / "web/app/src/locales"
SRC = ROOT / "web/app/src/lib"

# The language that is complete, and therefore the one a hole is a bug in.
COMPLETE = "de"

# Reaction fields whose slugs reach a screen through tSlug()/t(). `id` is
# the experiment name in the map list and the catalogue heading; the rest
# are the chips beside it. `calculations` is deliberately absent: it is
# parsed into CodexEntry and rendered nowhere, and a key nobody asks for
# is as much a bug as a key nobody translated.
RENDERED_FIELDS = ("concepts", "requires", "apparatus", "models")


def deslug(slug: str) -> str:
    """The transformation `tSlug` performs, and the only reason this lint
    can predict a key at all."""
    return slug.replace("-", " ")


def check_derivation() -> list[str]:
    """Fail loudly if the shell stops de-slugging the way this lint assumes.

    The whole key set below is a prediction about someone else's code. If
    `tSlug` learns to strip a suffix or title-case a word, every key here
    silently becomes wrong and the lint goes green over an English map —
    the exact failure it is here to prevent. So the assumption is pinned
    to the source that implements it.
    """
    problems = []
    tslug = (SRC / "i18n.svelte.ts").read_text(encoding="utf8")
    if 'slug.replace(/-/g, " ")' not in tslug:
        problems.append(
            "i18n.svelte.ts: tSlug no longer reads `slug.replace(/-/g, \" \")`; "
            "the keys this lint derives are guesses until it is updated"
        )
    map_src = (SRC / "components/ConceptMap.svelte").read_text(encoding="utf8")
    if 'replace(/-/g, " ")' not in map_src:
        problems.append(
            "ConceptMap.svelte: no longer de-slugs its node labels; "
            "check what it asks the dictionary for now"
        )
    return problems


def wanted() -> dict[str, dict[str, str]]:
    """Every key the map and the catalogue ask for, by family.

    Concepts and requires are one family: `conceptGraph` puts both on the
    map as nodes, and the "needs:" line under a locked experiment renders
    a `requires` slug with the same call.
    """
    experiments: dict[str, str] = {}
    concepts: dict[str, str] = {}
    apparatus: dict[str, str] = {}
    models: dict[str, str] = {}
    families = {
        "concepts": concepts,
        "requires": concepts,
        "apparatus": apparatus,
        "models": models,
    }
    for f in sorted(CODEX.glob("*.toml")):
        doc = tomllib.load(open(f, "rb"))
        for entry in doc.get("reaction", []):
            experiments[deslug(entry["id"])] = entry["id"]
            for field in RENDERED_FIELDS:
                for slug in entry.get(field, []):
                    families[field][deslug(slug)] = slug
    return {
        "experiment names": experiments,
        "concepts": concepts,
        "apparatus": apparatus,
        "models": models,
    }


def main() -> int:
    check = "--check" in sys.argv
    problems = check_derivation()
    for line in problems:
        print(f"DERIVATION: {line}")

    families = wanted()
    total = sum(len(f) for f in families.values())
    print(f"codex slugs the shell de-slugs: {total}")

    bundles = sorted(p for p in LOCALES.glob("*.json") if not p.name.startswith("_"))
    if not bundles:
        print("no locale bundles in web/app/src/locales/")
        return 1 if check else 0

    for path in bundles:
        bundle = json.loads(path.read_text(encoding="utf8"))
        code = bundle.get("@@locale", path.stem)
        # `terms` then `messages`, the order i18n.svelte.ts flattens them in.
        have = {**bundle.get("terms", {}), **bundle.get("messages", {})}
        print(f"\n== {code}")
        print(f"{'family':<18} {code:>7} {'slugs':>7}   coverage")
        missing_here: list[str] = []
        for name, keys in families.items():
            gaps = sorted(k for k in keys if not have.get(k, "").strip())
            missing_here += gaps
            n, all_n = len(keys) - len(gaps), len(keys)
            pct = 100 * n / all_n if all_n else 100.0
            print(f"{name:<18} {n:>7} {all_n:>7}   {pct:5.1f}% {'#' * int(pct / 5)}")
            for key in gaps:
                print(f"   missing: {key!r}  (from {keys[key]!r})")
        # A value equal to its key is a translation that was never made:
        # `t()` would have returned exactly that on its own.
        echoes = sorted(
            k for keys in families.values() for k in keys if have.get(k) == k
        )
        for key in echoes:
            print(f"   ECHO: {key!r} is translated as itself")
        if code == COMPLETE and (missing_here or echoes):
            problems.append(f"{code}: {len(missing_here) + len(echoes)} slug(s) unanswered")

    if problems:
        print(f"\n{len(problems)} problem(s)")
    return 1 if (problems and check) else 0


if __name__ == "__main__":
    raise SystemExit(main())
