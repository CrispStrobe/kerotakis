"""Labelled lesson prose: the one parser both the payload and the lint read.

A lesson's title, description, section comments and boundary note are the
`.lab` file's own `#` comments. They were rendered verbatim, so a German
learner met a German lesson name above six English lines (I18N-9).

The translatable unit is the **paragraph**, not the line and not the
fragment. That is the lesson I18N-7 paid for: `"there is "` + list +
`" in the beaker"` cannot be translated piecewise because the pieces are
grammar, not words. A `.lab` wraps its prose at 78 columns for the
terminal, and those line breaks are typography; German rebuilds the
sentence with the verb somewhere else entirely.

So a unit is opened by a LABELLED comment and continued by the plain
comment lines under it:

    #@intro Citric acid and baking soda can sit beside each other while dry.
    # Water lets ions move and starts the acid-carbonate reaction.

is one unit, `<stem>.intro`, whose English is those two lines joined with a
space. A bare `#`, a blank line, or a command closes it.

The label names the PLACE the prose is said — the same discipline
`i18n/de.toml` uses — so rewording the English does not orphan the German.
And it is still a `#` comment, so `kero run lessons/x.lab` is unchanged and
a `.lab` stays runnable on its own.

Unlabelled comments are left exactly as they were: they parse to nothing
here, render verbatim as before, and are counted by
`tools/lesson-prose-lint.py` as the work outstanding. A half-migrated
library that reports itself accurately is the intended state.
"""

from __future__ import annotations

import pathlib
import re
import tomllib
from dataclasses import dataclass

# `#@label text`. The label is lowercase, dotted or hyphenated — `title`,
# `intro`, `part.displacement`, `boundary`. It must be followed by text on
# the same line: a label with no sentence is a key with nothing to say.
LABELLED = re.compile(r"^#@([a-z0-9][a-z0-9._-]*)[ \t]+(\S.*?)[ \t]*$")
# A plain comment line carrying text — a continuation when a unit is open.
CONTINUATION = re.compile(r"^#[ \t]+(\S.*?)[ \t]*$")
# `#@` with nothing usable after it: a typo worth naming rather than
# silently treating as prose.
MALFORMED = re.compile(r"^#@(?![a-z0-9][a-z0-9._-]*[ \t]+\S)")


@dataclass(frozen=True)
class Unit:
    """One labelled paragraph of a lesson."""

    label: str
    text: str
    line: int

    def key(self, stem: str) -> str:
        return f"{stem}.{self.label}"


def units(text: str) -> list[Unit]:
    """Every labelled paragraph in a `.lab`, in file order."""
    found: list[Unit] = []
    label: str | None = None
    parts: list[str] = []
    start = 0

    def close() -> None:
        nonlocal label, parts
        if label is not None:
            found.append(Unit(label, " ".join(parts), start))
        label, parts = None, []

    for number, raw in enumerate(text.splitlines(), start=1):
        # Trimmed at both ends, because the player trims: the two parsers
        # have to agree about which lines a label owns, or the payload's
        # keys describe a paragraph the screen never shows.
        line = raw.strip()
        if (match := LABELLED.match(line)) is not None:
            close()
            label, parts, start = match.group(1), [match.group(2)], number
            continue
        if label is not None and (match := CONTINUATION.match(line)) is not None:
            parts.append(match.group(1))
            continue
        close()
    close()
    return found


def malformed(text: str) -> list[tuple[int, str]]:
    """Lines that tried to be a label and are not one."""
    return [
        (number, line.strip())
        for number, line in enumerate(text.splitlines(), start=1)
        if MALFORMED.match(line.strip())
    ]


def unlabelled_comment_lines(text: str) -> int:
    """Comment lines still rendered verbatim — the work outstanding.

    Continuation lines of a labelled unit do not count: they are already
    translated, as part of their paragraph. A bare `#` does not count
    either; it is a spacer, not prose.
    """
    remaining = 0
    inside = False
    for raw in text.splitlines():
        line = raw.strip()
        if LABELLED.match(line):
            inside = True
            continue
        if CONTINUATION.match(line):
            if not inside:
                remaining += 1
            continue
        inside = False
    return remaining


def lesson_units(directory: pathlib.Path) -> dict[str, str]:
    """Every key a `.lab` in `directory` references, to its English.

    This is the lint's DENOMINATOR, and it is read from the lessons —
    never from `en.toml`'s own key count. `models.toml` reported 100%
    German over 325 English strings in #505 because its denominator was
    the list of fields somebody remembered to write down.
    """
    out: dict[str, str] = {}
    for path in sorted(directory.glob("*.lab")):
        for unit in units(path.read_text(encoding="utf8")):
            out[unit.key(path.stem)] = unit.text
    return out


def flatten(document: dict, prefix: str = "") -> dict[str, str]:
    """`[dry-then-wet-fizz] part.displacement = "…"` to one dotted key.

    A prose file is one table per lesson, so a translator sees a lesson at
    a time rather than a flat wall of dotted keys; a dotted LABEL nests one
    level further, which is TOML doing what the label already meant.
    """
    out: dict[str, str] = {}
    for name, value in document.items():
        key = f"{prefix}{name}"
        if isinstance(value, str):
            out[key] = value
        elif isinstance(value, dict):
            out.update(flatten(value, f"{key}."))
    return out


def catalogue(directory: pathlib.Path) -> dict[str, dict[str, str]]:
    """Locale code to its flattened prose, from `lessons/prose/*.toml`.

    Discovered by filename, which is the whole point: adding French is
    `lessons/prose/fr.toml` and no code, exactly as adding French to the
    interface is `src/locales/fr.json` and no code.
    """
    out: dict[str, dict[str, str]] = {}
    if not directory.is_dir():
        return out
    for path in sorted(directory.glob("*.toml")):
        if path.stem.startswith("_"):
            continue
        with path.open("rb") as handle:
            out[path.stem] = flatten(tomllib.load(handle))
    return out


def render(rows: dict[str, str], header: str) -> str:
    """A prose file: one table per lesson, keys in the lesson's own order."""
    lines = [header.rstrip("\n"), ""]
    stem = None
    for key, value in rows.items():
        this_stem, label = key.split(".", 1)
        if this_stem != stem:
            if stem is not None:
                lines.append("")
            stem = this_stem
            lines.append(f'["{stem}"]')
        lines.append(f"{label} = {toml_string(value)}")
    return "\n".join(lines).rstrip("\n") + "\n"


def toml_string(value: str) -> str:
    """A TOML basic string. Prose carries quotes and the odd backslash."""
    escaped = value.replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'
