#!/usr/bin/env python3
"""Refuse text I/O that has no explicit encoding.

Python picks the platform's preferred encoding when you don't name one, and on
Windows that is cp1252. The v0.1.0 release build died exactly here: the Windows
runner read `curiosity/*.toml` with cp1252 and hit byte 0x81 in 'bécher' —

    UnicodeDecodeError: 'charmap' codec can't decode byte 0x81 in position 1298

A write is worse than a read, because it does not raise: it silently emits
mojibake that only a reader in another country notices.

The check is AST-based rather than textual on purpose — several of these calls
span lines, and a grep for `.write_text(` on one line cannot see the
`encoding=` two lines below it.
"""
from __future__ import annotations
import argparse, ast, pathlib, sys

# Reading bytes needs no encoding, and these callers say so.
BINARY_MODES = {"rb", "wb", "ab", "r+b", "w+b", "rb+", "wb+", "br", "bw"}
METHODS = {"read_text", "write_text"}


def keyword_names(call: ast.Call) -> set[str]:
    return {k.arg for k in call.keywords if k.arg}


def mode_of(call: ast.Call) -> str | None:
    for i, a in enumerate(call.args):
        if i == 1 and isinstance(a, ast.Constant) and isinstance(a.value, str):
            return a.value
    for k in call.keywords:
        if k.arg == "mode" and isinstance(k.value, ast.Constant):
            return k.value.value
    return None


def offences(path: pathlib.Path) -> list[tuple[int, str]]:
    try:
        tree = ast.parse(path.read_text(encoding="utf-8"))
    except (SyntaxError, UnicodeDecodeError) as exc:
        return [(getattr(exc, "lineno", 1) or 1, f"will not parse: {exc}")]
    found: list[tuple[int, str]] = []
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call):
            continue
        names = keyword_names(node)
        if "encoding" in names:
            continue
        func = node.func
        if isinstance(func, ast.Attribute) and func.attr in METHODS:
            found.append((node.lineno, f"{func.attr}() without encoding="))
        elif isinstance(func, ast.Name) and func.id == "open":
            mode = mode_of(node)
            if mode is None or "b" not in mode:
                found.append((node.lineno, "open() in text mode without encoding="))
    return found


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("roots", nargs="*", default=["tools"], type=pathlib.Path)
    ap.add_argument("--check", action="store_true",
                    help="exit non-zero on any finding (CI); otherwise only report")
    args = ap.parse_args()

    files = sorted({p for root in args.roots for p in root.rglob("*.py") if p.is_file()})
    if not files:
        print("python-encoding-lint: no files found — a silent pass is not a pass",
              file=sys.stderr)
        return 1

    total = 0
    for path in files:
        for line, why in offences(path):
            print(f"{path}:{line}: {why}")
            total += 1

    print(f"python-encoding-lint: {len(files)} files, {total} offence(s)")
    if total and args.check:
        print("::error::text I/O without an explicit encoding breaks on Windows "
              "(cp1252). Pass encoding=\"utf-8\".", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
