#!/usr/bin/env bash
# Hosted build: native CLI compatible with the AL2023 Vercel Python runtime.
# Run only with adequate resources; deployment CI supplies the build container.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="${1:?usage: build-rest-native.sh NEW_ARTIFACT_DIRECTORY}"
test ! -e "$OUT"
git -C "$ROOT" diff --quiet HEAD
mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd)"
REST_BUILD_CACHE="$(mktemp -d)"
trap 'rm -rf "$REST_BUILD_CACHE"' EXIT
SOURCE_HEAD="$(git -C "$ROOT" rev-parse HEAD)"
SOURCE_TREE="$(git -C "$ROOT" rev-parse 'HEAD^{tree}')"
docker pull public.ecr.aws/amazonlinux/amazonlinux:2023
REST_BUILD_IMAGE="$(docker image inspect public.ecr.aws/amazonlinux/amazonlinux:2023 --format '{{index .RepoDigests 0}}')"
docker run --rm --cpus=2 --memory=4g --pids-limit=256 \
  -v "$ROOT:/src:ro" -v "$OUT:/out" -v "$REST_BUILD_CACHE:/build" \
  -w /src -e KEROTAKIS_GIT_REV="$SOURCE_HEAD" -e CARGO_TARGET_DIR=/build/target \
  "$REST_BUILD_IMAGE" bash -euo pipefail -c '
    dnf install -y --allowerasing curl gcc gcc-c++ cmake make git tar gzip perl python3 diffutils findutils
    curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
    source /root/.cargo/env
    cargo build --locked --release -j 2 -p kerotakis-cli
    cp /build/target/release/kero /out/kero
    strip /out/kero
    chmod 755 /out/kero
    mkdir /out/lib
    ldd /out/kero > /out/ldd.txt
    python3 - <<"PY"
from pathlib import Path
import re,shutil
for line in Path("/out/ldd.txt").read_text().splitlines():
    if "not found" in line:
        raise SystemExit("Missing native dependency")
    match=re.search(r"=> (/[^ ]+)",line)
    if match:
        path=Path(match[1])
        if path.name not in {"libc.so.6","libm.so.6","libpthread.so.0","libdl.so.2","librt.so.1"}:
            shutil.copyfile(path,Path("/out/lib")/path.name)
PY
    rustc --version > /out/rustc.txt
  '
git -C "$ROOT" diff --quiet HEAD
python3 - "$ROOT" "$OUT" "$SOURCE_HEAD" "$SOURCE_TREE" "$REST_BUILD_IMAGE" <<'PY'
from pathlib import Path
import hashlib,json,sys
root,out=map(Path,sys.argv[1:3])
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
manifest={"source_head":sys.argv[3],"source_tree":sys.argv[4],"build_image":sys.argv[5],
          "binary_sha256":sha(out/"kero"),"cargo_lock_sha256":sha(root/"Cargo.lock"),
          "libraries":{p.name:sha(p) for p in sorted((out/"lib").iterdir())},
          "rustc":(out/"rustc.txt").read_text().strip(),"target":"x86_64-unknown-linux-gnu"}
(out/"manifest.json").write_text(json.dumps(manifest,indent=2)+"\n")
PY
