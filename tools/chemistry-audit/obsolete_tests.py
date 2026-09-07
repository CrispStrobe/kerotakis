#!/usr/bin/env python3
"""Plan/apply removal of old, rebuildable test executables in this worktree only."""
import argparse
import datetime
import json
import pathlib
import stat

ROOT = pathlib.Path(__file__).resolve().parents[2]
SAFE = (ROOT / "target/debug/deps").resolve()
CUTOFF = datetime.datetime(2026, 9, 6, 21, tzinfo=datetime.timezone.utc).timestamp()

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=pathlib.Path)
    parser.add_argument("--apply", action="store_true")
    args = parser.parse_args()
    if not args.apply:
        entries = []
        for path in sorted(SAFE.iterdir()):
            if path.is_symlink() or path.suffix or path.name.startswith("kero"):
                continue
            info = path.stat()
            if stat.S_ISREG(info.st_mode) and info.st_mode & stat.S_IXUSR and info.st_mtime < CUTOFF:
                entries.append({"path": str(path), "bytes": info.st_size, "mtime_ns": info.st_mtime_ns})
        with args.manifest.open("x") as output:
            json.dump({"safe_root": str(SAFE), "entries": entries}, output, indent=2)
            output.write("\n")
        print(f"Planned {len(entries)} old test executables, {sum(e['bytes'] for e in entries)} bytes")
    else:
        plan = json.loads(args.manifest.read_text())
        assert plan["safe_root"] == str(SAFE)
        for entry in plan["entries"]:
            path = pathlib.Path(entry["path"])
            assert not path.is_symlink() and path.resolve().parent == SAFE
            assert not path.suffix and not path.name.startswith("kero")
            info = path.stat()
            assert stat.S_ISREG(info.st_mode) and info.st_mode & stat.S_IXUSR
            assert info.st_mtime < CUTOFF
            assert (info.st_size, info.st_mtime_ns) == (entry["bytes"], entry["mtime_ns"])
        # Validation of every exact target precedes any deletion.
        for entry in plan["entries"]:
            pathlib.Path(entry["path"]).unlink()
        print(f"Removed {len(plan['entries'])} obsolete test executables; recoverable by rebuilding")
