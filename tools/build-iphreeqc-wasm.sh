#!/usr/bin/env bash
# Build IPhreeqc as a WebAssembly module (Track B in PLAN.md).
#
# Produces an Emscripten build of the vendored IPhreeqc with the string-based
# C API exported, ready to be driven from a thin JS bridge alongside the
# kerotakis-wasm (Track A) module. Requires the Emscripten SDK (emcmake/emmake
# on PATH).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/vendor/iphreeqc"
BUILD="${1:-$ROOT/target/iphreeqc-wasm}"

if [ ! -f "$SRC/CMakeLists.txt" ]; then
    echo "vendor/iphreeqc missing — run: git submodule update --init" >&2
    exit 1
fi

EXPORTED_FUNCTIONS='["_CreateIPhreeqc","_DestroyIPhreeqc","_LoadDatabaseString","_RunString","_GetErrorString","_GetSpeciesDeltaH","_SetOutputFileOn","_SetErrorFileOn","_SetLogFileOn","_SetDumpFileOn","_SetSelectedOutputFileOn","_SetSelectedOutputStringOn","_SetOutputStringOn","_GetOutputString","_GetOutputStringLineCount","_GetSelectedOutputStringLineCount","_GetSelectedOutputStringLine","_malloc","_free"]'

case "${IPHREEQC_BASIC_MODE:-disabled}" in
    disabled)
        BASIC_CMAKE_ARGS=(-DIPHREEQC_WITH_BASIC=OFF -DIPHREEQC_WITH_MY_BASIC=OFF)
        ;;
    my-basic)
        BASIC_CMAKE_ARGS=(
            -DIPHREEQC_WITH_BASIC=OFF
            -DIPHREEQC_WITH_MY_BASIC=ON
            -DKEROTAKIS_MY_BASIC_DIR="$ROOT/vendor/my-basic"
        )
        ;;
    *)
        echo "IPHREEQC_BASIC_MODE must be 'disabled' or 'my-basic'" >&2
        exit 2
        ;;
esac

# -fexceptions is required: PHREEQC uses C++ exceptions for error control
# flow, and Emscripten disables exception catching by default.
emcmake cmake -S "$SRC" -B "$BUILD" \
    -DCMAKE_BUILD_TYPE=MinSizeRel \
    -DBUILD_SHARED_LIBS=OFF \
    -DIPHREEQC_ENABLE_MODULE=OFF \
    "${BASIC_CMAKE_ARGS[@]}" \
    -DBUILD_TESTING=OFF \
    -DCMAKE_CXX_FLAGS="-fexceptions" \
    -DCMAKE_C_FLAGS="-fexceptions"

cmake --build "$BUILD" --target IPhreeqc -j"$(nproc 2>/dev/null || sysctl -n hw.ncpu)"

# Link the static archive into a standalone module with the C API exported.
# (The archive name varies with the CMake configuration: libIPhreeqc.a or
# libIPhreeqcmsr.a.)
ARCHIVE="$(find "$BUILD" -maxdepth 2 -name 'libIPhreeqc*.a' | head -1)"
if [ -z "$ARCHIVE" ]; then
    echo "no libIPhreeqc*.a produced in $BUILD" >&2
    exit 1
fi

# em++ (not emcc): the archive is C++ and needs libc++ linked in.
em++ "$ARCHIVE" \
    -o "$BUILD/iphreeqc.mjs" \
    -sMODULARIZE=1 \
    -sEXPORT_ES6=1 \
    -sEXPORT_NAME=createIPhreeqc \
    -sEXPORTED_FUNCTIONS="$EXPORTED_FUNCTIONS" \
    -sEXPORTED_RUNTIME_METHODS='["ccall","cwrap","UTF8ToString","stringToNewUTF8","HEAPU8"]' \
    -sALLOW_MEMORY_GROWTH=1 \
    -sSTACK_SIZE=8388608 \
    -sFILESYSTEM=0 \
    -fexceptions \
    -O2

ls -lh "$BUILD"/iphreeqc.{mjs,wasm}
echo "OK: IPhreeqc wasm module built."
