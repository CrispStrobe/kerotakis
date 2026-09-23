#!/usr/bin/env python3
"""Every German row on the console page is actually looked up (I18N-14).

`web/index.html` is the one page in the repo that carries its own
translation dictionary — an inline `const DE = {…}` and a `tr()` beside
it — because it is a hand-written static page with no build step and no
bundler to reach `web/app/src/locales/`. Nothing in `tools/` read it, and
that is why this bug lived there:

    <p><a href="app/">→ the bench app</a> — same
      engine, drawn glassware</p>

Both halves of that sentence had German rows. Neither was ever reached.

  * `"→ the bench app"` sits inside an `<a>`, and the DOM sweep visits the
    DIRECT text children of the elements it selects. `<a>` was not one of
    them, so the anchor's text was never looked up.
  * `"— same engine, drawn glassware"` is the `<p>`'s own text node, and
    because the source wraps across two lines that node really reads
    `"— same\n      engine, drawn glassware"`. `tr()` is an exact-match
    lookup with a silent fallback, so it returned the English.

Two independent causes, one symptom, and fixing either alone still leaves
a German reader reading English.

So this lint asks the question that catches both, and it is the same
question `tools/engine-locale-lint.py` asks of the engine catalogue: **is
every translated string reachable?** A row nobody looks up is a
translation that was written and then thrown away.

    python3 tools/console-locale-lint.py
    python3 tools/console-locale-lint.py --check   # non-zero if a row is orphaned

Deliberately NOT the mirror question. "Every text node has a German row"
would fail on `PHREEQC`, `github.com/CrispStrobe/kerotakis`, `(USGS).`
and every command name a button carries — all correctly the same in both
languages — and a lint that cries wolf is one people learn to skip. The
unrooted-string half is reported for information and never fails.
"""

from __future__ import annotations

import html.parser
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
PAGE = ROOT / "web/index.html"

# The selectors the page's own DOM sweep uses, as (ancestor, parent)
# pairs. Kept here as DATA read out of the page itself below, so this
# lint cannot drift from the sweep it is checking.
SWEEP = re.compile(r'querySelectorAll\(\s*"([^"]+)"\s*\)')
# `const DE = { "key":"value", Bare:"value", … };`
ENTRY = re.compile(
    r'(?:"((?:[^"\\]|\\.)*)"|([A-Za-z_$][\w$]*))\s*:\s*"((?:[^"\\]|\\.)*)"'
)
# `tr("…")` — the lookups written out by hand, beside the ones the sweep
# performs.
TR_LITERAL = re.compile(r'\btr\(\s*"((?:[^"\\]|\\.)*)"')
# Does the sweep collapse the whitespace INSIDE a text node before it
# looks the sentence up? This lint has to model whichever lookup the page
# actually performs, or it grades the page it wishes were there: with an
# exact match, a <p> wrapped across two source lines carries the newline
# and its indentation into the key and reaches nothing.
NORMALISES = re.compile(r"replace\(\s*/\\s\+/g")


def unescape(text: str) -> str:
    """The handful of JS string escapes these two files actually use."""
    return (
        text.replace("\\n", "\n")
        .replace('\\"', '"')
        .replace("\\'", "'")
        .replace("\\\\", "\\")
    )


def normalise(text: str) -> str:
    """A sentence as the catalogue keys it: trimmed, inner runs of
    whitespace collapsed. This is the rule the page's sweep must follow,
    and the reason it must is the `<p>` wrapped across two source lines."""
    return " ".join(text.split())


def _is(node: tuple[str, str | None], selector: str) -> bool:
    """`p` matches a tag, `#side` matches an id. The two halves of a
    descendant selector are each one or the other."""
    tag, ident = node
    return ident == selector[1:] if selector.startswith("#") else tag == selector


class Sweep(html.parser.HTMLParser):
    """Which text nodes the page's own DOM sweep would visit.

    Only DIRECT text children count, because that is what
    `for (const node of el.childNodes)` means — the bug is that an
    element's text is invisible unless the element itself is selected.
    """

    def __init__(self, selectors: list[tuple[str, str]]) -> None:
        super().__init__(convert_charrefs=True)
        self.selectors = selectors
        self.stack: list[tuple[str, str | None]] = []
        self.visited: list[str] = []
        self.raw: list[str] = []
        self.skipped: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if tag in ("br", "hr", "img", "input", "meta", "link"):
            return
        ident = dict(attrs).get("id")
        self.stack.append((tag, ident))

    def handle_endtag(self, tag: str) -> None:
        for index in range(len(self.stack) - 1, -1, -1):
            if self.stack[index][0] == tag:
                del self.stack[index:]
                return

    def handle_data(self, data: str) -> None:
        if not self.stack or not data.strip():
            return
        if any(tag in ("script", "style") for tag, _ in self.stack):
            return
        matched = False
        for ancestor, child in self.selectors:
            if not _is(self.stack[-1], child):
                continue
            if any(_is(node, ancestor) for node in self.stack[:-1]):
                matched = True
                break
        if matched:
            self.visited.append(normalise(data))
            self.raw.append(data.strip())
        else:
            self.skipped.append(normalise(data))


