"""The lint has to catch the two shapes that actually shipped broken.

Both came out of the v0.1.0 Windows build. The nested case is the one that
matters most: a first pass written as a regex "skip this call if it already
mentions encoding" let five real offences through, because an *inner*
read_text(encoding=...) satisfied the test for the *outer* write_text.
"""
import importlib.util
import pathlib
import sys
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "python_encoding_lint", ROOT / "tools/python-encoding-lint.py"
)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader
SPEC.loader.exec_module(MODULE)


def offences(source: str) -> list[tuple[int, str]]:
    with tempfile.TemporaryDirectory() as d:
        path = pathlib.Path(d) / "sample.py"
        path.write_text(source, encoding="utf-8")
        return MODULE.offences(path)


class PythonEncodingLintTests(unittest.TestCase):
    def test_a_bare_read_is_an_offence(self):
        self.assertEqual(len(offences("p.read_text()\n")), 1)

    def test_a_named_encoding_is_accepted(self):
        self.assertEqual(offences('p.read_text(encoding="utf-8")\n'), [])

    def test_a_bare_write_is_an_offence(self):
        """Worse than a read: it does not raise, it emits mojibake."""
        self.assertEqual(len(offences('p.write_text("x")\n')), 1)

    def test_an_inner_encoding_does_not_excuse_the_outer_call(self):
        """The false negative that shipped: five of these passed a regex."""
        found = offences('a.write_text(b.read_text(encoding="utf-8"))\n')
        self.assertEqual([why for _, why in found],
                         ["write_text() without encoding="])

    def test_a_call_spanning_lines_is_still_seen(self):
        """Why this is AST-based: `encoding=` is two lines below the call."""
        self.assertEqual(offences('p.write_text(\n    json.dumps(x),\n'
                                  '    encoding="utf-8",\n)\n'), [])
        self.assertEqual(len(offences('p.write_text(\n    json.dumps(x),\n)\n')), 1)

    def test_binary_needs_no_encoding(self):
        self.assertEqual(offences('open(p, "rb")\n'), [])
        self.assertEqual(offences('open(p, mode="wb")\n'), [])

    def test_text_open_without_encoding_is_an_offence(self):
        self.assertEqual(len(offences('open(p)\n')), 1)
        self.assertEqual(len(offences('open(p, "w")\n')), 1)

    def test_a_file_that_will_not_parse_is_reported_not_skipped(self):
        """A lint that returns "no offences" for a file it could not read is
        the failure mode this whole change is about."""
        found = offences("def (:\n")
        self.assertEqual(len(found), 1)
        self.assertIn("will not parse", found[0][1])

    def test_the_repository_is_clean(self):
        """The lint's own subject. If someone adds a bare call, this fails
        here as well as in CI."""
        bad = [(p, line, why)
               for p in sorted((ROOT / "tools").rglob("*.py")) if p.is_file()
               for line, why in MODULE.offences(p)]
        self.assertEqual(bad, [], f"{len(bad)} call(s) without an encoding")

    def test_an_empty_file_list_is_a_failure_not_a_pass(self):
        """`main` must refuse to report success over nothing at all — a lint
        pointed at the wrong directory is the shape that hides real work."""
        with tempfile.TemporaryDirectory() as empty:
            self.assertEqual(self.run_main(empty), 1)

    @staticmethod
    def run_main(directory: str) -> int:
        argv = sys.argv
        sys.argv = ["python-encoding-lint.py", directory, "--check"]
        try:
            return MODULE.main()
        finally:
            sys.argv = argv

    # --- inline python in shell scripts and workflows ---

    def inline(self, source: str, suffix: str = ".sh") -> list[tuple[int, str]]:
        with tempfile.TemporaryDirectory() as d:
            path = pathlib.Path(d) / f"sample{suffix}"
            path.write_text(source, encoding="utf-8")
            return MODULE.inline_offences(path)

    def test_a_one_liner_reading_a_config_is_caught(self):
        """The real shape: three scripts read tauri.conf.json this way."""
        self.assertEqual(len(self.inline(
            """VERSION="$(python3 -c "import json; print(json.load(open('$T/x.json'))['version'])")"\n""")), 1)

    def test_an_inline_write_is_caught(self):
        self.assertEqual(len(self.inline('open(os.path.join(d, "i.json"), "w").write(x)\n')), 1)

    def test_a_named_encoding_inline_is_accepted(self):
        self.assertEqual(self.inline('manifest = open(sys.argv[1], encoding="utf-8").read()\n'), [])

    def test_binary_inline_is_accepted(self):
        self.assertEqual(self.inline('doc = tomllib.load(open(f, "rb"))\n'), [])

    def test_a_shell_comment_is_not_code(self):
        self.assertEqual(self.inline('# once upon a time this said open(x)\n'), [])

    def test_workflows_are_scanned_too(self):
        self.assertEqual(len(self.inline('        run: python3 -c "print(open(p).read())"\n',
                                         suffix=".yml")), 1)

    def test_the_workflows_directory_is_clean(self):
        bad = [(p, line) for p in sorted((ROOT / ".github").rglob("*.yml"))
               for line, _ in MODULE.inline_offences(p)]
        self.assertEqual(bad, [])

    def test_shell_scripts_under_tools_are_clean(self):
        bad = [(p.name, line) for p in sorted((ROOT / "tools").rglob("*.sh"))
               for line, _ in MODULE.inline_offences(p)]
        self.assertEqual(bad, [])


if __name__ == "__main__":
    unittest.main()
