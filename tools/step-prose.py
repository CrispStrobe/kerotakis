#!/usr/bin/env python3
"""Validate and export the per-step prose that paces a catalogue run.

The step-by-step runner submits a script one line at a time and shows the
learner what that line produced. What it could not show was what to WATCH
for, because no entry carried a sentence per step: a codex entry has a
script and the guided catalogue has one phenomenon for a whole experiment.

The prose therefore lives here, as data, keyed by the codex entry whose
script it paces — never in a component, and never in the catalogue's own
source, so that the entry a lesson runs and the entry the codex lists get
the same sentences without either of them owning the other's file.

The alignment IS the schema. A sentence array must have exactly one entry
per runnable line of that script — comments and blanks are not steps — and
a misaligned array is refused rather than silently offset, because prose
that is one step out does not read as missing, it reads as WRONG: it
describes the fizz while the learner is still measuring out water.

One file per language, beside the English source, so adding French is one
new file and no change to any type. Missing prose is fine at any
granularity: an entry with no sentences runs exactly as it did before.
"""

import json
import pathlib
import re
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]

#: Age and childhood wording, in both languages, is never shown to a
#: learner (GUI-470). The bundles are gated by `learnerWording.test.ts`;
#: this prose never reaches a bundle, so it is gated here instead.
FORBIDDEN = re.compile(
    r"\b(kind|kinder|kindern|kindes|kids?|child|children|children's"
    r"|alter|altersgruppe|altersgruppen|jahre|jahren|jahrgang"
    r"|ages?|aged|years)\b",
    re.IGNORECASE,
)


def runnable_lines(script: str) -> list[str]:
    """The lines the runner will submit — the same rule `catalogRunner` uses."""
    return [
        line.strip()
        for line in script.split("\n")
        if line.strip() and not line.strip().startswith("#")
    ]


def codex_scripts(root: pathlib.Path = ROOT) -> dict[str, list[str]]:
    scripts: dict[str, list[str]] = {}
    for source in sorted((root / "codex").glob("*.toml")):
        for reaction in tomllib.loads(source.read_text()).get("reaction", []):
            script = (reaction.get("setup") or {}).get("script")
            if isinstance(script, str):
                scripts[reaction["id"]] = runnable_lines(script)
    return scripts


def check_rows(document: dict, scripts: dict[str, list[str]], where: str) -> dict[str, list[str]]:
    if document.get("schema") != 1:
        raise ValueError(f"{where}: step prose schema must be 1")
    rows = document.get("scripts")
    if not isinstance(rows, dict):
        raise ValueError(f"{where}: scripts must be an object keyed by codex id")
    if list(rows) != sorted(rows):
        raise ValueError(f"{where}: entries must be sorted by id")
    for entry_id, sentences in rows.items():
        lines = scripts.get(entry_id)
        if lines is None:
            raise ValueError(f"{where}: {entry_id} is not a codex entry")
        if not isinstance(sentences, list) or not all(isinstance(s, str) for s in sentences):
            raise ValueError(f"{where}: {entry_id} must be an array of strings")
        if len(sentences) != len(lines):
            raise ValueError(
                f"{where}: {entry_id} has {len(sentences)} sentences for "
                f"{len(lines)} runnable lines — prose must align with the script"
            )
        for index, sentence in enumerate(sentences):
            if not sentence.strip():
                raise ValueError(f"{where}: {entry_id}[{index}] is empty")
            offence = FORBIDDEN.search(sentence)
            if offence:
                raise ValueError(
                    f"{where}: {entry_id}[{index}] addresses a reader by age "
                    f"or as a child: {offence.group(0)!r}"
                )
    return rows


def add_translation(document: dict, translation: dict) -> dict:
    """Fold a complete locale file in as `say_<code>`, English unchanged."""
    locale = translation.get("locale")
    if translation.get("schema") != 1 or not isinstance(locale, str) or not locale:
        raise ValueError("step prose translation must use schema 1 and name a locale")
    english = document["scripts"]
    translated = translation.get("scripts")
    if not isinstance(translated, dict):
        raise ValueError("translated scripts must be an object")
    missing = sorted(set(english) - set(translated))
    if missing:
        raise ValueError(f"missing {locale} prose for: {', '.join(missing)}")
    extra = sorted(set(translated) - set(english))
    if extra:
        raise ValueError(f"{locale} prose for entries with no English: {', '.join(extra)}")
    merged: dict[str, dict[str, list[str]]] = {}
    for entry_id, sentences in english.items():
        other = translated[entry_id]
        if len(other) != len(sentences):
            raise ValueError(
                f"{entry_id}: {locale} has {len(other)} sentences against "
                f"{len(sentences)} in English"
            )
        merged[entry_id] = {"say": sentences, f"say_{locale}": other}
    document["scripts"] = merged
    return document


def build(source: pathlib.Path, root: pathlib.Path = ROOT) -> dict:
    scripts = codex_scripts(root)
    document = json.loads(source.read_text())
    check_rows(document, scripts, source.name)
    for translation in sorted(source.parent.glob("step-prose-*-v1.json")):
        other = json.loads(translation.read_text())
        check_rows(other, scripts, translation.name)
        add_translation(document, other)
    return document


def main() -> None:
    source = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "data/steps/step-prose-v1.json"
    document = build(source)
    if len(sys.argv) > 2:
        target = pathlib.Path(sys.argv[2])
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(json.dumps(document, indent=2, ensure_ascii=False) + "\n")
    lines = sum(len(row["say"]) for row in document["scripts"].values())
    print(f"   {len(document['scripts'])} paced scripts, {lines} step sentences validated")


if __name__ == "__main__":
    main()
