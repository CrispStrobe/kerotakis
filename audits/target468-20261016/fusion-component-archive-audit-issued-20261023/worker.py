#!/usr/bin/env python3
"""Read-only audit of the original component archive projection.

Runs on the already mounted, versioned original kernel output. It does not
import, execute, load, or inspect executable contents as code.
"""
from __future__ import annotations

import gzip
import hashlib
import json
import os
import re
import signal
import sys
import time
from pathlib import Path, PurePosixPath

# Independently copied from the external original ISSUE and frozen plan.
# No kernel owner/id, credentials, local NAS path, or source checkout is needed.
# External reference computed from the private original ISSUE.json before
# dispatch. ISSUE.json is not expected in the mounted kernel output, so this is
# reported as a reference and is never described as a mounted-file measurement.
EXPECTED_ISSUE_SHA256_REFERENCE = "74d35ff4e31e0c63223ccb46e70335b8b71007fb5c676d092ee78e669d333b6a"
EXPECTED_PLAN_SHA256 = "04e747e6f5e6a3b73ed8e4951bf2600fc9c08c95de76c7c848ca935640277c82"
EXPECTED_ORIGINAL_WORKER_SHA256 = "7b2b3d44831aa4df7a955354d730b81771103cfc19bc9db65517c3a38421886b"
EXPECTED_MANIFEST_SHA256 = "cd630e766ee6ea0382515606a0e357197f86dbc353a98f46145815a68151d7fd"
EXPECTED_SOURCE = "51bc28ddc576d587b86fab8da2ee6ea120822bc4"
EXPECTED_TREE = "826551744876447dd4c537a124c362c3bc1a124f"
EXPECTED_LOCK_SHA256 = "41bbdc743cad4758007f10c040cdacb2caf1e34ce32a4348512fb16a396300cb"
EXPECTED_ARCHIVE_COUNT = 342
EXPECTED_COMPRESSED_ARCHIVE_BYTES = 4_092_069_492
EXPECTED_MANIFEST_ROWS = 2792

MAX_SCAN_FILES = 5000
MAX_INPUT_BYTES = 5 * 1024**3
MAX_MANIFEST_BYTES = 8 * 1024**2
MAX_METADATA_BYTES = 2 * 1024**2
MAX_ARCHIVE_BYTES = 64 * 1024**2
MAX_ONE_EXECUTABLE_BYTES = 128 * 1024**2
MAX_TOTAL_DECOMPRESSED_BYTES = 64 * 1024**3
MAX_SECONDS = 20 * 60
CHUNK_BYTES = 1024 * 1024
KEY_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_.-]{0,180}$")


class AuditFailure(Exception):
    def __init__(self, code: str):
        self.code = code
        super().__init__(code)


def fail(code: str) -> None:
    raise AuditFailure(code)


PROGRESS = {
    "manifest_sha256": None,
    "source": EXPECTED_SOURCE,
    "tree": EXPECTED_TREE,
    "actual_producer_lock_sha256": None,
    "archives_observed": [],
}


def check_time(deadline: float) -> None:
    if time.monotonic() > deadline:
        fail("time_budget_exceeded")


def sha256_file(path: Path, deadline: float) -> tuple[str, int]:
    digest = hashlib.sha256()
    total = 0
    with path.open("rb") as handle:
        while True:
            check_time(deadline)
            block = handle.read(CHUNK_BYTES)
            if not block:
                break
            digest.update(block)
            total += len(block)
    return digest.hexdigest(), total


def safe_member(root: Path, name: str) -> Path:
    if not isinstance(name, str) or not name or "\\" in name or "\x00" in name:
        fail("unsafe_manifest_path")
    pure = PurePosixPath(name)
    if pure.is_absolute() or any(part in ("", ".", "..") for part in pure.parts):
        fail("unsafe_manifest_path")
    candidate = root.joinpath(*pure.parts)
    try:
        resolved = candidate.resolve(strict=True)
        resolved.relative_to(root.resolve(strict=True))
    except (OSError, ValueError):
        fail("manifest_member_missing_or_escaped")
    if candidate.is_symlink() or not resolved.is_file():
        fail("manifest_member_not_regular_file")
    return resolved


