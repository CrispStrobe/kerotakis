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
