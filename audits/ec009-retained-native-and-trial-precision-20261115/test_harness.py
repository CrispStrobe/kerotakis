"""Tiny synthetic adapter controls: never execute chemistry or count originals."""
import copy
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
import diagnostic as d


class HarnessBoundaryTests(unittest.TestCase):
    def test_exact_producer_binding_and_seven_corruptions(self):
        good = dict(source=d.SOURCE, tree=d.TREE, sha256=d.CACHE,
                    actual_lock_sha256=d.LOCK, toolchain="1.100.0")
        d.validate_binding(good, d.COMPILER, d.LOCK)
        for key in good:
            with self.subTest(key=key):
                bad = copy.deepcopy(good)
                bad[key] = "wrong"
                with self.assertRaises(AssertionError):
                    d.validate_binding(bad, d.COMPILER, d.LOCK)
        with self.assertRaises(AssertionError):
            d.validate_binding(good, "wrong", d.LOCK)
        with self.assertRaises(AssertionError):
            d.validate_binding(good, d.COMPILER, "wrong")

    def archive(self, path, members):
        with tarfile.open(path, "w:gz") as archive:
            for name, kind, content in members:
                info = tarfile.TarInfo(name)
                info.type = kind
                if kind == tarfile.REGTYPE:
                    info.size = len(content)
                    archive.addfile(info, io.BytesIO(content))
                else:
                    info.linkname = "/outside"
                    archive.addfile(info)

    def test_exact_member_only_no_path_extraction(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.archive(root/"cache.tgz", [("../../outside", tarfile.REGTYPE, b"ignored"),
                         (d.MEMBER, tarfile.REGTYPE, b"synthetic executable bytes")])
            d.copy_member(root/"cache.tgz", root/"chosen")
            self.assertEqual((root/"chosen").read_bytes(), b"synthetic executable bytes")
            self.assertEqual({p.name for p in root.iterdir()}, {"chosen", "cache.tgz"})

    def test_missing_duplicate_symlink_and_empty_member_refuse(self):
        designs = [[], [(d.MEMBER, tarfile.REGTYPE, b"x")]*2,
                   [(d.MEMBER, tarfile.SYMTYPE, b"")],
                   [(d.MEMBER, tarfile.REGTYPE, b"")]]
        for members in designs:
            with self.subTest(members=members), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                self.archive(root/"cache.tgz", members)
                with self.assertRaises(AssertionError):
                    d.copy_member(root/"cache.tgz", root/"chosen")


if __name__ == "__main__":
    unittest.main()
