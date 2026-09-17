import importlib.util
import pathlib
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("lessons_index", ROOT / "tools/lessons-index.py")
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


class LessonsIndexTests(unittest.TestCase):
    def test_kit_is_deduplicated_sorted_and_matches_player_verbs(self):
        text = """# Demo
add v1 water 1mL
titrate v1 HCl NaOH 1mL
grind v1 NaCl 1g
add v1 water 2mL
measure v1 ph
"""
        self.assertEqual(MODULE.lesson_kit(text), ["HCl", "NaCl", "water"])

    def test_index_emits_kit_derived_from_lesson(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "demo.lab"
            path.write_text("# A demo\nadd v1 CuSO4 1g\nadd v1 water 10mL\n")
            self.assertEqual(MODULE.index(path.parent)[0]["kit"], ["CuSO4", "water"])

    def test_corrosion_lessons_have_a_visible_topic(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "rusting.lab").write_text("# Rust controls\nadd v1 Fe 1g\n")
            (root / "copper-patina.lab").write_text("# Copper controls\nadd v1 CuO 1g\n")
            indexed = {entry["file"]: entry for entry in MODULE.index(root)}
            self.assertEqual(indexed["rusting.lab"]["topic"], "corrosion & materials")
            self.assertEqual(indexed["copper-patina.lab"]["topic"], "corrosion & materials")

    def test_every_shipped_lesson_has_a_shelf(self):
        """No lesson falls into "more".

        The bucket held 47 of 113 lessons, which was the picker and the
        catalogue disagreeing about one curriculum. The shelves are derived
        from the catalogue's own authored topics now, so a lesson landing in
        "more" means its catalogue row carries no topic this list knows —
        a real gap, and this says which lesson.
        """
        indexed = MODULE.index(ROOT / "lessons")
        homeless = sorted(e["file"] for e in indexed if e["topic"] == "more")
        self.assertEqual(homeless, [], f"unshelved lessons: {homeless}")

    def test_a_lesson_with_no_catalogue_row_keeps_the_more_shelf(self):
        """The fallback still exists, and payload builds still work alone."""
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "not-a-shipped-lesson.lab").write_text("# Demo\nadd v1 water 1mL\n")
            self.assertEqual(MODULE.index(root)[0]["topic"], "more")

    def test_the_catalogue_decides_where_a_lesson_without_a_curated_slot_goes(self):
        """Ordered, and the order is the editorial judgement.

        `heating-proteins` carries food, proteins and heat. `proteins`
        outranks `heat`, so it is food rather than thermochemistry; and
        `invisible-ink-boundary` carries heat and food with no protein, so
        the same list sends it the other way.
        """
        authored = MODULE.catalogue_topics()
        self.assertEqual(
            MODULE.topic_for("heating-proteins", {}, authored), "food & life"
        )
        self.assertEqual(
            MODULE.topic_for("invisible-ink-boundary", {}, authored), "heat & fire"
        )

    def test_crystal_collection_separates_outcome_from_boundary(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "rock-candy.lab"
            path.write_text("# Rock candy\nadd v1 water 10mL\n")
            entry = MODULE.index(path.parent)[0]
            self.assertEqual(entry["topic"], "crystals & solubility")
            self.assertEqual(entry["collection"], "crystal lab")
            self.assertIn("computed", entry["outcome_note"])
            self.assertIn("no crystal", entry["boundary_note"])

    def test_liquid_nitrogen_lesson_is_visible_and_loans_both_materials(self):
        lesson = ROOT / "lessons" / "liquid-nitrogen-freezes-ethanol.lab"
        entry = MODULE.index(lesson.parent)
        entry = next(item for item in entry if item["file"] == lesson.name)
        self.assertEqual(entry["topic"], "heat & fire")
        self.assertEqual(entry["kit"], ["ethanol", "liquid_nitrogen"])

    def test_staged_salt_transfer_is_an_intermediate_separation_mission(self):
        lesson = ROOT / "lessons" / "follow-the-salt-through-staged-transfers.lab"
        entry = next(item for item in MODULE.index(lesson.parent)
                     if item["file"] == lesson.name)
        self.assertEqual(entry["topic"], "separations")
        self.assertEqual(entry["progress"], "intermediate")
        self.assertEqual(entry["kit"], ["NaCl", "water"])

    def test_dual_axis_carbon_dioxide_lesson_is_an_intermediate_acid_base_mission(self):
        lesson = ROOT / "lessons" / "two-limits-one-carbon-dioxide-ledger.lab"
        entry = next(item for item in MODULE.index(lesson.parent)
                     if item["file"] == lesson.name)
        self.assertEqual(entry["topic"], "acids & bases")
        self.assertEqual(entry["progress"], "intermediate")
        self.assertEqual(entry["kit"], ["HCl", "NaHCO3", "water"])

    def test_wait_boundary_lesson_is_an_intermediate_rates_mission(self):
        lesson = ROOT / "lessons" / "longer-waits-advance-peroxide-kinetics.lab"
        entry = next(item for item in MODULE.index(lesson.parent)
                     if item["file"] == lesson.name)
        self.assertEqual(entry["topic"], "rates")
        self.assertEqual(entry["progress"], "intermediate")
        self.assertEqual(entry["kit"], ["H2O2", "water"])


if __name__ == "__main__":
    unittest.main()


class LessonProseTests(unittest.TestCase):
    """The picker's blurb is lesson prose too (I18N-9)."""

    def test_a_labelled_first_comment_carries_its_key_beside_the_english(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "demo.lab"
            path.write_text("#@title A demo\nadd v1 water 1mL\n")
            entry = MODULE.index(path.parent)[0]
            self.assertEqual(entry["blurb"], "A demo")
            self.assertEqual(entry["blurb_key"], "demo.title")

    def test_an_unlabelled_lesson_ships_the_blurb_it_always_shipped(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "demo.lab"
            path.write_text("# A demo\nadd v1 water 1mL\n")
            entry = MODULE.index(path.parent)[0]
            self.assertEqual(entry["blurb"], "A demo")
            self.assertNotIn("blurb_key", entry)

    def test_a_wrapped_title_is_one_blurb_rather_than_its_first_line(self):
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "demo.lab"
            path.write_text("#@title A demo that\n# wraps\nadd v1 water 1mL\n")
            self.assertEqual(MODULE.index(path.parent)[0]["blurb"], "A demo that wraps")

    def test_english_is_not_shipped_twice(self):
        """The `.lab` carries the English inline and the player falls back
        to it, so `en.toml` in the payload would be one more copy to drift."""
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            (root / "prose").mkdir()
            (root / "prose/en.toml").write_text('["demo"]\ntitle = "A demo"\n')
            (root / "prose/de.toml").write_text('["demo"]\ntitle = "Eine Demo"\n')
            self.assertEqual(MODULE.prose(root), {"de": {"demo.title": "Eine Demo"}})

    def test_every_shipped_translation_names_a_label_a_lesson_asks_for(self):
        """The payload and the lint agree about the key set, or a German
        row is shipped that nothing can look up."""
        shipped = MODULE.prose(ROOT / "lessons")
        referenced = set(MODULE.lesson_prose.lesson_units(ROOT / "lessons"))
        for code, rows in shipped.items():
            self.assertEqual(set(rows) - referenced, set(), code)

