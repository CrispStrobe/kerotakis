#!/usr/bin/env python3
"""Validate and fold in the curiosity corpus's translations.

The capability explorer had translated CHROME and English CONTENT: its
buttons, filters and level words came from `web/app/src/locales/de.json`,
while the five hundred questions it exists to show, and the material
classes and concept tags under them, were the corpus's authored English.
A German reader got a German dialog listing English questions.

The corpus cannot carry the German itself. It is a test artefact — the
baseline joins to it by `id` and `kero coverage curiosity --check` gates
the join — and `CuriosityPrompt` is `deny_unknown_fields`, so a
`question_de` in a shard is a build failure, not a translation. So the
translations live BESIDE the corpus, one file per language, in
`tests/coverage/curiosity-v1/i18n/<locale>.toml`, keyed by the id the
shard already owns. Adding French is `fr.toml` and nothing else: no field
in a shard, no branch in a component, no existing translation touched.

Three key spaces, because the explorer shows three kinds of English:

  [question]        keyed by prompt id, one row per prompt.
  [material_class]  keyed by the English token, so a class is translated
  [tag]             once and reads the same under every question using it.

What this module refuses, and why each refusal is worth a build:

  * a MISSING id — a half-translated corpus is worse than an English one,
    because the reader cannot tell a gap from an answer nobody has;
  * a STALE key — a translation for an id or token the corpus no longer
    has is a translation nobody will ever see, and it hides the rename
    that orphaned it;
  * a TASK IDENTIFIER among the tags — `CAP-5` and `BRD-041` are names,
    not words, and a translated identifier stops joining to its task;
  * an ECHO — German byte-identical to the English question, which is how
    an untranslated row survives a review that only counts rows;
  * AGE or CHILDHOOD wording (GUI-470). The corpus bands prompts by school
    age and the learner never sees that: the band renders through the
    level words the explorer maps, so no translation may say Kind, Alter
    or Jahre. `learnerWording.test.ts` gates the shell's own bundles; this
    prose never reaches one, so it is gated here.

Absence, at the file level, stays free: a language with no file simply
does not appear, and every field falls back to English on its own.
"""

from __future__ import annotations

import pathlib
import re
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]
CORPUS = ROOT / "tests/coverage/curiosity-v1"

#: A task name — `CAP-5`, `EXP-17`, `BRD-041`. These appear among the tags
#: and are identifiers: the explorer labels them through `t()` like any
#: other chip, but their TEXT joins a prompt to the task that owns it, and
#: a translated identifier joins to nothing.
IDENTIFIER = re.compile(r"^[A-Z][A-Z0-9]{1,7}-\d+$")

#: Age and childhood wording, in both languages, is never shown to a
#: learner (GUI-470). Same list as `tools/step-prose.py` — deliberately,
#: because the two files are read by the same reader on the same screen.
FORBIDDEN = re.compile(
    r"\b(kind|kinder|kindern|kindes|kinderlabor|kids?|child|children|children's"
    r"|alter|altersgruppe|altersgruppen|altersband|jahre|jahren|jahrgang"
    r"|ages?|aged|years)\b",
    re.IGNORECASE,
)


class Corpus:
    """What the shipped corpus actually says, in English."""

    def __init__(self, questions: dict[str, str], material_classes: set[str], tags: set[str]):
        self.questions = questions
        self.material_classes = material_classes
        #: Only the words. Identifiers are not translatable content.
        self.tags = {tag for tag in tags if not IDENTIFIER.match(tag)}
        self.identifier_tags = {tag for tag in tags if IDENTIFIER.match(tag)}


def read_corpus(directory: pathlib.Path = CORPUS) -> Corpus:
    manifest = tomllib.loads((directory / "manifest.toml").read_text())
    questions: dict[str, str] = {}
    material_classes: set[str] = set()
    tags: set[str] = set()
    for shard in manifest["shards"]:
        for prompt in tomllib.loads((directory / shard).read_text())["prompt"]:
            questions[prompt["id"]] = prompt["question"]
            material_classes.add(prompt["material_class"])
            tags.update(prompt.get("tags", []))
    return Corpus(questions, material_classes, tags)


