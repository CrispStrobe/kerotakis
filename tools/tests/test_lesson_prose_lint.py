"""The lesson-prose lint, and the denominator that makes it worth having."""

import importlib.util
import pathlib
import subprocess
import sys
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))

import lesson_prose  # noqa: E402

SPEC = importlib.util.spec_from_file_location("lesson_prose_lint", ROOT / "tools/lesson-prose-lint.py")
LINT = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(LINT)


class ParserTests(unittest.TestCase):
    def test_a_label_owns_the_comment_lines_under_it(self):
        units = lesson_prose.units(
            "#@intro Citric acid and baking soda can sit beside each other\n"
            "# while dry. Water lets ions move.\n"
            "add v1 water 1mL\n"
        )
        self.assertEqual(len(units), 1)
        self.assertEqual(units[0].label, "intro")
        self.assertEqual(
            units[0].text,
            "Citric acid and baking soda can sit beside each other while dry. "
            "Water lets ions move.",
        )

    def test_a_bare_hash_a_blank_line_and_a_command_all_close_a_paragraph(self):
        for closer in ("#", "", "add v1 water 1mL"):
            units = lesson_prose.units(f"#@a One\n{closer}\n# loose\n")
            self.assertEqual([u.text for u in units], ["One"], closer)

    def test_an_unmigrated_lesson_has_no_units_and_counts_as_outstanding(self):
        text = "# One\n# Two\n#\nadd v1 water 1mL\n"
        self.assertEqual(lesson_prose.units(text), [])
        self.assertEqual(lesson_prose.unlabelled_comment_lines(text), 2)

    def test_a_labelled_paragraph_is_not_outstanding_work(self):
        text = "#@intro One\n# Two\n# Three\n"
        self.assertEqual(lesson_prose.unlabelled_comment_lines(text), 0)

    def test_a_malformed_label_is_named_rather_than_read_as_prose(self):
        self.assertEqual(
            lesson_prose.malformed("#@Title Nope\n#@ nor this\n#@ok fine\n"),
            [(1, "#@Title Nope"), (2, "#@ nor this")],
        )

    def test_a_dotted_label_round_trips_through_toml(self):
        rows = {"x.part.displacement": "Part 1", "x.title": "T"}
        rendered = lesson_prose.render(rows, "# header")
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "en.toml"
            path.write_text(rendered, encoding="utf8")
            self.assertEqual(lesson_prose.catalogue(path.parent)["en"], rows)

    def test_prose_files_are_discovered_by_filename(self):
        """Adding French is `fr.toml` and no code. This is that claim."""
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "fr.toml").write_text('["x"]\ntitle = "Titre"\n', encoding="utf8")
            (root / "_template.toml").write_text('["x"]\ntitle = ""\n', encoding="utf8")
            found = lesson_prose.catalogue(root)
            self.assertEqual(found, {"fr": {"x.title": "Titre"}})


class DenominatorTests(unittest.TestCase):
    """#505's scar: a coverage number is only as honest as its denominator."""

    def test_the_denominator_is_read_from_the_lessons_not_from_the_catalogue(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "a.lab").write_text("#@title A\n#@boundary B\nadd v1 water 1mL\n")
            keys, problems = LINT.referenced(root)
            self.assertEqual(problems, [])
            self.assertEqual(keys, {"a.title": "A", "a.boundary": "B"})

    def test_two_sentences_under_one_label_is_fatal(self):
        """The reader gets the WRONG sentence — not a missing one, which no
        missing-key count would ever show."""
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "a.lab").write_text("#@part One\nadd v1 water 1mL\n#@part Two\n")
            keys, problems = LINT.referenced(root)
            self.assertEqual(keys, {"a.part": "One"})
            self.assertEqual(len(problems), 1)
            self.assertIn("used twice", problems[0])

    def test_an_unmigrated_lesson_is_counted_and_named(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "done.lab").write_text("#@title A\n")
            (root / "waiting.lab").write_text("# A\n# B\n")
            migrated, waiting, lines = LINT.migration(root)
            self.assertEqual(migrated, ["done"])
            self.assertEqual(waiting, ["waiting"])
            self.assertEqual(lines, 2)


class ShippedTreeTests(unittest.TestCase):
    def test_the_shipped_lessons_pass_their_own_lint(self):
        result = subprocess.run(
            [sys.executable, "tools/lesson-prose-lint.py", "--check"],
            cwd=ROOT, capture_output=True, text=True,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_en_toml_is_exactly_what_the_lessons_say(self):
        """`en.toml` is generated: a stale row means German was authored
        against wording that no longer exists."""
        keys, _ = LINT.referenced()
        self.assertEqual(lesson_prose.catalogue(LINT.PROSE)["en"], keys)


if __name__ == "__main__":
    unittest.main()
