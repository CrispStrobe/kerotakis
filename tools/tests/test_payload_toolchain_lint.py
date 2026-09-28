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
        needs = MODULE.python_tools_needing("tomllib")
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


class WasmToolchainTests(unittest.TestCase):
    """The second requirement, learned the same way as the first: vercel.yml
    had existed for weeks, had never been dispatched, and failed on its first
    real run with the wasm target missing."""

    def test_build_web_is_recognised_as_a_wasm_builder(self):
        self.assertIn("tools/build-web.sh", MODULE.wasm_building_scripts())

    def test_every_workflow_that_builds_wasm_installs_the_target(self):
        self.assertEqual(MODULE.wasm_offenders(), [])

    def test_a_workflow_with_no_wasm_is_not_flagged(self):
        """The check must not demand a wasm target of workflows that build
        none — a lint that fires on unrelated files is one people route
        around."""
        scripts = MODULE.wasm_building_scripts()
        for wf in sorted(MODULE.WORKFLOWS.glob("*.yml")):
            text = MODULE.uncommented(wf.read_text(encoding="utf-8"))
            if "wasm32-unknown-unknown" in text or any(s in text for s in scripts):
                continue
            self.assertNotIn(wf.name, " ".join(MODULE.wasm_offenders()))


class CryptographyTests(unittest.TestCase):
    """The ASC client signs its JWT with cryptography, and two workflows ran
    its scripts without installing it — one that failed and one that had never
    been dispatched and would have."""

    def test_client_is_found_as_the_importer(self):
        needing = MODULE.python_tools_needing("cryptography", subdirs=("asc",))
        self.assertIn("client.py", needing)

    def test_the_import_chain_is_followed_into_the_scripts(self):
        """No asc script but client.py names cryptography; they all import
        client, which is the whole reason the scan is transitive."""
        needing = MODULE.python_tools_needing("cryptography", subdirs=("asc",))
        self.assertIn("testflight.py", needing)
        self.assertIn("store.py", needing)

    def test_every_workflow_running_an_asc_script_installs_it(self):
        self.assertEqual(MODULE.cryptography_offenders(), [])

    def test_an_empty_scan_reports_itself_as_broken(self):
        """A scan that finds no importer must not return "clean" — that is
        indistinguishable from a repository with nothing to fix."""
        rows = MODULE.cryptography_offenders.__doc__
        self.assertIn("client.py", rows)
