"""`without_test_modules`, and the scan that was missing it.

On 2026-09-18 a fixture in `solve.rs` reused the live key
`routing.default-inorganic` with the stand-in sentence `"x"`, and the
collision scan — which read composer files raw — reported it as one key
meaning two different things. That is exactly the defect the scan exists
to catch, so it was not wrong; it was reading prose that reaches nobody.
Two CI cycles went past before anyone looked at which line was flagged.

The two other composer scans already went through `without_test_modules`.
This one simply had not, and nothing said so.
"""

import importlib.util
import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "engine_locale_lint", ROOT / "tools/engine-locale-lint.py"
)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


FIXTURE = '''
fn compose() -> Phrase {
    Phrase::bare("routing.default-inorganic", "the default inorganic aqueous dataset")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fixture_may_borrow_the_key() {
        let _ = Phrase::bare("routing.default-inorganic", "x");
        if true {
            let _nested = "braces inside the module must not end it early";
        }
    }
}

fn after_the_tests() -> Phrase {
    Phrase::bare("routing.extended-chemistry", "chosen because the problem needs more")
}
'''


class WithoutTestModulesTests(unittest.TestCase):
    def test_the_fixture_prose_is_removed(self):
        stripped = MODULE.without_test_modules(FIXTURE)
        self.assertNotIn('"x"', stripped)

    def test_the_real_sentence_survives(self):
        stripped = MODULE.without_test_modules(FIXTURE)
        self.assertIn("the default inorganic aqueous dataset", stripped)

    def test_engine_code_AFTER_a_test_module_survives(self):
        """#505's lesson in its sharpest form: a lint that cannot see part
        of the source is one whose percentage means nothing. `kinetics.rs`
        has a thousand lines of engine below its test module."""
        stripped = MODULE.without_test_modules(FIXTURE)
        self.assertIn("routing.extended-chemistry", stripped)

    def test_the_collision_scan_would_have_fired_on_the_raw_text(self):
        """The half of this that is a claim about the DEFECT, not the fix.

        Read raw, the fixture and the real composer disagree about one key
        — which is what the scan reported, correctly, about prose nobody
        reads. Read through `without_test_modules`, they do not.
        """
        raw = {
            m.group(2) for m in MODULE.PHRASE_CALL.finditer(FIXTURE)
            if m.group(1) == "routing.default-inorganic"
        }
        self.assertEqual(len(raw), 2, f"the raw text must collide: {raw}")

        stripped = {
            m.group(2)
            for m in MODULE.PHRASE_CALL.finditer(MODULE.without_test_modules(FIXTURE))
            if m.group(1) == "routing.default-inorganic"
        }
        self.assertEqual(len(stripped), 1, f"stripped must not collide: {stripped}")


if __name__ == "__main__":
    unittest.main()
