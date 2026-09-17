#!/usr/bin/env python3
"""I18N-9 coverage: the prose a lesson says, in the reader's language.

A lesson's title, description, section comments and boundary note are the
`.lab` file's own `#` comments, and they were rendered verbatim. Only the
lesson's SLUG was translated, which is why a German learner met

    Lektion begonnen: Trocken, dann nass: Brausen
    Dry, then wet: why sherbet and bath-bomb powders wait for water
    Citric acid and baking soda can sit beside each other while dry…

a German title above six English lines. `t()` did not catch it because the
sentences were never keys: they arrive from a data file the dictionary has
never seen.

    python3 tools/lesson-prose-lint.py           # report
    python3 tools/lesson-prose-lint.py --check   # non-zero on a hole
    python3 tools/lesson-prose-lint.py --write   # regenerate en.toml

## The denominator

**Every label a `.lab` REFERENCES**, read from `lessons/*.lab` — never the
key count of `lessons/prose/en.toml`. That distinction is the whole lint.
#505 is the scar: `models.toml` reported 100% German while 325 of 409
strings were English, because the denominator was a list of fields somebody
had remembered to write down, and a field left off the list was invisible
rather than missing.

So a lesson nobody has migrated is not 100% translated here — it is a
lesson with no labels, counted and named under "not yet migrated". A
half-migrated library that reports itself accurately is the intended
state; one that claims completeness is the failure this file exists to
prevent.

## What is fatal

- a `#@` line that is not a well-formed label (a typo renders as prose);
- the same label twice in one lesson (two sentences, one key: the reader
  gets the WRONG sentence, which no missing-key count would ever show);
- `en.toml` disagreeing with the `.lab` it mirrors — the `.lab` is the one
  source of truth for a lesson (GUI-020), so `en.toml` is generated from
  it and a stale file means the German was authored against wording that
  no longer exists;
- a row in any locale file that no `.lab` asks for (an orphan: the prose
  moved or the label was renamed, and the translation is now dead weight
  that reads as coverage);
- a key the COMPLETE language has not translated. German is the language
  that is finished, so a hole in it is a bug rather than the ordinary
  half-translated state every other language is allowed to be in — the
  same floor `tools/i18n-slug-lint.py` holds.
"""

from __future__ import annotations

import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

import lesson_prose  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parent.parent
LESSONS = ROOT / "lessons"
PROSE = LESSONS / "prose"
PLAYER = ROOT / "web/app/src/lib/lesson.ts"

# The language that is complete, and therefore the one a hole is a bug in.
COMPLETE = "de"

EN_HEADER = """# Lesson prose, English — GENERATED from the `.lab` files.
#
# Do not edit by hand: `python3 tools/lesson-prose-lint.py --write`
# regenerates it, and `--check` fails if it drifts. The `.lab` file is the
# one source of truth for a lesson (GUI-020) and carries this English
# inline, where `kero run` and an untranslated payload both read it.
#
# This file exists for two readers: a translator, who needs the key list
# and the source text without grepping 113 lessons, and the lint, which
# uses it to notice that a sentence was reworded after its German was
# written. A translation lives in `<code>.toml` beside this file — adding
# French is `fr.toml` and no code anywhere."""


def check_player() -> list[str]:
    """Fail loudly if the player stops parsing labels the way we assume.

    Every key below is a prediction about someone else's code: the payload
    is keyed by what `tools/lesson_prose.py` finds, and the screen is
    rendered by what `web/app/src/lib/lesson.ts` finds. If those two
    disagree about which lines a label owns, this lint goes green over a
    German row nothing can look up — the exact failure it is here to
    prevent. So the assumption is pinned to the source that implements it.
    """
    problems = []
    if not PLAYER.exists():
        return ["web/app/src/lib/lesson.ts is missing; the player's parser cannot be checked"]
    source = PLAYER.read_text(encoding="utf8")
    for pattern, name in (
        (r"/^#@([a-z0-9][a-z0-9._-]*)[ \t]+(\S.*?)[ \t]*$/", "LABELLED"),
        (r"/^#[ \t]+(\S.*?)[ \t]*$/", "CONTINUATION"),
    ):
        if pattern not in source:
            problems.append(
                f"lesson.ts: the player's {name} pattern is no longer "
                f"`{pattern}`; the payload's keys are guesses until this lint "
                f"and the player agree again"
            )
    return problems


