"""The lint's value is entirely in the edges it follows, so the tests are
about the three ways it was wrong while being written.
"""
import importlib.util
import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "payload_toolchain_lint", ROOT / "tools/payload-toolchain-lint.py"
)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


class PayloadToolchainLintTests(unittest.TestCase):
    def test_comments_are_not_calls(self):
        """It reported its own prose as the call chain until this existed."""
        self.assertEqual(
            MODULE.uncommented("real_call()\n# tauri build is mentioned here\n").strip(),
            "real_call()")
        self.assertEqual(MODULE.uncommented("run()  # trailing").strip(), "run()")

    def test_tomllib_is_found_through_a_sibling_module(self):
        """lessons-index.py never says tomllib; lesson_prose.py does."""
        needs = MODULE.python_tools_needing_tomllib()
        self.assertIn("lesson_prose.py", needs)
        self.assertIn("lessons-index.py", needs,
                      "the import chain lesson_prose -> tomllib was not followed")

    def test_the_web_build_is_reached_even_though_it_calls_no_payload_script(self):
        """The first version keyed on build-shell-payload.sh and missed this:
        build-web.sh invokes the same four tools itself."""
        self.assertIn("tools/build-web.sh", MODULE.reaching_tokens())

    def test_tauri_before_build_command_counts_as_reaching(self):
        """appstore.yml names only a shell script, which runs `npx tauri build`,
        which runs beforeBuildCommand. Three hops, no mention of Python."""
        tokens = MODULE.reaching_tokens()
        self.assertIn("tauri build", tokens)
        self.assertIn("tools/build-macos-appstore.sh", tokens,
                      "the fixed point did not interleave: `tauri build` became "
                      "a token after the shell scripts had already been grown")

    def test_every_workflow_that_reaches_it_pins_python(self):
        bad, rows = MODULE.offenders(MODULE.reaching_tokens())
        self.assertEqual(bad, [], f"unpinned: {bad}")
        names = {name for name, _, _ in rows}
        # If one of these stops appearing, the scan broke rather than the repo
        # improving — that is how the appstore.yml row vanished mid-change.
        for expected in ("ci.yml", "release.yml", "appstore.yml", "vercel.yml"):
            self.assertIn(expected, names)

    def test_an_empty_scan_is_a_failure_not_a_pass(self):
        self.assertTrue(MODULE.reaching_tokens(),
                        "a lint that finds nothing must not report success")


if __name__ == "__main__":
    unittest.main()
