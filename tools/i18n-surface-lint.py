#!/usr/bin/env python3
"""Filter and sort the strings you render, not the ones underneath them.

Four surfaces displayed German and operated on English. The worst was the
experiment filter: typing "Säure" returned "nothing matches" while the
German word sat on screen. An empty result list looks like an answer, so
nothing about it reads as broken — which is why it survived a full
translation pass, a browser gate and a review.

The invariant: if a field reaches the screen through a translator, then a
comparator or predicate over that same field must go through one too.

Two rules, both read the balanced call expression rather than one line,
because the locale argument is usually on the next line:

  1. `localeCompare` takes a locale. Without one it uses the runtime's
     default, so German umlauts sort after z rather than beside their base
     letters.
  2. A `.sort()` or `.filter()` touching a rendered field mentions a
     translator inside the same expression.

Both rules cover the DISPLAY layer (`.svelte`) only. The data layer sorts
deterministically on purpose: `conceptGraph` in codex.ts orders the nodes
the concept map lays out, and a map whose nodes move when the reader
switches language is worse than one ordered by an invisible slug. Canonical
order stays canonical; translate when you draw it.

Neither rule can know a value test from a display test — `phase` is a wire
key in one place and a visible chip in another. Mark a deliberate value
test with `i18n-ok:` and a reason inside the expression.
"""
import pathlib
import re
import sys

# `--check` is accepted so this reads like its three siblings in
# preflight.sh; there is no other mode, since it only ever reports.
_ = "--check" in sys.argv

ROOT = pathlib.Path(__file__).resolve().parent.parent
SRC = ROOT / "web/app/src"

# Fields the catalogue and shelf render through t()/tSlug()/tEngine().
RENDERED = ("\\.id\\b", "\\.summary\\b", "\\.equation\\b", "\\.phase\\b", "\\.concepts\\b")
TRANSLATORS = ("t(", "tSlug(", "tEngine(", "i18n.locale")


def call_at(text, open_paren):
    """The source of the call whose '(' is at open_paren, parens balanced."""
    depth, i = 0, open_paren
    while i < len(text):
        if text[i] == "(":
            depth += 1
        elif text[i] == ")":
            depth -= 1
            if depth == 0:
                return text[open_paren : i + 1]
        i += 1
    return text[open_paren:]


def line_of(text, index):
    return text.count("\n", 0, index) + 1


problems = []
for path in sorted(SRC.rglob("*.svelte")):
    text = path.read_text()
    rel = path.relative_to(ROOT)

    for m in re.finditer(r"\.localeCompare\(", text):
        call = call_at(text, m.end() - 1)
        # One argument means no locale: "a".localeCompare(b)
        if "," not in call and "i18n.locale" not in call:
            problems.append(f"{rel}:{line_of(text, m.start())}: localeCompare without a locale")

    for m in re.finditer(r"\.(sort|filter)\(", text):
        call = call_at(text, m.end() - 1)
        # A reason is usually written on the line above the call, not
        # inside the parens, so look back a little as well.
        before = text[max(0, m.start() - 400) : m.start()]
        if "i18n-ok:" in call or "i18n-ok:" in before.rsplit("\n\n", 1)[-1]:
            continue
        if not any(re.search(f, call) for f in RENDERED):
            continue
        if any(tr in call for tr in TRANSLATORS):
            continue
        problems.append(
            f"{rel}:{line_of(text, m.start())}: .{m.group(1)}() over a rendered field "
            f"without a translator"
        )

for p in problems:
    print(p)
print(f"\n{len(problems)} surface(s) operating on untranslated text")
sys.exit(1 if problems else 0)