def referenced(directory: pathlib.Path = LESSONS) -> tuple[dict[str, str], list[str]]:
    """The denominator: every key a `.lab` references, to its English."""
    keys: dict[str, str] = {}
    problems: list[str] = []
    for path in sorted(directory.glob("*.lab")):
        text = path.read_text(encoding="utf8")
        for line, content in lesson_prose.malformed(text):
            problems.append(f"{path.name}:{line}: not a label — {content!r}")
        seen: dict[str, str] = {}
        for unit in lesson_prose.units(text):
            key = unit.key(path.stem)
            if unit.label in seen:
                problems.append(
                    f"{path.name}:{unit.line}: label `{unit.label}` is used twice; "
                    f"one key cannot hold two sentences "
                    f"({seen[unit.label]!r} and {unit.text!r})"
                )
                continue
            seen[unit.label] = unit.text
            keys[key] = unit.text
    return keys, problems


def migration(directory: pathlib.Path = LESSONS) -> tuple[list[str], list[str], int]:
    """Which lessons carry labels, which do not, and what is left."""
    migrated: list[str] = []
    waiting: list[str] = []
    lines = 0
    for path in sorted(directory.glob("*.lab")):
        text = path.read_text(encoding="utf8")
        (migrated if lesson_prose.units(text) else waiting).append(path.stem)
        lines += lesson_prose.unlabelled_comment_lines(text)
    return migrated, waiting, lines


def main() -> int:
    check = "--check" in sys.argv
    write = "--write" in sys.argv
    verbose = "--verbose" in sys.argv

    keys, problems = referenced()
    problems += check_player()
    tables = lesson_prose.catalogue(PROSE)
    english = lesson_prose.render(keys, EN_HEADER)

    if write:
        PROSE.mkdir(parents=True, exist_ok=True)
        (PROSE / "en.toml").write_text(english, encoding="utf8")
        print(f"wrote {PROSE / 'en.toml'}: {len(keys)} rows")
        return 0

    source = tables.get("en")
    if source is None:
        problems.append("lessons/prose/en.toml is missing; run --write")
    elif source != keys:
        stale = sorted(k for k in keys if source.get(k) != keys[k])
        gone = sorted(k for k in source if k not in keys)
        for key in stale[:10]:
            problems.append(
                f"en.toml: `{key}` no longer matches the lesson's own words "
                f"(the .lab says {keys[key]!r}); rerun --write and check the "
                f"translations of it"
            )
        for key in gone[:10]:
            problems.append(f"en.toml: `{key}` is orphaned; no .lab asks for it")
        if len(stale) + len(gone) > 20:
            problems.append(f"en.toml: … and {len(stale) + len(gone) - 20} more rows adrift")

    migrated, waiting, remaining_lines = migration()
    total = len(migrated) + len(waiting)
    print(f"lesson prose — {len(keys)} labels referenced by {len(migrated)} of {total} lessons")
    print(f"   {remaining_lines} comment lines in {len(waiting)} lessons are still rendered verbatim")

    fatal = list(problems)
    if COMPLETE not in tables:
        # An absent file is not an empty denominator. Without this the lint
        # reports "every referenced label is authored" over a language that
        # does not exist — #505's mistake, said a different way.
        fatal.append(
            f"lessons/prose/{COMPLETE}.toml is missing, and {len(keys)} labels "
            f"are referenced; the complete language cannot be absent"
        )
    for code, rows in sorted(tables.items()):
        if code == "en":
            continue
        missing = sorted(k for k in keys if k not in rows)
        orphans = sorted(k for k in rows if k not in keys)
        copied = sorted(k for k, v in rows.items() if keys.get(k) == v)
        done = len(keys) - len(missing)
        share = 100 * done // len(keys) if keys else 100
        print(f"   {code}: {done}/{len(keys)} labels translated ({share}%)")
        for key in orphans:
            fatal.append(f"{code}.toml: `{key}` is orphaned; no .lab asks for it")
        if copied:
            print(f"      {len(copied)} rows are identical to the English: {copied[:3]}")
        if missing:
            where = sorted({k.split('.', 1)[0] for k in missing})
            line = f"      {len(missing)} untranslated, in {len(where)} lessons"
            if code == COMPLETE:
                fatal.append(
                    f"{code}.toml: {len(missing)} labels have no German — "
                    f"{missing[:5]}{' …' if len(missing) > 5 else ''}. "
                    f"German is the complete language; a lesson whose German "
                    f"is not written should not carry labels yet."
                )
            else:
                print(line + " (reported, not failed: only "
                      f"{COMPLETE} is held complete)")
        if verbose and missing:
            for key in missing:
                print(f"      missing: {key}")

    if verbose:
        for stem in waiting:
            print(f"   not yet migrated: {stem}")

    for line in fatal:
        print(f"   {line}")
    if fatal and check:
        print(f"\nlesson prose: {len(fatal)} problem(s)")
        return 1
    if not fatal:
        print("lesson prose: every referenced label is authored in " + COMPLETE)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
