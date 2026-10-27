"""Regenerate the multi-word alias block in services/rest_api/app.py from the blessed input-alias fixture.

The REST validator counts loop work by word position, so every multi-word
spelling the engine accepts must collapse to one token first. The engine's
generated table is the single source; this file is derived, never edited.
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FIXTURE = ROOT / "crates/kerotakis-core/tests/fixtures/input-alias-coverage-20261027.json"
TARGET = ROOT / "services/rest_api/app.py"
BEGIN = "# BEGIN generated multi-word aliases (tools/gen_api_multiword_aliases.py)\n"
END = "# END generated multi-word aliases\n"


def render() -> str:
    rows = json.loads(FIXTURE.read_text())["aliases"]
    table = sorted({tuple(row["alias"].lower().split()): row["canonical"]
                    for row in rows if len(row["alias"].split()) > 1}.items())
    body = "".join(f"    {alias!r}: {canonical!r},\n" for alias, canonical in table)
    block = "MULTIWORD_INGREDIENT_ALIASES = {\n" + body + "}\n"
    source = TARGET.read_text()
    head, rest = source.split(BEGIN, 1)
    return head + BEGIN + block + END + rest.split(END, 1)[1]


if __name__ == "__main__":
    if sys.argv[1:] == ["--check"]:
        sys.exit(TARGET.read_text() != render())
    TARGET.write_text(render())
