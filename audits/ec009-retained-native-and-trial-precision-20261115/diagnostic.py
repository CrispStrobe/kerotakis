"""Single exact retained-native diagnostic; standard library, no build/network."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import resource
import signal
import subprocess
import tarfile
import time

SOURCE = "5271b14894f68d91e472acca3aa6bae982e4712a"
TREE = "452d8dab7de58405cc0485b640203b863a27a2b6"
CACHE = "ae309879ccd707a808512fe1033692a22e7602999de1e79221dd4cb2b108aa68"
LOCK = "c2d28fbc40c3e864abc011d7c3731cf449a2a13737a3c7e4ecd89d8b5c238fd7"
COMPILER = "be69adad9137eaa0bdac1dd07e2fe286b1831fc8bad193ea27806ca0c7c8e7c6"
EXE = "9370afba7b99d8d6d100768608a7392431158f35f9c0780b042589083a65b41a"
MEMBER = "target/debug/build/kerotakis-phreeqc/92d2678191d7180f/out/redox-92d2678191d7180f"
TEST = "permanganate_oxidises_iron_in_the_ratio_the_half_equations_give"


def digest(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def validate_binding(receipt, compiler_sha, lock_sha):
    assert receipt["source"] == SOURCE
    assert receipt["tree"] == TREE
    assert receipt["sha256"] == CACHE
    assert receipt["actual_lock_sha256"] == lock_sha == LOCK
    assert receipt["toolchain"] == "1.100.0"
    assert compiler_sha == COMPILER


def copy_member(cache, destination):
    found = 0
    with tarfile.open(cache, "r|gz") as archive:
        for member in archive:
            if member.name != MEMBER:
                continue
            found += 1
            assert found == 1 and member.isfile() and 0 < member.size <= 32 * 1024**2
            content = archive.extractfile(member)
            assert content is not None
            with Path(destination).open("xb") as output:
                while True:
                    block = content.read(256 * 1024)
                    if not block:
                        break
                    output.write(block)
    assert found == 1


def save(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2)
        stream.write("\n")


def child_limits():
    resource.setrlimit(resource.RLIMIT_AS, (2 * 1024**3, 2 * 1024**3))
    resource.setrlimit(resource.RLIMIT_CPU, (30, 30))
    resource.setrlimit(resource.RLIMIT_FSIZE, (32 * 1024**2, 32 * 1024**2))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--inputs", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    # Only existing attached producer output may supply the cache. The harness
    # never downloads, uploads, copies a dataset, or selects an arbitrary exe.
    caches = list(args.inputs.glob("**/failed-source-build-cache.tar.gz"))
    assert len(caches) == 1
    cache = caches[0]
    assert digest(cache) == CACHE
    receipt = json.loads((cache.parent / "failed-source-build-cache.json").read_text())
    validate_binding(receipt, digest(cache.parent / "compiler.stdout"),
                     digest(cache.parent / "actual-producer-Cargo.lock"))
    args.out.mkdir(exist_ok=False)
    exe = args.out / "retained-positive-test"
    copy_member(cache, exe)
    assert digest(exe) == EXE
    exe.chmod(0o500)
    tmp = args.out / "tmp"
    tmp.mkdir()
    argv = [str(exe.resolve()), "--exact", TEST, "--test-threads=1", "--nocapture"]
    save(args.out / "STARTED.json", dict(source=SOURCE, tree=TREE,
         compiler_sha256=COMPILER, cache_sha256=CACHE, producer_lock_sha256=LOCK,
         binary_sha256=EXE, argv=argv, no_build=True, no_repair=True,
         no_original_continuation=True, no_qualification=True,
         cpu_seconds=30, wall_seconds=45, address_space_bytes=2*1024**3))
    env = dict(PATH=os.defpath, TZ="UTC", TMPDIR=str(tmp.resolve()),
               MALLOC_ARENA_MAX="2", OMP_NUM_THREADS="1",
               KERO_REDOX="1", KERO_READBACK="1", KERO_DUMP_INPUT="all")
    start = time.monotonic()
    timed_out = False
    with (args.out / "stdout").open("xb") as stdout, (args.out / "stderr").open("xb") as stderr:
        process = subprocess.Popen(argv, cwd=args.out, env=env, stdout=stdout,
                 stderr=stderr, preexec_fn=child_limits, start_new_session=True)
        try:
            code = process.wait(timeout=45)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(process.pid, signal.SIGKILL)
            code = process.wait(timeout=5)
    save(args.out / "COMPLETED.json", dict(exit=code, timed_out=timed_out,
         seconds=time.monotonic()-start, no_qualification=True,
         stdout_sha256=digest(args.out / "stdout"), stderr_sha256=digest(args.out / "stderr")))
    print(json.dumps(dict(exit=code, timed_out=timed_out, diagnostic_only=True)), flush=True)


if __name__ == "__main__":
    main()
