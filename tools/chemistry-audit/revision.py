#!/usr/bin/env python3
"""Record or verify the changed source tree without overwriting audit evidence."""
import argparse
import hashlib
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[2]


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT)


def state():
    names = set(git("diff", "HEAD", "--name-only", "-z").decode().split("\0"))
    names.update(git("ls-files", "--others", "--exclude-standard", "-z").decode().split("\0"))
    files = {}
    for name in sorted(names):
        if not name or name.startswith("tools/chemistry-audit/"):
            continue
        path = ROOT / name
        if path.is_file():
            files[name] = hashlib.sha256(path.read_bytes()).hexdigest()
        elif not path.exists():
            files[name] = None
    return {"base": git("rev-parse", "HEAD").decode().strip(),
            "branch": git("branch", "--show-current").decode().strip(),
            "submodules": git("submodule", "status").decode(), "files": files}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=pathlib.Path)
    parser.add_argument("--verify", action="store_true")
    args = parser.parse_args()
    current = state()
    if args.verify:
        expected = json.loads(args.manifest.read_text())
        if expected != current:
            changed = sorted(k for k in expected["files"].keys() | current["files"].keys()
                             if expected["files"].get(k) != current["files"].get(k))
            raise SystemExit("Source revision differs: " + ", ".join(changed))
        print(f'Verified {len(current["files"])} changed source files and base/submodules')
    else:
        with args.manifest.open("x") as output:
            json.dump(current, output, indent=2)
            output.write("\n")
        print(f'Recorded {len(current["files"])} changed source files')
