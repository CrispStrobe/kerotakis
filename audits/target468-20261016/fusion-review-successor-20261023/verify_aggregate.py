#!/usr/bin/env python3
"""Aggregate saved-byte verification for the published water-fusion projections.

This checks only the public capsule projections and immutable Git objects. It
never executes a native test binary or makes claims about private archives.
Run with: python3 -B aggregate_public_verifier.py --repo-root .
"""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Iterable


class EvidenceError(AssertionError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise EvidenceError(message)


def read_json(path: Path):
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def git(repo: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", "--no-replace-objects", "-C", str(repo), *args],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    return result.stdout.strip()


def check_tree(actual_tree: str, expected_tree: str) -> None:
    require(actual_tree == expected_tree,
            f"source tree mismatch: actual={actual_tree} expected={expected_tree}")


def check_unique(identities: Iterable[tuple[str, str, str]]) -> int:
    rows = list(identities)
    require(len(rows) == len(set(rows)), "duplicate (package,target,test) identity")
    return len(rows)


def expect_rejection(label, check) -> str:
    try:
        check()
    except EvidenceError as exc:
        return f"rejected as expected: {exc}"
    raise EvidenceError(f"negative control did not reject: {label}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", required=True, type=Path)
    args = parser.parse_args()
    repo = args.repo_root.resolve(strict=True)
    require((repo / ".git").exists(), f"not a Git worktree: {repo}")

    capsule_relpaths = [
        Path("audits/target468-20261016/water-fusion46-focused-accepted-20261023"),
        Path("audits/target468-20261016/water-fusion-native5-focused-accepted-20261023"),
    ]
    capsule_roots = [repo / item for item in capsule_relpaths]
    for root in capsule_roots:
        require(root.is_dir(), f"missing public capsule: {root}")
        verifier = root / "verify_public.py"
        result = subprocess.run(
            ["python3", "-B", str(verifier)], cwd=repo,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
        )
        require(result.returncode == 0,
                f"portable verifier failed ({verifier.name}): {result.stderr.strip()}")
        print(f"portable_verifier={verifier.relative_to(repo)} exit={result.returncode} output={result.stdout.strip()}")

    acceptances = [read_json(root / "acceptance.json") for root in capsule_roots]
    source_ids = {item["source"] for item in acceptances}
    declared_trees = {item["tree"] for item in acceptances}
    require(len(source_ids) == 1, f"source IDs differ: {sorted(source_ids)}")
    require(len(declared_trees) == 1, f"declared trees differ: {sorted(declared_trees)}")
    source = next(iter(source_ids))
    declared_tree = next(iter(declared_trees))
    actual_tree = git(repo, "rev-parse", "--verify", f"{source}^{{tree}}")
    check_tree(actual_tree, declared_tree)

    lock_hashes = set()
    groups = []
    identities: list[tuple[str, str, str]] = []
    for root, acceptance in zip(capsule_roots, acceptances):
        lock_hash = sha256(root / "actual-producer-Cargo.lock")
        lock_hashes.add(lock_hash)
        for group in acceptance["groups"]:
            group_root = root / group["name"]
            binding = read_json(group_root / "binding-projection.json")
            require(binding["source"] == source, f"binding source mismatch in {group_root}")
            require(binding["tree"] == declared_tree, f"binding tree mismatch in {group_root}")
            require(binding["actual_lock_sha256"] == lock_hash,
                    f"actual lock binding mismatch in {group_root}")
            require(binding.get("public_projection_is_not_complete_raw_collection") is True,
                    f"projection/raw-collection limitation missing in {group_root}")
            # Recheck exact source inputs without local Git replacement refs.
            for path, row in binding['source_pins'].items():
                raw = subprocess.check_output(
                    ['git', '--no-replace-objects', '-C', str(repo),
                     'show', source + ':' + path])
                require(hashlib.sha256(raw).hexdigest() == row['sha256'],
                        'immutable source input mismatch: ' + path)
            group_count = 0
            for target in binding["tracked_targets"]:
                package = target["package"]
                target_name = target["target"]
                for test in target["tests"]:
                    identities.append((package, target_name, test))
                    group_count += 1
            require(group_count == group["passed"],
                    f"declared group count differs from projection for {group_root}")
            groups.append({"capsule": root.name, "group": group["name"], "controls": group_count})
    require(len(lock_hashes) == 1, f"producer lock hashes differ: {sorted(lock_hashes)}")
    lock_hash = next(iter(lock_hashes))
    total = check_unique(identities)
    require(total == sum(item["passed_controls"] for item in acceptances),
            "enumerated total differs from acceptance totals")
    require(total == 51, f"expected three named focused stages to reconcile to 51, found {total}")

    # Negative controls use the actual source tree and an actual published
    # identity. They change only the expected check input in memory.
    wrong_tree = ("0" if declared_tree[0] != "0" else "1") + declared_tree[1:]
    tree_negative = expect_rejection(
        "wrong expected tree", lambda: check_tree(actual_tree, wrong_tree)
    )
    duplicate_negative = expect_rejection(
        "duplicate real control identity",
        lambda: check_unique(identities + [identities[0]]),
    )

    print(json.dumps({
        "result": "PASS_SAVED_PUBLIC_PROJECTIONS_ONLY",
        "source": source,
        "declared_tree": declared_tree,
        "git_no_replace_actual_tree": actual_tree,
        "actual_producer_lock_sha256": lock_hash,
        "groups": groups,
        "distinct_package_target_test_controls": total,
        "negative_controls": {
            "wrong_expected_tree": tree_negative,
            "duplicated_actual_identity": duplicate_negative,
        },
        "scope_limits": [
            "Existing capsule verifiers ran with Python bytecode writing disabled.",
            "No engine, native executable, test API, provider, or private archive was executed or accessed.",
            "Public projection metadata does not prove rehashing private executable bytes or completeness of private raw collections.",
            "No full-source, platform, protected-CI, integration, product, calibration, or experiment qualification claim is made.",
        ],
    }, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (EvidenceError, OSError, subprocess.CalledProcessError, json.JSONDecodeError) as exc:
        print(f"aggregate verification failed: {exc}", file=sys.stderr)
        raise SystemExit(1)
