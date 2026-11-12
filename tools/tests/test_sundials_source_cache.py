"""Exercise real archive hashes/extraction with a tiny, offline transport fixture.

SUNDIALS_FETCH_SCRIPT can name a proposed script during source-only review.
Only the fixture's pinned digest changes; curl supplies bytes, never a verdict.
"""
import hashlib
import io
import os
from pathlib import Path
import re
import subprocess
import tarfile
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = Path(os.environ.get("SUNDIALS_FETCH_SCRIPT", ROOT / "tools/fetch-sundials.sh"))


class SundialsSourceCacheTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.archive = self.root / "fixture.tar.gz"
        self.payload = b"# verified fixture source\n"
        with tarfile.open(self.archive, "w:gz") as archive:
            entry = tarfile.TarInfo("sundials-7.2.1/CMakeLists.txt")
            entry.size = len(self.payload)
            archive.addfile(entry, io.BytesIO(self.payload))
        self.digest = hashlib.sha256(self.archive.read_bytes()).hexdigest()
        tools = self.root / "tools"
        tools.mkdir()
        self.script = tools / "fetch-sundials.sh"
        body, count = re.subn(r'^SHA256="[0-9a-f]{64}"$',
                             f'SHA256="{self.digest}"', SCRIPT.read_text(),
                             flags=re.MULTILINE)
        self.assertEqual(count, 1, "exactly one authoritative pin is required")
        self.script.write_text(body)
        binary = self.root / "bin"
        binary.mkdir()
        curl = binary / "curl"
        curl.write_text('''#!/usr/bin/env bash
set -euo pipefail
printf 'called\\n' >> "$TRANSPORT_CALLS"
if [ "${TRANSPORT_FAIL:-0}" = 1 ]; then exit 22; fi
output=''
while [ "$#" -gt 0 ]; do
    if [ "$1" = -o ]; then output="$2"; shift 2; else shift; fi
done
test -n "$output"
cp "$TRANSPORT_ARCHIVE" "$output"
''')
        curl.chmod(0o755)
        self.calls = self.root / "transport-calls"
        self.cache = self.root / "cache" / f"{self.digest}.tar.gz"
        self.dest = self.root / "sundials-kinetics-rs/sundials-sys/sundials_src"
        self.env = dict(os.environ, PATH=f"{binary}:{os.environ['PATH']}",
                        TRANSPORT_CALLS=str(self.calls),
                        TRANSPORT_ARCHIVE=str(self.archive),
                        SUNDIALS_ARCHIVE_CACHE=str(self.cache))
        self.env.pop("TRANSPORT_FAIL", None)

    def run_fetch(self, *arguments):
        return subprocess.run(["bash", str(self.script), *arguments],
                              env=self.env, capture_output=True, text=True,
                              timeout=10)

    def seed_cache(self, data=None):
        self.cache.parent.mkdir()
        self.cache.write_bytes(self.archive.read_bytes() if data is None else data)

    def assert_source(self):
        self.assertEqual((self.dest / "CMakeLists.txt").read_bytes(), self.payload)

    def test_metadata_reads_the_pin_without_fetch_or_extract(self):
        result = self.run_fetch("--archive-sha256")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout.strip(), self.digest)
        self.assertFalse(self.calls.exists())
        self.assertFalse(self.cache.exists())
        self.assertFalse(self.dest.exists())

    def test_verified_cache_hit_extracts_without_network(self):
        self.seed_cache()
        result = self.run_fetch()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assert_source()
        self.assertFalse(self.calls.exists())

    def test_corrupt_cache_is_rejected_without_extraction_or_redownload(self):
        corrupt = self.archive.read_bytes() + b"tampered"
        self.seed_cache(corrupt)
        result = self.run_fetch()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("checksum", result.stderr)
        self.assertFalse(self.dest.exists())
        self.assertFalse(self.calls.exists())
        self.assertEqual(self.cache.read_bytes(), corrupt)

    def test_existing_source_does_not_hide_corrupt_cache(self):
        self.seed_cache(b"corrupt cache")
        self.dest.mkdir(parents=True)
        (self.dest / "CMakeLists.txt").write_bytes(b"existing source")
        result = self.run_fetch()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("checksum", result.stderr)
        self.assertFalse(self.calls.exists())
        self.assertEqual((self.dest / "CMakeLists.txt").read_bytes(), b"existing source")

    def test_cache_miss_downloads_and_publishes_verified_bytes(self):
        result = self.run_fetch()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls.read_text(), "called\n")
        self.assertEqual(self.cache.read_bytes(), self.archive.read_bytes())
        self.assert_source()
        self.assertEqual(list(self.cache.parent.iterdir()), [self.cache])

    def test_corrupt_download_is_never_published_or_extracted(self):
        self.archive.write_bytes(self.archive.read_bytes() + b"tampered")
        result = self.run_fetch()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("checksum", result.stderr)
        self.assertEqual(self.calls.read_text(), "called\n")
        self.assertFalse(self.cache.exists())
        self.assertFalse(self.dest.exists())

    def test_failed_download_is_never_published_or_extracted(self):
        self.env["TRANSPORT_FAIL"] = "1"
        result = self.run_fetch()
        self.assertEqual(result.returncode, 22, result.stderr)
        self.assertEqual(self.calls.read_text(), "called\n")
        self.assertFalse(self.cache.exists())
        self.assertFalse(self.dest.exists())

    def test_local_call_without_cache_keeps_download_and_source_fast_path(self):
        del self.env["SUNDIALS_ARCHIVE_CACHE"]
        result = self.run_fetch()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assert_source()
        result = self.run_fetch()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls.read_text(), "called\n")
        self.assertFalse(self.cache.exists())


if __name__ == "__main__":
    unittest.main()
