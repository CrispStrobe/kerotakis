"""Linux x86_64 restrictions for a native engine inside a serverless function.

No fallback: absent Landlock ABI3 or seccomp support prevents exec. This module
is loaded before restricting filesystem access. It requires no libseccomp DSO.
"""
import ctypes
import os
from pathlib import Path
import platform


def restrict(binary, script, libraries=(), loader=None):
    if platform.machine() != "x86_64":
        raise RuntimeError("Unsupported sandbox architecture")
    libc = ctypes.CDLL(None, use_errno=True)
    libc.syscall.restype = ctypes.c_long
    abi = libc.syscall(444, 0, 0, 1)
    if abi < 3:
        raise RuntimeError("Landlock ABI3 unavailable:%d:%d" % (abi, ctypes.get_errno()))
    if libc.prctl(38, 1, 0, 0, 0):  # PR_SET_NO_NEW_PRIVS
        raise RuntimeError("Cannot restrict process privileges")

    class CapHeader(ctypes.Structure):
        _fields_ = [("version", ctypes.c_uint32), ("pid", ctypes.c_int)]

    class CapData(ctypes.Structure):
        _fields_ = [("effective", ctypes.c_uint32), ("permitted", ctypes.c_uint32),
                    ("inheritable", ctypes.c_uint32)]

    caps = (CapData * 2)()
    header = CapHeader(0x20080522, 0)
    if libc.capset(ctypes.byref(header), ctypes.byref(caps)):
        raise RuntimeError("Cannot remove process capabilities")

    class Ruleset(ctypes.Structure):
        _fields_ = [("handled_access_fs", ctypes.c_uint64)]

    class PathRule(ctypes.Structure):
        _pack_ = 1
        _fields_ = [("allowed_access", ctypes.c_uint64), ("parent_fd", ctypes.c_int32)]

    ruleset = Ruleset((1 << (16 if abi >= 5 else 15)) - 1)
    fd = libc.syscall(444, ctypes.byref(ruleset), ctypes.sizeof(ruleset), 0)
    if fd < 0:
        raise RuntimeError("Cannot create filesystem restrictions")
    try:
        paths = [(binary, 5), (script, 4)]  # EXECUTE|READ_FILE, READ_FILE.
        if loader:
            paths.append((loader, 5))
        paths += [(p, 12) for p in ("/lib", "/lib64", "/usr/lib", *libraries) if Path(p).exists()]
        for loader in ("/lib64/ld-linux-x86-64.so.2", "/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2"):
            if Path(loader).exists():
                paths.append((loader, 5))
        for path, access in paths:
            parent = os.open(path, os.O_PATH | os.O_CLOEXEC)
            try:
                rule = PathRule(access, parent)
                if libc.syscall(445, fd, 1, ctypes.byref(rule), 0):
                    raise RuntimeError("Cannot add filesystem restriction")
            finally:
                os.close(parent)
        if libc.syscall(446, fd, 0):
            raise RuntimeError("Cannot restrict filesystem")
    finally:
        os.close(fd)

    # BPF seccomp_data offsets: syscall number 0, audit architecture 4, args16.
    # Check x86_64 AND the x32 bit; syscall numbers below are only for that ABI.
    instructions = [(0x20, 0, 0, 4), (0x15, 1, 0, 0xC000003E),
                    (0x06, 0, 0, 0x80000000), (0x20, 0, 0, 0),
                    (0x45, 0, 1, 0x40000000), (0x06, 0, 0, 0x80000000)]
    allow, deny = 0x7FFF0000, 0x50000 | 1  # SECCOMP_RET_ALLOW / EPERM.

    def argument_check(number, accepted):
        block = [(0x20, 0, 0, 16)]
        for value in accepted:
            block += [(0x15, 0, 1, value), (0x06, 0, 0, allow)]
        block += [(0x06, 0, 0, deny)]
        instructions.append((0x15, 0, len(block), number))
        instructions.extend(block)

    argument_check(62, [os.getpid()])  # kill: self only.
    argument_check(234, [os.getpid()])  # tgkill: own thread group only.
    argument_check(302, [0, os.getpid()])  # prlimit64: self only.
    argument_check(157, [15, 16])  # prctl: thread names only.
    # Thread creation only. clone3 returns ENOSYS so libc can use clone.
    block = [(0x20, 0, 0, 16), (0x45, 1, 0, 0x10000),
             (0x06, 0, 0, deny), (0x06, 0, 0, allow)]
    instructions += [(0x15, 0, len(block), 56), *block,
                     (0x15, 0, 1, 435), (0x06, 0, 0, 0x50000 | 38)]
    allowed = (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
               16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
               32, 33, 34, 35, 36, 37, 38, 39, 59, 60, 63, 72, 73, 74, 75,
               78, 79, 80, 89, 96, 97, 98, 99, 100, 102, 104, 107, 108,
               110, 111, 118, 120, 131, 137, 138, 158, 186, 201, 202, 204,
               217, 218, 219, 228, 229, 230, 231, 257, 262, 267, 269, 270,
               271, 273, 318, 324, 332, 334, 436, 439)
    for number in allowed:
        instructions += [(0x15, 0, 1, number), (0x06, 0, 0, allow)]
    instructions.append((0x06, 0, 0, deny))

    class Filter(ctypes.Structure):
        _fields_ = [("code", ctypes.c_ushort), ("jt", ctypes.c_ubyte),
                    ("jf", ctypes.c_ubyte), ("k", ctypes.c_uint32)]

    class Program(ctypes.Structure):
        _fields_ = [("length", ctypes.c_ushort), ("filter", ctypes.POINTER(Filter))]

    code = (Filter * len(instructions))(*(Filter(*row) for row in instructions))
    program = Program(len(instructions), code)
    if libc.prctl(22, 2, ctypes.byref(program), 0, 0):  # PR_SET_SECCOMP/FILTER
        raise RuntimeError("Cannot restrict system calls")
