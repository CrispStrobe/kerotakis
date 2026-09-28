#!/usr/bin/env python3
"""Catch workflow steps that are wrong in ways that have already cost something.

Two shapes, both from real failures: an input that ended up on the wrong
step, and a `run:` block whose pipeline throws away the exit status of the
command that matters.

`actions/setup-node` with `cache: npm` and no `cache-dependency-path` looks
fine, parses fine, and fails only where there is no `package-lock.json` beside
the workflow's working directory. That is what broke the App Store upload for
v0.1.1: adding a `setup-python` step to appstore.yml inserted it *between*
`cache: npm` and the `cache-dependency-path:` line below it, so node lost its
lockfile and python inherited a line meant for node. Both steps still parsed.

The class is an edit landing inside a structure it was not meant to enter, and
YAML validity says nothing about it. So the shapes that have actually bitten
get named here:

- `setup-node` with `cache` but no `cache-dependency-path` — npm then looks for
  a lockfile at the repository root, which this repo does not have.
- a `with:` key that belongs to a different action than the one it sits on.

Only steps that are wrong in a way that has cost something are listed. A lint
that flags every unfamiliar input would be one people stop reading.
"""
from __future__ import annotations
import argparse, pathlib, sys

import yaml

WORKFLOWS = pathlib.Path(__file__).resolve().parent.parent / ".github/workflows"

# Inputs each action owns, for the "this belongs to another step" check. Only
# actions used in this repository, and only inputs it actually passes.
OWNED = {
    "actions/setup-node": {"node-version", "node-version-file", "cache",
                           "cache-dependency-path", "registry-url", "scope",
                           "check-latest", "architecture", "always-auth"},
    "actions/setup-python": {"python-version", "python-version-file", "cache",
                             "cache-dependency-path", "architecture",
                             "check-latest", "update-environment",
                             "allow-prereleases", "token"},
}


def action_of(uses: str) -> str:
    return uses.split("@", 1)[0]


def findings(path: pathlib.Path) -> list[str]:
    doc = yaml.safe_load(path.read_text(encoding="utf-8"))
    out: list[str] = []
    for job_name, job in (doc.get("jobs") or {}).items():
        for step in job.get("steps") or []:
            uses = step.get("uses") or ""
            if not uses:
                continue
            action = action_of(uses)
            with_ = step.get("with") or {}

            if action == "actions/setup-node" and "cache" in with_ \
                    and "cache-dependency-path" not in with_:
                out.append(
                    f"{path.name}:{job_name}: setup-node sets `cache: "
                    f"{with_['cache']}` with no `cache-dependency-path`; npm "
                    "will look for a lockfile at the repository root"
                )

            owned = OWNED.get(action)
            if owned:
                for key in with_:
                    if key in owned:
                        continue
                    elsewhere = [a for a, keys in OWNED.items()
                                 if a != action and key in keys]
                    out.append(
                        f"{path.name}:{job_name}: {action} has input {key!r}"
                        + (f", which belongs to {elsewhere[0]}" if elsewhere
                           else ", which it does not accept")
                    )
    return out


PIPE_NEEDS_PIPEFAIL = ("| tee", "|tee", "| grep", "| head", "| tail")


def pipeline_findings(path: pathlib.Path) -> list[str]:
    """`run:` blocks that pipe a command whose failure would be swallowed.

    A shell pipeline exits with the status of its LAST command, so

        npx vercel deploy ... | tee "$RUNNER_TEMP/url.txt"

    reports tee's success no matter what the CLI did. The Vercel deploy failed
    with `Error: The "--prebuilt" option was used...` and the job went green.
    `tools/deploy-vercel.sh` had already learned this — "take ITS exit code
    directly instead of trusting log greps" — but the workflow had not.

    Only multi-line `run:` blocks are judged: a one-line `run:` is a single
    command whose status GitHub takes directly, and `set -o pipefail` cannot be
    written there anyway.
    """
    doc = yaml.safe_load(path.read_text(encoding="utf-8"))
    out = []
    for job_name, job in (doc.get("jobs") or {}).items():
        for step in job.get("steps") or []:
            run = step.get("run")
            if not run or "\n" not in run.strip():
                continue
            if "pipefail" in run:
                continue
            for marker in PIPE_NEEDS_PIPEFAIL:
                if marker in run:
                    name = step.get("name") or run.strip().splitlines()[0][:40]
                    out.append(
                        f"{path.name}:{job_name}: step {name!r} pipes into "
                        f"`{marker.strip()}` with no `set -o pipefail`; the step "
                        "reports the pipe's exit status, not the command's"
                    )
                    break
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--check", action="store_true", help="exit non-zero on a finding")
    args = ap.parse_args()

    files = sorted(WORKFLOWS.glob("*.yml"))
    if not files:
        print("workflow-step-lint: no workflows found — not a pass",
              file=sys.stderr)
        return 1

    found = [row for path in files for row in findings(path)]
    found += [row for path in files for row in pipeline_findings(path)]
    for row in found:
        print(row)
    print(f"workflow-step-lint: {len(files)} workflows, {len(found)} finding(s)")
    if found and args.check:
        print("::error::a step input is on the wrong step, a cache has no "
              "dependency path, or a pipeline is hiding a failure", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