def _check_table(
    rows: object,
    expected: set[str],
    corpus_english: dict[str, str] | None,
    where: str,
    table: str,
) -> dict[str, str]:
    if not isinstance(rows, dict):
        raise ValueError(f"{where}: [{table}] must be a table")
    bad = sorted(k for k, v in rows.items() if not isinstance(v, str) or not v.strip())
    if bad:
        raise ValueError(f"{where}: [{table}] is blank for: {', '.join(bad)}")
    missing = sorted(expected - set(rows))
    if missing:
        raise ValueError(
            f"{where}: [{table}] is missing {len(missing)} of {len(expected)} "
            f"— the corpus has no translation for: {', '.join(missing[:8])}"
            + (" …" if len(missing) > 8 else "")
        )
    stale = sorted(set(rows) - expected)
    if stale:
        raise ValueError(
            f"{where}: [{table}] translates keys the corpus does not have "
            f"(renamed or removed?): {', '.join(stale[:8])}"
            + (" …" if len(stale) > 8 else "")
        )
    if list(rows) != sorted(rows):
        raise ValueError(f"{where}: [{table}] must be sorted by key")
    for key, value in rows.items():
        offence = FORBIDDEN.search(value)
        if offence:
            raise ValueError(
                f"{where}: [{table}].{key} addresses a reader by age or as a "
                f"child: {offence.group(0)!r} — the band renders as a level"
            )
        if corpus_english is not None and value == corpus_english[key]:
            raise ValueError(
                f"{where}: [{table}].{key} is byte-identical to the English "
                f"— an untranslated row, not a translation"
            )
    return dict(rows)


def check(document: dict, corpus: Corpus, where: str) -> dict:
    """Refuse anything a reader would experience as a hole or a lie."""
    if document.get("schema") != 1:
        raise ValueError(f"{where}: curiosity prose schema must be 1")
    locale = document.get("locale")
    if not isinstance(locale, str) or not locale or locale == "en":
        raise ValueError(f"{where}: must name a locale, and English is the source")
    if locale != pathlib.Path(where).stem:
        raise ValueError(f"{where}: locale {locale!r} does not match the filename")
    known = {"schema", "locale", "question", "material_class", "tag"}
    unknown = sorted(set(document) - known)
    if unknown:
        raise ValueError(f"{where}: unknown section(s): {', '.join(unknown)}")
    translated_identifiers = sorted(
        set(document.get("tag") or {}) & corpus.identifier_tags
    )
    if translated_identifiers:
        raise ValueError(
            f"{where}: [tag] translates task identifiers, which are names "
            f"rather than words: {', '.join(translated_identifiers)}"
        )
    return {
        "locale": locale,
        "question": _check_table(
            document.get("question"), set(corpus.questions), corpus.questions, where, "question"
        ),
        # The glossaries are single words. "pH" and "Aluminium" translate to
        # themselves, so the echo check that catches an untranslated question
        # would only ever be a false alarm here.
        "material_class": _check_table(
            document.get("material_class"), corpus.material_classes, None, where, "material_class"
        ),
        "tag": _check_table(document.get("tag"), corpus.tags, None, where, "tag"),
    }


def read_translations(directory: pathlib.Path = CORPUS) -> list[dict]:
    """Every language shipped beside the corpus, discovered by filename."""
    found = []
    corpus = read_corpus(directory)
    for source in sorted((directory / "i18n").glob("*.toml")):
        found.append(check(tomllib.loads(source.read_text()), corpus, source.name))
    return found


def apply(prompts: list[dict], translations: list[dict]) -> list[dict]:
    """Fold each language in as `<field>_<locale>` siblings.

    Sibling keys rather than a nested per-locale map, matching what the
    engine's records already do (`tEngine` in the shell reads exactly this
    shape): a field degrades one at a time, so a language can ship the
    questions before the glossaries and the untranslated half simply falls
    back to English instead of blocking the translated half.
    """
    for translation in translations:
        code = translation["locale"]
        for prompt in prompts:
            question = translation["question"].get(prompt["id"])
            if question:
                prompt[f"question_{code}"] = question
            material_class = translation["material_class"].get(prompt["material_class"])
            if material_class:
                prompt[f"material_class_{code}"] = material_class
            if prompt.get("tags"):
                # Positional twin of `tags`, so the two lists index together.
                # An identifier has no translation and stays itself.
                prompt[f"tags_{code}"] = [
                    translation["tag"].get(tag, tag) for tag in prompt["tags"]
                ]
    return prompts


def main() -> None:
    directory = CORPUS
    args = [a for a in sys.argv[1:] if a != "--check"]
    if args:
        directory = pathlib.Path(args[0])
    corpus = read_corpus(directory)
    translations = read_translations(directory)
    if not translations:
        print("   no curiosity translations shipped — the corpus reads in English")
        return
    for translation in translations:
        print(
            f"   {translation['locale']}: {len(translation['question'])} questions, "
            f"{len(translation['material_class'])} material classes, "
            f"{len(translation['tag'])} tags validated "
            f"({len(corpus.identifier_tags)} task identifiers left as names)"
        )


if __name__ == "__main__":
    try:
        main()
    except ValueError as error:
        raise SystemExit(f"curiosity prose: {error}")
