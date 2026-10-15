"""Exec-only child: set limits outside the threaded ASGI process, then sandbox."""
import os
from pathlib import Path
import resource
import sys

resource.setrlimit(resource.RLIMIT_AS, (1024 ** 3, 1024 ** 3))
resource.setrlimit(resource.RLIMIT_CPU, (8, 8))
resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
resource.setrlimit(resource.RLIMIT_FSIZE, (1024 ** 2, 1024 ** 2))
resource.setrlimit(resource.RLIMIT_NOFILE, (64, 64))
direct = sys.argv[1] == "--vercel"
if direct:
    if os.geteuid() == 0:
        os.write(2, b"sandbox: unprivileged execution required\n")
        os._exit(125)
    resource.setrlimit(resource.RLIMIT_NPROC, (64, 64))
    import importlib.machinery
    import types
    sandbox_path = Path(__file__).with_name("sandbox.py")
    if not sandbox_path.is_file():
        sandbox_path = sandbox_path.with_suffix(".runtime")
    module = types.ModuleType("kero_runtime_sandbox")
    importlib.machinery.SourceFileLoader(module.__name__, str(sandbox_path)).exec_module(module)
    restrict = module.restrict
    binary, script, libraries = sys.argv[2:]
    try:
        restrict(binary, script, (libraries,) if libraries else ())
        os.execv(binary, [binary, "run", script, "--json"])
    except Exception:
        os.write(2, b"sandbox: direct execution restrictions unavailable\n")
        os._exit(125)
binary, script = sys.argv[1:]
args = ["/usr/bin/bwrap", "--unshare-all", "--die-with-parent", "--cap-drop", "ALL",
        "--clearenv", "--setenv", "LANG", "C.UTF-8", "--setenv", "KERO_LANG", "en",
        "--dir", "/engine", "--ro-bind", binary, "/engine/kero",
        "--dir", "/job", "--ro-bind", script, "/job/input.lab",
        "--size", "16777216", "--tmpfs", "/tmp", "--proc", "/proc", "--dev", "/dev", "--chdir", "/job"]
for directory in ("/lib", "/lib64", "/usr/lib"):
    if Path(directory).exists():
        args += ["--ro-bind", directory, directory]
args += ["--", "/engine/kero", "run", "/job/input.lab", "--json"]
os.execv(args[0], args)