def locate_manifest(input_root: Path, deadline: float) -> tuple[Path, str, int]:
    matches: list[tuple[Path, str, int]] = []
    file_count = 0
    bytes_seen = 0
    for directory, dirs, files in os.walk(input_root, followlinks=False):
        check_time(deadline)
        here = Path(directory)
        dirs[:] = sorted(d for d in dirs if not (here / d).is_symlink())
        for filename in files:
            file_count += 1
            if file_count > MAX_SCAN_FILES:
                fail("input_file_count_limit_exceeded")
            path = here / filename
            if path.is_symlink() or not path.is_file():
                fail("input_contains_non_regular_file")
            size = path.stat().st_size
            bytes_seen += size
            if bytes_seen > MAX_INPUT_BYTES:
                fail("input_size_limit_exceeded")
            if filename == "manifest.json":
                if size > MAX_MANIFEST_BYTES:
                    continue
                digest, actual_size = sha256_file(path, deadline)
                if digest == EXPECTED_MANIFEST_SHA256:
                    matches.append((path, digest, actual_size))
    if len(matches) != 1:
        fail("exact_manifest_not_found_once")
    return matches[0]


class HashingReader:
    """Minimal read-through wrapper to hash compressed bytes consumed by gzip."""
    def __init__(self, raw, deadline: float):
        self.raw = raw
        self.digest = hashlib.sha256()
        self.count = 0
        self.deadline = deadline

    def read(self, size=-1):
        check_time(self.deadline)
        if size is None or size < 0:
            size = CHUNK_BYTES
        else:
            size = min(size, CHUNK_BYTES)
        data = self.raw.read(size)
        if data:
            self.digest.update(data)
            self.count += len(data)
        return data


def hash_and_decompress(path: Path, archive_row: dict, binary: dict,
                        deadline: float, remaining_decompressed: int) -> dict:
    declared_gzip_size = archive_row.get("bytes")
    declared_gzip_sha = archive_row.get("sha256")
    if not isinstance(declared_gzip_size, int) or not isinstance(declared_gzip_sha, str):
        fail("invalid_archive_manifest_row")
    if declared_gzip_size > MAX_ARCHIVE_BYTES or declared_gzip_size < 1:
        fail("archive_size_limit_exceeded")
    if binary.get("gzip_bytes") != declared_gzip_size or binary.get("gzip_sha256") != declared_gzip_sha:
        fail("archive_binary_metadata_disagreement")
    expected_raw_size = binary.get("bytes")
    expected_raw_sha = binary.get("sha256")
    if not isinstance(expected_raw_size, int) or not isinstance(expected_raw_sha, str):
        fail("invalid_binary_metadata")
    if expected_raw_size > MAX_ONE_EXECUTABLE_BYTES or expected_raw_size < 1:
        fail("executable_size_limit_exceeded")
    if expected_raw_size > remaining_decompressed:
        fail("total_decompressed_limit_exceeded")

    with path.open("rb") as raw:
        tracker = HashingReader(raw, deadline)
        digest = hashlib.sha256()
        decompressed = 0
        try:
            with gzip.GzipFile(fileobj=tracker, mode="rb") as stream:
                while True:
                    check_time(deadline)
                    block = stream.read(CHUNK_BYTES)
                    if not block:
                        break
                    decompressed += len(block)
                    if decompressed > expected_raw_size or decompressed > remaining_decompressed:
                        fail("decompressed_size_limit_exceeded")
                    digest.update(block)
        except AuditFailure:
            raise
        except (OSError, EOFError, gzip.BadGzipFile, zlib_error_types()) as exc:
            fail("gzip_stream_invalid")
        # Drain only to account for any bytes gzip did not consume; trailing bytes
        # are rejected so no unbound payload can hide after the gzip member.
        tail = raw.read(CHUNK_BYTES)
        if tail:
            tracker.digest.update(tail)
            tracker.count += len(tail)
            fail("gzip_trailing_bytes")
    actual_gzip_sha = tracker.digest.hexdigest()
    if tracker.count != declared_gzip_size or actual_gzip_sha != declared_gzip_sha:
        fail("compressed_archive_hash_or_length_mismatch")
    if decompressed != expected_raw_size or digest.hexdigest() != expected_raw_sha:
        fail("executable_hash_or_length_mismatch")
    return {
        "key": binary["key"],
        "package": binary["package"],
        "target": binary["target"],
        "compressed_bytes": tracker.count,
        "compressed_sha256": actual_gzip_sha,
        "executable_bytes": decompressed,
        "executable_sha256": digest.hexdigest(),
    }


def zlib_error_types():
    # Local import from the standard library only; no project or mounted-code imports.
    import zlib
    return zlib.error


def set_cpu_affinity() -> dict:
    if not hasattr(os, "sched_getaffinity") or not hasattr(os, "sched_setaffinity"):
        return {"applied": False, "reason": "unsupported"}
    allowed = sorted(os.sched_getaffinity(0))
    chosen = allowed[:2]
    if not chosen:
        return {"applied": False, "reason": "empty_affinity"}
    os.sched_setaffinity(0, set(chosen))
    return {"applied": True, "cpu_count": len(chosen)}


