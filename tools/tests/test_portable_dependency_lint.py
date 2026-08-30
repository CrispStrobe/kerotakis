import importlib.util
import unittest
from pathlib import Path


SCRIPT = Path(__file__).parents[1] / "portable-dependency-lint.py"
SPEC = importlib.util.spec_from_file_location("portable_dependency_lint", SCRIPT)
MODULE = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(MODULE)


def metadata(native_is_reachable: bool):
    packages = [
        {"id": "wasm", "name": "kerotakis-wasm", "metadata": {}},
        {"id": "core", "name": "kerotakis-core", "metadata": {}},
        {
            "id": "native",
            "name": "native-engine",
            "metadata": {"kerotakis": {"runtime": "native-only"}},
        },
    ]
    return {
        "packages": packages,
        "resolve": {
            "nodes": [
                {"id": "wasm", "dependencies": ["core"]},
                {
                    "id": "core",
                    "dependencies": ["native"] if native_is_reachable else [],
                },
                {"id": "native", "dependencies": []},
            ]
        },
    }


class PortableDependencyLintTests(unittest.TestCase):
    def test_accepts_native_only_package_outside_portable_closure(self):
        self.assertEqual(MODULE.dependency_paths(metadata(False)), [])

    def test_reports_complete_path_to_native_only_package(self):
        self.assertEqual(
            MODULE.dependency_paths(metadata(True)),
            [["kerotakis-wasm", "kerotakis-core", "native-engine"]],
        )

    def test_accepts_cargo_null_metadata(self):
        fixture = metadata(False)
        fixture["packages"][1]["metadata"] = None
        self.assertEqual(MODULE.dependency_paths(fixture), [])


if __name__ == "__main__":
    unittest.main()
