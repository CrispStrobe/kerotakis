"""Add the disabled-by-default native FastAPI service to a Vercel payload.

Usage: python tools/package-rest-api.py EXISTING_WEB_PAYLOAD [NATIVE_ARTIFACTS]
The GitHub Pages payload stays static; only the Vercel copy gets Python code.
"""
import json
import hashlib
from pathlib import Path
import shutil
import subprocess
import sys

root = Path(__file__).resolve().parent.parent
payload = Path(sys.argv[1]).resolve(strict=True)
if not (payload / "index.html").is_file() or (payload / "api").exists():
    raise SystemExit("Expected an existing static payload without API files")
service = root / "services/rest_api"
source = (service / "app.py").read_text()
revision = subprocess.check_output(["git", "-C", str(root), "rev-parse", "HEAD"], text=True).strip()
for relative in ("services/rest_api/app.py", "services/rest_api/worker.py",
                 "services/rest_api/sandbox.py", "services/rest_api/requirements.txt", "tools/package-rest-api.py"):
    committed = subprocess.run(["git", "-C", str(root), "show", "HEAD:" + relative],
                               stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    if committed.returncode or committed.stdout != (root / relative).read_bytes():
        revision += "-dirty"
        break
entry = '''os.environ.setdefault("KERO_API_MODE", "vercel")
_cfg = Settings.from_env()
if _cfg.mode != "vercel":
    _cfg = Settings()
_api = create_app(_cfg)
app = FastAPI(docs_url=None, redoc_url=None, openapi_url=None)
app.mount("/api", _api)
'''.replace('_api = create_app(_cfg)', '_cfg.revision = ' + repr(revision) + '\n_api = create_app(_cfg)')
if source.count("app = create_app()\n") != 1:
    raise SystemExit("Service entrypoint changed; review packaging")
(payload / "api").mkdir()
(payload / "api/index.py").write_text(source.replace("app = create_app()\n", entry))
assets = payload / "api/assets"
assets.mkdir()
for filename in ("worker", "sandbox"):
    (assets / (filename + ".runtime")).write_bytes((service / (filename + ".py")).read_bytes())
if len(sys.argv) > 2:
    artifacts = Path(sys.argv[2]).resolve(strict=True)
    manifest = json.loads((artifacts / "manifest.json").read_text())
    binary = (artifacts / "kero").read_bytes()
    if (hashlib.sha256(binary).hexdigest() != manifest["binary_sha256"]
            or binary[:6] != b"\x7fELF\x02\x01" or binary[18:20] != b"\x3e\x00"):
        raise SystemExit("Invalid native executable")
    (assets / "lib").mkdir(parents=True)
    for name, digest in manifest.get("libraries", {}).items():
        if Path(name).name != name or hashlib.sha256((artifacts / "lib" / name).read_bytes()).hexdigest() != digest:
            raise SystemExit("Invalid native library")
        shutil.copyfile(artifacts / "lib" / name, assets / "lib" / name)
    if manifest.get("loader"):
        if manifest["loader"] not in manifest.get("libraries", {}):
            raise SystemExit("Unverified native loader")
        (assets / "lib" / manifest["loader"]).chmod(0o755)
    shutil.copyfile(artifacts / "kero", assets / "kero")
    (assets / "kero").chmod(0o755)
    shutil.copyfile(artifacts / "manifest.json", assets / "manifest.json")
(payload / "requirements.txt").write_bytes((service / "requirements.txt").read_bytes())
config = json.loads((root / "web/vercel.json").read_text())
config["rewrites"] = [{"source": "/api/:path*", "destination": "/api/index.py"}]
config["functions"] = {"api/index.py": {"maxDuration": 30, "memory": 2048,
    "includeFiles": "api/assets/**"}}
(payload / "vercel.json").write_text(json.dumps(config, indent=2) + "\n")
