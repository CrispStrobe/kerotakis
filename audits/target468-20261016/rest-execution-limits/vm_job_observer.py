"""Fixed Vercel microVM supervisor. No keys, shells or user-supplied paths.

The platform must start this file as root in a fresh, network-denied microVM.
Only the engine child enters the prepared root and drops privileges. The
supervisor bounds and reaps that child before emitting a bounded JSON envelope.
"""
import asyncio
import base64
import ctypes
import hashlib
import importlib.machinery
import json
import os
from pathlib import Path
import resource
import signal
import sys
import time
import types

ROOT = "/vercel/sandbox/engine"
LIMIT = 1_000_000


def child():
    if os.geteuid() != 0:
        raise RuntimeError("Root required before entering the isolated filesystem")
    module = types.ModuleType("kero_vm_seccomp")
    importlib.machinery.SourceFileLoader(module.__name__, ROOT + "/sandbox.runtime").exec_module(module)
    # Load ctypes before chroot; the child cannot load Python or arbitrary code
    # once inside. All remaining executable assets are verified deployment files.
    libc = ctypes.CDLL(None, use_errno=True)
    # Recent Python versions lazily import their structure-layout helper.
    # Resolve that trusted stdlib dependency before removing the filesystem.
    class PreloadLayout(ctypes.Structure):
        _fields_ = [("value", ctypes.c_uint64)]
    resource.setrlimit(resource.RLIMIT_AS, (1024 ** 3, 1024 ** 3))
    resource.setrlimit(resource.RLIMIT_CPU, (8, 8))
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
    resource.setrlimit(resource.RLIMIT_FSIZE, (1024 ** 2, 1024 ** 2))
    resource.setrlimit(resource.RLIMIT_NOFILE, (64, 64))
    resource.setrlimit(resource.RLIMIT_NPROC, (64, 64))
    os.chdir(ROOT)
    os.chroot(".")
    os.chdir("/")
    os.setgroups([])
    os.setgid(65534)
    os.setuid(65534)
    if libc.prctl(38, 1, 0, 0, 0):
        raise RuntimeError("Cannot restrict privileges")
    module.restrict_system_calls()
    os.execve("/lib/ld-linux-x86-64.so.2",
        ["/lib/ld-linux-x86-64.so.2", "--library-path", "/lib", "/kero", "run", "/input.lab", "--json"],
        {"LANG": "C.UTF-8", "KERO_LANG": "en", "RAYON_NUM_THREADS": "1"})


async def supervise():
    started = time.monotonic()
    manifest = json.loads(Path(ROOT + "/manifest.json").read_text())
    if hashlib.sha256(Path(ROOT + "/kero").read_bytes()).hexdigest() != manifest["binary_sha256"]:
        raise RuntimeError("Engine identity mismatch")
    for name, digest in manifest["libraries"].items():
        if Path(name).name != name or hashlib.sha256(Path(ROOT + "/lib/" + name).read_bytes()).hexdigest() != digest:
            raise RuntimeError("Engine runtime identity mismatch")
    proc = await asyncio.create_subprocess_exec(sys.executable, "-I", __file__, "--child",
        stdin=asyncio.subprocess.DEVNULL, stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE, env={"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8"},
        start_new_session=True)
    total = 0
    async def read(stream):
        nonlocal total
        chunks = []
        while chunk := await stream.read(8192):
            total += len(chunk)
            if total > LIMIT:
                raise OverflowError
            chunks.append(chunk)
        return b"".join(chunks)
    tasks = [asyncio.create_task(read(proc.stdout)), asyncio.create_task(read(proc.stderr)), asyncio.create_task(proc.wait())]
    status = "completed"
    try:
        stdout, stderr, code = await asyncio.wait_for(asyncio.gather(*tasks), 10)
    except (TimeoutError, OverflowError) as exc:
        status = "timeout" if isinstance(exc, TimeoutError) else "output_limit"
        stdout, stderr, code = b"", b"Execution limit reached", None
    finally:
        try:
            os.killpg(proc.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        for task in tasks:
            if not task.done():
                task.cancel()
        await asyncio.gather(*tasks, return_exceptions=True)
        await proc.communicate()
    usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    resource_usage = dict(user_cpu_seconds=usage.ru_utime,
                          system_cpu_seconds=usage.ru_stime, max_rss_kib=usage.ru_maxrss)
    return dict(resource_usage=resource_usage, binary_sha256=manifest["binary_sha256"], status=status, code=code, stdout=base64.b64encode(stdout).decode(),
                stderr=base64.b64encode(stderr).decode(), elapsed=time.monotonic()-started)


if __name__ == "__main__":
    if sys.argv[1:] == ["--child"]:
        child()
    elif not sys.argv[1:]:
        print(json.dumps(asyncio.run(supervise()), separators=(",", ":")))
    else:
        raise SystemExit(2)