def selectors_from(page: str) -> list[tuple[str, str]]:
    found = SWEEP.search(page)
    if not found:
        sys.exit("console-locale-lint: the page has no querySelectorAll sweep")
    out = []
    for part in found.group(1).split(","):
        words = part.split()
        if len(words) < 2:
            sys.exit(f"console-locale-lint: cannot read the selector {part.strip()!r}")
        # `#side p a` — only the outermost scope and the element holding
        # the text matter to the question this lint asks.
        out.append((words[0], words[-1]))
    return out


def bundle_bodies(page: str) -> dict[str, str]:
    """Each language's dictionary body, by code.

    The page carried ONE `const DE = {…}` and this lint read it by that
    name, so the lint was as German-only as the page was: the moment the
    console page learned a second language the lint stopped parsing at
    all. It reads `const BUNDLES = { de: {…}, fr: {…} }` now, and a
    language added there is checked without touching this file.
    """
    if "const BUNDLES = {" not in page:
        sys.exit(
            "console-locale-lint: no `const BUNDLES = {` in the page — it "
            "carried one `const DE` until I18N-10, and a caller still "
            "writing that shape is reading a page this lint cannot parse"
        )
    start = page.index("const BUNDLES = {")
    bodies: dict[str, str] = {}
    for found in re.finditer(r"^  ([a-z]{2}): \{$", page[start:], re.M):
        code = found.group(1)
        body_at = start + found.end()
        end = page.index("\n  },", body_at)
        bodies[code] = page[body_at:end]
    if not bodies:
        sys.exit("console-locale-lint: found `const BUNDLES` but no `<code>: {` inside it")
    return bodies


def analyse(page: str) -> tuple[set[str], set[str], list[str], list[str]]:
    """(translated rows, swept text, rows nobody reaches, swept-and-untranslated).

    The ROWS are the union across every language the page ships: a key one
    language carries and another does not is a gap in that language, which
    the coverage table below reports, and not an orphan — an orphan is a
    row NO language's page can ever reach.
    """
    body = "".join(bundle_bodies(page).values())
    rows = {
        # `findall` yields '' for a group that did not participate, not
        # None, so `or` is the test and `is not None` silently turned
        # every BARE key — `Catalysis:"Katalyse"` — into the empty string.
        unescape(quoted or bare)
        for quoted, bare, _ in ENTRY.findall(body)
    }

    sweep = Sweep(selectors_from(page))
    sweep.feed(page)
    swept = set(sweep.visited)
    # What the page would actually look up. `swept` stays normalised
    # because that is the sentence a human wrote and a translator keys on;
    # `reached` is what `tr()` would really be handed.
    reached = set(swept if NORMALISES.search(page) else sweep.raw)
    reached |= {normalise(unescape(m)) for m in TR_LITERAL.findall(page)}

    orphans = sorted(row for row in rows if normalise(row) not in reached)
    unrooted = sorted(swept - {normalise(r) for r in rows})
    return rows, swept, orphans, unrooted


def main() -> int:
    page = PAGE.read_text(encoding="utf-8")
    rows, swept, orphans, unrooted = analyse(page)

    print(f"console page {PAGE.relative_to(ROOT)}")
    print(f"   German rows            : {len(rows):>4}")
    print(f"   text nodes swept       : {len(swept):>4}")
    print(f"   rows never looked up   : {len(orphans):>4}")
    for row in orphans:
        print(f"      orphan: {row!r}")
    if unrooted:
        print(f"   swept, no translated row   : {len(unrooted):>4}   (reported, not a failure)")
        for value in unrooted:
            print(f"      untranslated: {value!r}")

    if "--check" in sys.argv and orphans:
        print(
            "\nA German row nobody looks up is a translation that was written "
            "and thrown away. Either the element holding it is outside the "
            "DOM sweep's selector, or the key does not match the text node "
            "(check for a sentence wrapped across two source lines).",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
