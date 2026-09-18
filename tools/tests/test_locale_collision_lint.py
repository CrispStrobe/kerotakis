import importlib.util
import json
import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "locale_collision_lint", ROOT / "tools/locale-collision-lint.py"
)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


class LocaleCollisionLintTests(unittest.TestCase):
    def test_a_collision_is_a_key_in_both_sections(self):
        doc = {
            "terms": {"shared": "A", "term-only": "T"},
            "messages": {"shared": "A", "message-only": "M"},
        }
        self.assertEqual(MODULE.collisions(doc), {"shared": ("A", "A")})

    def test_agreeing_sides_are_reported_but_not_a_problem(self):
        """57 of the 59 agree. The merge picks one of two identical strings,
        which is harmless, and a lint that cried wolf on it would be one
        people learn to ignore."""
        doc = {"terms": {"k": "gleich"}, "messages": {"k": "gleich"}}
        term, message = MODULE.collisions(doc)["k"]
        self.assertEqual(term, message)

    def test_disagreeing_sides_are_what_the_lint_is_for(self):
        doc = {"terms": {"k": "Kalkwasser"}, "messages": {"k": "Kalkwasserprobe"}}
        term, message = MODULE.collisions(doc)["k"]
        self.assertNotEqual(term, message)

    def test_the_template_is_not_a_language(self):
        """`_template.json` is the translator's brief and its values are
        empty by design, so every key in both sections would 'agree' at ''
        and pad the count with nothing."""
        names = [p.name for p in MODULE.bundles()]
        self.assertNotIn("_template.json", names)
        self.assertIn("de.json", names)

    def test_every_recorded_key_really_disagrees_today(self):
        """A stale exemption is a lint that has quietly stopped checking.
        This is the same guard the mutation and perturbation records use:
        a recorded entry that has healed must be deleted, and saying so is
        the only thing that keeps the recorded list honest."""
        de = json.loads((ROOT / "web/app/src/locales/de.json").read_text())
        found = MODULE.collisions(de)
        for key in MODULE.RECORDED:
            self.assertIn(key, found, f"{key!r} is recorded but no longer collides")
            term, message = found[key]
            self.assertNotEqual(
                term, message, f"{key!r} is recorded but its two sides agree now"
            )


if __name__ == "__main__":
    unittest.main()
