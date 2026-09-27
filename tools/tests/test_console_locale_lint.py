"""The two ways the console page lost a translation it already had.

`web/index.html` carries its own inline dictionary because it is a
hand-written static page with no build step. Both German rows for

    <p><a href="app/">→ the bench app</a> — same
      engine, drawn glassware</p>

existed and neither was ever looked up: the anchor was outside the DOM
sweep's selector, and the paragraph's own text node carries the source
file's line break, so an exact-match lookup missed it. Fixing one still
leaves a German reader reading English, which is why both are pinned.

The third case is the one that keeps the lint usable: `PHREEQC`, a URL
and a command name are swept, have no German row, and must never fail.
"""

import importlib.util
import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "console_locale_lint", ROOT / "tools/console-locale-lint.py"
)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


def page(sweep: str, body: str, normalising: bool = True) -> str:
    """A console page in miniature.

    `normalising` is the fix to the second bug, written the way the real
    page writes it, because the lint FEATURE-DETECTS it: a lint that
    assumed the page normalised would grade the page it wished were there.

    The dictionary is written as `const BUNDLES = { de: {…} }` for the same
    reason: I18N-10 generalised the page from one hardcoded `const DE` to a
    table keyed by language, and this miniature has to keep the shape the
    lint parses. It did not, and six tests here went from testing the lint
    to raising `ValueError: substring not found` — a fixture that has
    drifted from the thing it stands in for tests nothing.
    """
    collapse = 'value.replace(/\\s+/g, " ")' if normalising else "value"
    return f"""<body>
<header><small>a bench</small></header>
<aside id="side">
{body}
</aside>
<script type="module">
const BUNDLES = {{
  de: {{
  "a bench":"ein Labor", "→ the bench app":"→ zur Labor-App",
  "— same engine, drawn glassware":"— dieselbe Engine, gezeichnete Glasgeräte",
  Source:"Quellcode"
  }},
}};
const tr = (m) => (BUNDLES[locale]?.[m]) || m;
for (const el of document.querySelectorAll("{sweep}")) {{ set(tr({collapse})); }}
</script>
</body>"""


WRAPPED = """  <p><a href="app/">→ the bench app</a> — same
    engine, drawn glassware</p>
  <h2>Source</h2>"""

UNWRAPPED = """  <p><a href="app/">→ the bench app</a> — same engine, drawn glassware</p>
  <h2>Source</h2>"""

NARROW = "header small, #side h2, #side p"
WIDE = "header small, #side h2, #side p, #side p a"


def orphans_of(*args, **kwargs) -> list[str]:
    return MODULE.analyse(page(*args, **kwargs))[2]


class ConsoleLocaleLint(unittest.TestCase):
    def test_an_anchor_outside_the_selector_orphans_its_row(self):
        """The `<a>` holds the text, so selecting the `<p>` is no help."""
        self.assertEqual(orphans_of(NARROW, UNWRAPPED), ["→ the bench app"])

    def test_an_exact_match_orphans_a_sentence_the_source_wrapped(self):
        """The text node carries the newline and the indentation."""
        self.assertEqual(
            orphans_of(WIDE, WRAPPED, normalising=False),
            ["— same engine, drawn glassware"],
        )

    def test_either_cause_alone_is_enough_to_lose_the_sentence(self):
        """Which is why the real page needed both fixes, not one."""
        self.assertEqual(
            sorted(orphans_of(NARROW, WRAPPED, normalising=False)),
            ["— same engine, drawn glassware", "→ the bench app"],
        )
        self.assertEqual(orphans_of(WIDE, WRAPPED, normalising=False),
                         ["— same engine, drawn glassware"])
        self.assertEqual(orphans_of(NARROW, WRAPPED), ["→ the bench app"])

    def test_both_fixes_together_reach_every_row(self):
        self.assertEqual(orphans_of(WIDE, WRAPPED), [])
        self.assertEqual(orphans_of(WIDE, UNWRAPPED), [])

    def test_a_bare_javascript_key_is_a_key(self):
        """`Source:"Quellcode"` has no quotes on the left-hand side, and
        reading only the quoted form turned every one of them into the
        empty string — which then reported ten live rows as orphans and
        ten translated headings as untranslated."""
        rows = MODULE.analyse(page(WIDE, UNWRAPPED))[0]
        self.assertIn("Source", rows)
        self.assertNotIn("", rows)

    def test_an_untranslated_neutral_string_is_reported_not_failed(self):
        """A product name and a command name are swept, have no German
        row, and must never fail: a lint that cries wolf is one people
        learn to skip."""
        body = UNWRAPPED + "\n  <h2>PHREEQC</h2>\n  <h2>inspect v1</h2>"
        _, _, orphans, unrooted = MODULE.analyse(page(WIDE, body))
        self.assertEqual(orphans, [], "nothing here is an orphaned ROW")
        self.assertEqual(sorted(unrooted), ["PHREEQC", "inspect v1"])


if __name__ == "__main__":
    unittest.main()