def run() -> dict:
    start = time.monotonic()
    deadline = start + MAX_SECONDS
    signal.signal(signal.SIGALRM, lambda *_: fail("time_budget_exceeded"))
    signal.alarm(MAX_SECONDS)
    affinity = set_cpu_affinity()
    input_root = Path("/kaggle/input").resolve(strict=True)
    if not input_root.is_dir():
        fail("input_root_missing")

    manifest_path, manifest_sha, manifest_bytes = locate_manifest(input_root, deadline)
    PROGRESS["manifest_sha256"] = manifest_sha
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if not isinstance(manifest, dict) or len(manifest) != EXPECTED_MANIFEST_ROWS:
        fail("manifest_row_count_mismatch")
    if manifest_sha != EXPECTED_MANIFEST_SHA256:
        fail("manifest_hash_mismatch")

    started_path = safe_member(manifest_path.parent, "STARTED.json")
    started_row = manifest.get("STARTED.json")
    if not isinstance(started_row, dict):
        fail("started_manifest_row_missing")
    started_sha, started_bytes = sha256_file(started_path, deadline)
    if started_sha != started_row.get("sha256") or started_bytes != started_row.get("bytes"):
        fail("started_manifest_binding_mismatch")
    started = json.loads(started_path.read_text(encoding="utf-8"))
    if started.get("worker_sha256") != EXPECTED_ORIGINAL_WORKER_SHA256:
        fail("started_worker_mismatch_external_issue")
    if started.get("runtime_source") != EXPECTED_SOURCE or started.get("runtime_tree") != EXPECTED_TREE:
        fail("started_source_or_tree_mismatch")

    plan_row = manifest.get("frozen-plan.json")
    if not isinstance(plan_row, dict):
        fail("frozen_plan_manifest_row_missing")
    if plan_row.get("bytes", MAX_METADATA_BYTES + 1) > MAX_METADATA_BYTES:
        fail("frozen_plan_size_limit_exceeded")
    plan_path = safe_member(manifest_path.parent, "frozen-plan.json")
    plan_sha, plan_bytes = sha256_file(plan_path, deadline)
    if plan_sha != EXPECTED_PLAN_SHA256 or plan_sha != plan_row.get("sha256") or plan_bytes != plan_row.get("bytes"):
        fail("frozen_plan_hash_or_length_mismatch")
    frozen_plan = json.loads(plan_path.read_text(encoding="utf-8"))
    if frozen_plan.get("source") != EXPECTED_SOURCE or frozen_plan.get("tree") != EXPECTED_TREE:
        fail("frozen_plan_source_or_tree_mismatch")
    if frozen_plan.get("dependency_lock_sha256") != EXPECTED_LOCK_SHA256:
        fail("frozen_plan_actual_lock_mismatch")

    lock_path = safe_member(manifest_path.parent, "actual-producer-Cargo.lock")
    lock_row = manifest.get("actual-producer-Cargo.lock")
    if not isinstance(lock_row, dict):
        fail("lock_manifest_row_missing")
    lock_sha, lock_bytes = sha256_file(lock_path, deadline)
    if lock_sha != EXPECTED_LOCK_SHA256 or lock_sha != lock_row.get("sha256") or lock_bytes != lock_row.get("bytes"):
        fail("actual_lock_binding_mismatch")
    PROGRESS["actual_producer_lock_sha256"] = lock_sha

    archive_names = {name for name in manifest if isinstance(name, str) and name.endswith(".gz")}
    binary_names = {name for name in manifest if isinstance(name, str) and name.endswith("-binary.json")}
    if len(archive_names) != EXPECTED_ARCHIVE_COUNT or len(binary_names) != EXPECTED_ARCHIVE_COUNT:
        fail("archive_or_binary_member_count_mismatch")
    if sum(manifest[name].get("bytes", -1) for name in archive_names) != EXPECTED_COMPRESSED_ARCHIVE_BYTES:
        fail("compressed_archive_total_mismatch")

    metadata_by_key = {}
    for name in sorted(binary_names):
        row = manifest[name]
        if not isinstance(row, dict):
            fail("binary_manifest_row_invalid")
        if row.get("bytes", MAX_METADATA_BYTES + 1) > MAX_METADATA_BYTES:
            fail("binary_metadata_size_limit_exceeded")
        path = safe_member(manifest_path.parent, name)
        raw_sha, raw_bytes = sha256_file(path, deadline)
        if raw_sha != row.get("sha256") or raw_bytes != row.get("bytes"):
            fail("binary_metadata_manifest_binding_mismatch")
        metadata = json.loads(path.read_text(encoding="utf-8"))
        key = metadata.get("key")
        if not isinstance(key, str) or not KEY_RE.fullmatch(key) or key in metadata_by_key:
            fail("binary_metadata_key_invalid_or_duplicate")
        if name != key + "-binary.json":
            fail("binary_metadata_filename_key_mismatch")
        if metadata.get("runtime_source") != EXPECTED_SOURCE or metadata.get("runtime_tree") != EXPECTED_TREE:
            fail("binary_runtime_source_or_tree_mismatch")
        if metadata.get("actual_producer_lock_sha256") != EXPECTED_LOCK_SHA256:
            fail("binary_actual_lock_mismatch")
        metadata_by_key[key] = metadata

    expected_archive_names = {key + ".gz" for key in metadata_by_key}
    if archive_names != expected_archive_names:
        fail("archive_binary_key_sets_differ")

    observed = []
    total_compressed = 0
    total_decompressed = 0
    for key in sorted(metadata_by_key):
        check_time(deadline)
        archive_name = key + ".gz"
        row = manifest[archive_name]
        archive_path = safe_member(manifest_path.parent, archive_name)
        if row.get("bytes", MAX_ARCHIVE_BYTES + 1) > MAX_ARCHIVE_BYTES:
            fail("archive_manifest_size_limit_exceeded")
        expected_size = metadata_by_key[key].get("bytes")
        remaining = MAX_TOTAL_DECOMPRESSED_BYTES - total_decompressed
        record = hash_and_decompress(archive_path, row, metadata_by_key[key], deadline, remaining)
        total_compressed += record["compressed_bytes"]
        total_decompressed += record["executable_bytes"]
        observed.append(record)
        PROGRESS["archives_observed"] = observed.copy()

    if total_compressed != EXPECTED_COMPRESSED_ARCHIVE_BYTES:
        fail("observed_compressed_total_mismatch")
    if len(observed) != EXPECTED_ARCHIVE_COUNT:
        fail("observed_archive_count_mismatch")
    if total_decompressed > MAX_TOTAL_DECOMPRESSED_BYTES:
        fail("total_decompressed_limit_exceeded")

    return {
        "status": "PASS_ARCHIVE_BYTES_ONLY",
        "external_original_issue_sha256_reference": EXPECTED_ISSUE_SHA256_REFERENCE,
        "frozen_plan_sha256": EXPECTED_PLAN_SHA256,
        "manifest_sha256": manifest_sha,
        "manifest_bytes": manifest_bytes,
        "source": EXPECTED_SOURCE,
        "tree": EXPECTED_TREE,
        "original_started_worker_sha256": EXPECTED_ORIGINAL_WORKER_SHA256,
        "actual_producer_lock_sha256": lock_sha,
        "archive_count": len(observed),
        "compressed_archive_bytes": total_compressed,
        "decompressed_executable_bytes": total_decompressed,
        "limits": {
            "max_seconds": MAX_SECONDS,
            "max_input_files_scanned": MAX_SCAN_FILES,
            "max_input_bytes_scanned": MAX_INPUT_BYTES,
            "max_decompressed_total_bytes": MAX_TOTAL_DECOMPRESSED_BYTES,
            "chunk_bytes": CHUNK_BYTES,
        },
        "cpu_affinity": affinity,
        "archives": observed,
        "claims": {
            "mounted_original_kernel_output_only": True,
            "private_complete_raw_collection_rehashed": False,
            "native_test_binaries_executed": False,
            "source_or_test_code_imported": False,
            "provider_or_network_calls": False,
            "test_outcomes_recomputed": False,
        },
    }


def main() -> int:
    output = Path("/kaggle/working/fusion-archive-audit.json")
    try:
        result = run()
        code = 0
    except AuditFailure as exc:
        result = {
            "status": "FAIL",
            "failure_code": exc.code,
            "observed": PROGRESS,
            "claims": {
                "native_test_binaries_executed": False,
                "source_or_test_code_imported": False,
                "provider_or_network_calls": False,
            },
        }
        code = 1
    except Exception:
        # Avoid exposing paths, environment, owner/id, or runtime details.
        result = {
            "status": "FAIL",
            "failure_code": "unexpected_worker_error",
            "observed": PROGRESS,
        }
        code = 1
    try:
        output.parent.mkdir(parents=True, exist_ok=True)
        encoded = json.dumps(result, sort_keys=True, separators=(",", ":")) + "\n"
        output.write_text(encoded, encoding="utf-8")
        print(encoded, end="")
    except Exception:
        print('{"status":"FAIL","failure_code":"result_write_failed"}')
        return 1
    return code


if __name__ == "__main__":
    raise SystemExit(main())
