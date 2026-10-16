"""Package an already authenticated CLI with its matching ELF runtime.

Usage: bundle-rest-native.py BINARY VERIFIED_BUILD_MANIFEST NEW_OUTPUT_DIRECTORY
No compilation. The manifest's binary hash is checked before dependency loading.
"""
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys

binary, binding_path, output = map(Path, sys.argv[1:])
binding = json.loads(binding_path.read_text(encoding="utf-8"))
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
if sha(binary) != binding["binary_sha256"]:
    raise SystemExit("Executable does not match the verified build receipt")
header = binary.read_bytes()[:20]
if header[:6] != b"\x7fELF\x02\x01" or header[18:20] != b"\x3e\x00":
    raise SystemExit("Expected x86_64 Linux ELF")
text = subprocess.check_output(["ldd", str(binary.resolve())], text=True, env={"PATH": "/usr/bin:/bin", "LANG": "C"})
if "not found" in text:
    raise SystemExit("Unresolved native dependency")
paths = [Path(match) for match in re.findall(r"(?:=>\s+|^\s+)(/[^\s]+)", text, re.M)]
loaders = [path for path in paths if path.name.startswith("ld-linux-")]
if len(loaders) != 1:
    raise SystemExit("Expected exactly one matching ELF loader")
output.mkdir()
(output / "lib").mkdir()
for path in paths:
    shutil.copyfile(path, output / "lib" / path.name)
(output / "lib" / loaders[0].name).chmod(0o755)
shutil.copyfile(binary, output / "kero")
(output / "kero").chmod(0o755)
manifest = {key: binding[key] for key in ("source_head", "binary_sha256", "cargo_lock_sha256")}
for key in ("source_tree", "artifact_id", "source_run_id", "artifact_sha256"):
    if key in binding:
        manifest[key] = binding[key]
manifest.update(loader=loaders[0].name, target="x86_64-unknown-linux-gnu",
    packaging="Existing verified executable; bundled matching ELF loader and shared runtime; no rebuild",
    libraries={path.name: sha(path) for path in sorted((output / "lib").iterdir())})
(output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
