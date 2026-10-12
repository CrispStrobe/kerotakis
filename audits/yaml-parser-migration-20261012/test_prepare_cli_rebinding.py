"""Small synthetic preparation tests; no production CLI or Cargo is executed."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("rebinding", HERE / "prepare-cli-rebinding.py")
helper = importlib.util.module_from_spec(spec)
spec.loader.exec_module(helper)


class Preparation(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / "source"
        self.source.mkdir()
        self.audit = self.root / "audit"
        self.audit.mkdir()
        self.real_here = helper.HERE
        helper.HERE = self.audit
        self.addCleanup(setattr, helper, "HERE", self.real_here)
        for folder, commit, _, _ in helper.FREEZES.values():
            (self.audit / folder).mkdir()
            for name in ["manifest.json", "collect.py"]:
                (self.audit / folder / name).write_bytes((HERE / folder / name).read_bytes())
        original = json.loads((HERE / "cli-controls/manifest.json").read_text())
        for relative in original["source_sha256"]:
            path = self.source / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("synthetic reviewed source: " + relative + "\n")
        for entry in original["inputs"].values():
            if "repository_path" in entry:
                relative = entry["repository_path"]
                path = self.source / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes((helper.FREEZE_REPO / relative).read_bytes())
        self.git("init", "-q")
        self.git("add", ".")
        self.git("-c", "user.name=synthetic-test", "-c", "user.email=test@example.invalid", "commit", "-qm", "synthetic source")
        self.binary = self.root / "binary"
        self.binary.write_bytes(b"synthetic artifact, never executed")
        self.receipt = self.root / "receipt.json"
        self.value = dict(head=self.git("rev-parse", "HEAD").decode().strip(), source_tree=self.git("rev-parse", "HEAD^{tree}").decode().strip(), binary_sha256=helper.sha(self.binary.read_bytes()), artifact_zip_sha256="0" * 64, own_run_id=1, job_id=2, artifact_id=3)
        self.save_receipt()

    def git(self, *args):
        return subprocess.check_output(["git", "-C", str(self.source), *args], stderr=subprocess.DEVNULL)

    def save_receipt(self):
        self.receipt.write_text(json.dumps(self.value))
        self.receipt_sha = helper.sha(self.receipt.read_bytes())

    def prepare(self, matrix="cli15", out=None, receipt_sha=None):
        return helper.prepare(matrix, self.source, self.binary, self.receipt, receipt_sha or self.receipt_sha, out or self.root / matrix)

    def test_both_rebindings_preserve_all_expectations_and_collector_bytes(self):
        for matrix in helper.FREEZES:
            result = self.prepare(matrix)
            out = self.root / matrix
            original = json.loads((out / "original-manifest.json").read_text())
            rebound = json.loads((out / "manifest.json").read_text())
            lineage = rebound.pop("rebinding_lineage")
            self.assertEqual(lineage["reviewed_cli_artifact_receipt_sha256"], self.receipt_sha)
            self.assertEqual(result["manifest_sha256"], helper.sha((out / "manifest.json").read_bytes()))
            for field in ["source", "source_tree", "source_sha256", "candidate_binary_sha256"]:
                if field in original:
                    rebound[field] = original[field]
            self.assertEqual(rebound, original)
            self.assertEqual((out / "collect.py").read_bytes(), (HERE / helper.FREEZES[matrix][0] / "collect.py").read_bytes())
            self.assertFalse(result["preparation_executed_cli"])

    def test_changed_frozen_expectation_is_refused(self):
        file = self.audit / "cli-controls/manifest.json"
        value = json.loads(file.read_text())
        value["cases"][0]["expected_exit"] = 1
        file.write_text(json.dumps(value))
        with self.assertRaisesRegex(ValueError, "working freeze differs"):
            self.prepare()
        self.assertFalse((self.root / "cli15").exists())

    def test_dirty_tracked_source_is_refused(self):
        (self.source / "crates/kerotakis-cli/src/main.rs").write_text("changed after reviewed build")
        with self.assertRaisesRegex(ValueError, "tracked source bytes differ"):
            self.prepare()

    def test_wrong_source_head_is_refused(self):
        self.value["head"] = "0" * 40
        self.save_receipt()
        with self.assertRaisesRegex(ValueError, "source HEAD differs"):
            self.prepare()

    def test_wrong_receipt_sha_is_refused(self):
        with self.assertRaisesRegex(ValueError, "receipt SHA differs"):
            self.prepare(receipt_sha="0" * 64)

    def test_wrong_actual_binary_is_refused(self):
        self.binary.write_bytes(b"wrong binary")
        with self.assertRaisesRegex(ValueError, "actual binary SHA differs"):
            self.prepare()

    def test_existing_output_and_source_or_freeze_destinations_are_refused(self):
        existing = self.root / "existing"
        existing.mkdir()
        for out, text in [(existing, "output already exists"), (self.source / "evidence", "inside source"), (self.audit / "evidence", "inside audit/freeze")]:
            with self.assertRaisesRegex(ValueError, text):
                self.prepare(out=out)

    def test_source_binding_inventory_cannot_change(self):
        original = json.loads((HERE / "cli-controls/manifest.json").read_text())
        with self.assertRaisesRegex(ValueError, "source path inventory differs"):
            helper.rebound_manifest(original, self.value["head"], self.value["source_tree"], {}, self.value["binary_sha256"], {})


if __name__ == "__main__":
    unittest.main()
