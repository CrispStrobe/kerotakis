"""Add the disabled-by-default FastAPI gateway to an existing Vercel payload.

Usage: python tools/package-rest-api.py EXISTING_WEB_PAYLOAD
The GitHub Pages payload stays static; only the Vercel copy gets Python code.
"""
import json
from pathlib import Path
import sys

root = Path(__file__).resolve().parent.parent
payload = Path(sys.argv[1]).resolve(strict=True)
if not (payload / "index.html").is_file() or (payload / "api").exists():
    raise SystemExit("Expected an existing static payload without API files")
service = root / "services/rest_api"
source = (service / "app.py").read_text()
entry = '''_cfg = Settings.from_env()
if _cfg.mode != "gateway":
    _cfg = Settings()
_gateway = create_app(_cfg)
app = FastAPI(docs_url=None, redoc_url=None, openapi_url=None)
app.mount("/api", _gateway)
'''
if source.count("app = create_app()\n") != 1:
    raise SystemExit("Gateway entrypoint changed; review packaging")
(payload / "api").mkdir()
(payload / "api/index.py").write_text(source.replace("app = create_app()\n", entry))
(payload / "requirements.txt").write_bytes((service / "requirements.txt").read_bytes())
config = json.loads((root / "web/vercel.json").read_text())
config["rewrites"] = [{"source": "/api/:path*", "destination": "/api/index.py"}]
(payload / "vercel.json").write_text(json.dumps(config, indent=2) + "\n")
