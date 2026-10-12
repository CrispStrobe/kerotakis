#!/usr/bin/env python3
"""Prepare a lineage-bound copy of a frozen CLI matrix; never execute the CLI."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess

HERE = Path(__file__).resolve().parent
FREEZE_REPO = HERE.parents[1]
FREEZES = {
    "cli15": ("cli-controls", "6ae7b5bb7d0c6fedb5faffa62e16249e6c6b9263", "61b7277115004eb0ffd27f55a94a63f7880940a07f7d692da9339d8151071389", "8c5aa8f733e78e1b3dd63be69f9a38e9198452696fe1dfe4944ccc228b885ebf"),
    "nested12": ("nested-rate-controls", "c1fe0ef8f29f57de3b71365714cc1297ba0afe09", "a7193ca986bdb7b1c895110fc4f3ac72f48310243d5a857b8250c6df823d7d09", "69b3c7c4ea1ae1f459e2bd83a9dc5953cc94943db0a622ad0508ceacb6bfb4ba"),
}


def sha(blob):
    return hashlib.sha256(blob).hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def git(repo, *args):
    return subprocess.check_output(["git", "-C", str(repo), *args])


def frozen(matrix):
    folder, commit, manifest_sha, collector_sha = FREEZES[matrix]
    root = HERE / folder
    blobs = {}
    for filename in ["manifest.json", "collect.py"]:
        relative = "audits/yaml-parser-migration-20261012/" + folder + "/" + filename
        blob = git(FREEZE_REPO, "show", commit + ":" + relative)
        require((root / filename).read_bytes() == blob, "working freeze differs: " + filename)
        blobs[filename] = blob
    require(sha(blobs["manifest.json"]) == manifest_sha, "original manifest SHA differs")
    require(sha(blobs["collect.py"]) == collector_sha, "original collector SHA differs")
    value = json.loads(blobs["manifest.json"])
    require(value["collector_sha256"] == collector_sha, "original collector binding differs")
    return value, blobs, commit


def verify_source(source, head, tree):
    require(git(source, "rev-parse", "HEAD").decode().strip() == head, "source HEAD differs")
    require(git(source, "rev-parse", "HEAD^{tree}").decode().strip() == tree, "source tree differs")
    count, gitlinks = 0, []
    for entry in git(source, "ls-tree", "-r", "-z", head).split(b"\0"):
        if not entry:
            continue
        metadata, raw_path = entry.split(b"\t", 1)
        mode, kind, digest = metadata.split()
        relative = raw_path.decode()
        if kind == b"commit":
            gitlinks.append(dict(path=relative, revision=digest.decode()))
            continue
        path = source / relative
        blob = os.fsencode(os.readlink(path)) if mode == b"120000" else path.read_bytes()
        actual = hashlib.sha1(b"blob " + str(len(blob)).encode() + b"\0" + blob).hexdigest()
        require(actual == digest.decode(), "tracked source bytes differ: " + relative)
        count += 1
    return count, gitlinks


def rebound_manifest(original, head, tree, source_hashes, binary_sha, lineage):
    value = copy.deepcopy(original)
    require(set(source_hashes) == set(original["source_sha256"]), "source path inventory differs")
    value.update(source=head, source_tree=tree, source_sha256=source_hashes)
    if "candidate_binary_sha256" in original:
        value["candidate_binary_sha256"] = binary_sha
    value["rebinding_lineage"] = lineage
    unchanged = copy.deepcopy(value)
    del unchanged["rebinding_lineage"]
    for field in ["source", "source_tree", "source_sha256", "candidate_binary_sha256"]:
        if field in original:
            unchanged[field] = original[field]
    require(unchanged == original, "expectations or other frozen fields changed")
    return value


def prepare(matrix, source, binary, receipt_path, receipt_sha, out):
    require(not out.exists() and not out.is_symlink(), "output already exists")
    source = source.resolve(strict=True)
    binary = binary.resolve(strict=True)
    destination = out.resolve()
    require(not destination.is_relative_to(source), "output is inside source")
    require(not destination.is_relative_to(HERE), "output is inside audit/freeze")
    receipt_blob = receipt_path.read_bytes()
    require(sha(receipt_blob) == receipt_sha, "reviewed artifact receipt SHA differs")
    receipt = json.loads(receipt_blob)
    head, tree, binary_sha = receipt["head"], receipt["source_tree"], receipt["binary_sha256"]
    for field, size in [(head, 40), (tree, 40), (binary_sha, 64), (receipt["artifact_zip_sha256"], 64)]:
        require(isinstance(field, str) and len(field) == size and all(c in "0123456789abcdef" for c in field), "artifact association digest invalid")
    for field in ["own_run_id", "job_id", "artifact_id"]:
        require(type(receipt[field]) is int and receipt[field] > 0, "own artifact association missing: " + field)
    require(sha(binary.read_bytes()) == binary_sha, "actual binary SHA differs from reviewed artifact")
    count, gitlinks = verify_source(source, head, tree)
    original, blobs, commit = frozen(matrix)
    hashes = {}
    for path in original["source_sha256"]:
        require((source / path).read_bytes() == git(source, "show", head + ":" + path), "bound source bytes differ: " + path)
        hashes[path] = sha((source / path).read_bytes())
    for identity, entry in original["inputs"].items():
        blob = (source / entry["repository_path"]).read_bytes() if "repository_path" in entry else entry["text"].encode()
        require(sha(blob) == entry["sha256"], "frozen input bytes differ: " + identity)
    lineage = dict(original_freeze_commit=commit, original_manifest_sha256=sha(blobs["manifest.json"]), original_collector_sha256=sha(blobs["collect.py"]), reviewed_cli_artifact_receipt_sha256=receipt_sha, reviewed_cli_artifact_zip_sha256=receipt["artifact_zip_sha256"], reviewed_cli_binary_sha256=binary_sha, reviewed_cli_head=head, reviewed_cli_source_tree=tree, own_run_id=receipt["own_run_id"], job_id=receipt["job_id"], artifact_id=receipt["artifact_id"], tracked_source_blobs_verified=count, gitlinks_not_recursively_verified=gitlinks, preparation_executed_cli=False, accepted_integration=False)
    value = rebound_manifest(original, head, tree, hashes, binary_sha, lineage)
    manifest_blob = (json.dumps(value, indent=2) + "\n").encode()
    destination.mkdir(parents=True, exist_ok=False)
    (destination / "manifest.json").write_bytes(manifest_blob)
    (destination / "collect.py").write_bytes(blobs["collect.py"])
    (destination / "original-manifest.json").write_bytes(blobs["manifest.json"])
    (destination / "reviewed-cli-artifact-receipt.json").write_bytes(receipt_blob)
    return dict(matrix=matrix, source=head, source_tree=tree, manifest_sha256=sha(manifest_blob), collector_sha256=sha(blobs["collect.py"]), binary_sha256=binary_sha, preparation_executed_cli=False, accepted_integration=False)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--matrix", choices=FREEZES, required=True)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--reviewed-artifact-receipt", type=Path, required=True)
    parser.add_argument("--reviewed-artifact-receipt-sha256", required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(prepare(args.matrix, args.source, args.binary, args.reviewed_artifact_receipt, args.reviewed_artifact_receipt_sha256, args.out)))


if __name__ == "__main__":
    main()
