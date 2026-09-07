#!/usr/bin/env python3
"""Build the browser's static capability index from reviewed corpus data.

The index carries the corpus's authored English AND every language shipped
beside it: `tools/curiosity-prose.py` validates
`tests/coverage/curiosity-v1/i18n/<locale>.toml` and folds it in here as
`question_de`-style siblings, so the explorer reads one file and adding a
language changes nothing in this exporter.
"""

from __future__ import annotations

import importlib.util
import json
import pathlib
import sys
import tomllib

# The validator's filename is hyphenated like every other tool here, which
# is not importable by name; loading it by path keeps ONE definition of
# what a valid translation is, shared by this exporter and the preflight
# gate, rather than a second copy that drifts.
_SPEC = importlib.util.spec_from_file_location(
    "curiosity_prose", pathlib.Path(__file__).with_name("curiosity-prose.py")
)
prose = importlib.util.module_from_spec(_SPEC)
assert _SPEC.loader
_SPEC.loader.exec_module(prose)


def build(corpus: pathlib.Path) -> dict:
    manifest = tomllib.loads((corpus / "manifest.toml").read_text())
    baseline = tomllib.loads((corpus / "baseline.toml").read_text())
    observed = {row["id"]: row for row in baseline["observation"]}
    prompts = []
    for shard in manifest["shards"]:
        for prompt in tomllib.loads((corpus / shard).read_text())["prompt"]:
            result = observed[prompt["id"]]
            prompts.append({
                "id": prompt["id"],
                "question": prompt["question"],
                "age_band": prompt["age_band"],
                "topic": prompt["action"],
                "material_class": prompt["material_class"],
                "tags": prompt.get("tags", []),
                "script": prompt.get("script", []),
                "owning_task": result["owning_task"],
                "support": result["outcome"],
                "reason_code": result["reason_code"],
                "boundary": prompt.get("parse_boundary"),
            })
    prompts.sort(key=lambda row: row["id"])
    if len(prompts) != manifest["target_prompts"] or set(observed) != {p["id"] for p in prompts}:
        raise ValueError("corpus and reviewed baseline must describe the same target prompt set")
    prose.apply(prompts, prose.read_translations(corpus))
    return {"schema": 1, "corpus": manifest["id"], "prompts": prompts}


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("usage: curiosity-index.py <curiosity-dir> <output.json>")
    output = pathlib.Path(sys.argv[2])
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(build(pathlib.Path(sys.argv[1])), indent=1) + "\n")
