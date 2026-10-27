"""API acceptance: auth, bounded input, concurrent admission and real sandbox."""
import asyncio
import base64
from contextlib import asynccontextmanager
from concurrent.futures import ThreadPoolExecutor
import hashlib
import importlib
import json
import os
import re
from pathlib import Path
import subprocess
import sys
import tempfile
import time
from types import SimpleNamespace
import unittest
from unittest.mock import AsyncMock, patch

from fastapi import HTTPException
from fastapi.testclient import TestClient
import httpx

api = importlib.import_module("services.rest_api.app")
TOKEN = "test-only-" + "a" * 64
AUTH = {"Authorization": "Bearer " + TOKEN}


class ApiTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.cfg = api.Settings(mode="worker", keys={"test": hashlib.sha256(TOKEN.encode()).hexdigest()},
            origins=["https://client.example"], state=str(Path(self.tmp.name) / "admission.sqlite"))
        self.client = TestClient(api.create_app(self.cfg))

    def test_disabled_and_missing_credentials_never_execute(self):
        self.assertEqual(TestClient(api.create_app(api.Settings())).post("/run", json={"script": "inspect v1"}).status_code, 503)
        with patch.object(api, "run_engine", new_callable=AsyncMock) as engine:
            for headers in ({}, {"Authorization": "Bearer wrong"}):
                response = self.client.post("/run", headers=headers, content=b"x" * 100_000)
                self.assertEqual(response.status_code, 401)
            engine.assert_not_called()

    def test_config_rejects_unsafe_or_incomplete_environment(self):
        base = {"KERO_API_MODE": "gateway", "KERO_API_KEYS": json.dumps(self.cfg.keys),
                "KERO_API_UPSTREAM": "https://worker.example"}
        for bad in ("http://worker.example", "https://user:secret@worker.example", "https://worker.example/path", "https://worker.example?x=1"):
            with patch.dict(os.environ, {**base, "KERO_API_UPSTREAM": bad}, clear=True):
                self.assertEqual(api.Settings.from_env().mode, "disabled")
        with patch.dict(os.environ, base, clear=True):
            self.assertEqual(api.Settings.from_env().mode, "gateway")
        with patch.dict(os.environ, {**base, "KERO_API_KEYS": "[]"}, clear=True):
            self.assertEqual(api.Settings.from_env().mode, "disabled")

    def test_input_limits_and_no_script_echo_in_errors(self):
        cases = ({"script": "lang de\nwarte 1s"}, {"script": "quest start x"},
                 {"script": "extract v1 v2 hexane 1mol stages 999999"},
                 {"script": "transport v1 steps 999999"}, {"script": "titrate v1 NaOH 1M 1mL until ph 7 max 999999"},
                 {"script": "distil v1 v2 1 stages 999999"},
                 {"script": "wait 1e309s"}, {"script": "wait 720h\nwait 720h\nwait 720h\nwait 1s"},
                 {"script": "wait 2592001"}, {"script": "wait 43201minutes"},
                 {"script": "stir v1 500rpm 721hours"}, {"script": "wait inf"},
                 {"script": "wait 1s\n" * 129}, {"script": "add v1 SECRET 1mol\x00"},
                 {"script": "inspect v1", "path": "/etc/passwd"}, {"script": 5})
        for body in cases:
            response = self.client.post("/run", headers=AUTH, json=body)
            self.assertEqual(response.status_code, 422, body)
            self.assertNotIn("SECRET", response.text)
        self.assertEqual(self.client.post("/run", headers={**AUTH, "Content-Type": "application/json"},
                                         content=b" " * (api.BODY_LIMIT + 1)).status_code, 413)
        self.assertEqual(self.client.post("/run", headers=AUTH, content="inspect v1").status_code, 415)

    def test_supported_cli_commands_and_bounded_long_simulation_are_admitted(self):
        scripts = [
            "register lv3\nlook v1\nexplain v1",
            "evaporate v1 0.5\nregulate v1 1bar 100mL\nmix v1 1 v2 1 into v3",
            "extract v1 v2 hexane 20mL stages 4\ndistil v1 v2 10kJ stages 3",
            "titrate v1 NaOH 0.01M 0.1mL until ph 7 max 1200\n"
            "titrate v2 NaOH 0.01M 0.1mL until ph 8.7 max 1200",
            "titrate v1 KMnO4 0.02M 0.1mL until pe >= 8 max 200",
            "titrate v1 KMnO4 0.02M 0.1mL until colour persists",
            "transport v1 v2 v3 from v4 to v5 steps 15 courant 0.5",
            "wait 48h\nstir v1 500rpm 2hours",
        ]
        for script in scripts:
            with self.subTest(script=script):
                api.validate_script(script)
        script = "register lv3\ntitrate v1 NaOH 1M 1mL until ph 7 max 10"
        result = {"execution_status": "completed", "exit_status": 0, "observations": [],
                  "diagnostics": "", "elapsed_seconds": .001,
                  "script_sha256": hashlib.sha256(script.encode()).hexdigest(), "engine": {}}
        with patch.object(api, "check_resources"), patch.object(api, "run_engine", new=AsyncMock(return_value=result)) as engine:
            response = self.client.post("/run", headers=AUTH, json={"script": script})
            self.assertEqual(response.status_code, 200)
            engine.assert_awaited_once_with(self.cfg, script)

    def test_every_core_verb_and_alias_has_api_admission(self):
        # Read the engine's public grammar inventory, not another hand-copied
        # allowlist. A new engine verb must obtain an explicit API contract.
        source = Path("crates/kerotakis-core/src/script.rs").read_text()
        inventory = source.split("pub const VERBS:", 1)[1].split("];", 1)[0]
        examples = re.findall(r'\("([^"\n]+)", "([^"\n]+)"\)', inventory)
        self.assertGreater(len(examples), 30)
        aliases = set()
        for name in ("VERB_SYNONYMS", "VERB_SPELLINGS"):
            section = source.split("pub const " + name + ":", 1)[1].split("];", 1)[0]
            aliases.update(re.findall(r'"([^"\n]+)"', section))
        self.assertLessEqual({verb for verb, _ in examples} | aliases, api.VERBS)
        for verb, example in examples:
            with self.subTest(verb=verb):
                api.validate_script(example)
        for example in ("stock water 100mL", "stock blue_watercolour@1 10g",
                        "remove v2", "discard v1", "smell", "waft v1",
                        "zoom", "zoom v1", "voltmeter v1 v2"):
            api.validate_script(example)
        sessions = source.split("pub const SESSION_WORDS:", 1)[1].split("];", 1)[0]
        self.assertEqual(set(re.findall(r'"([^"\n]+)"', sessions)) - api.VERBS,
                         {"species", "help", "structure", "identify", "coverage"})
        self.assertLessEqual(set(re.findall(r'"([^"\n]+)"', sessions)),
                             api.VERBS | api.UNAVAILABLE_SESSION_WORDS)

    def test_spaced_wait_units_and_comments_preserve_duration_budgets(self):
        for script in ("wait 60 seconds", "wait 43200 minutes", "wait 720 hours",
                       "wait 720 hours\n" * 3,
                       "# lesson\nwait 2 min # " + "note " * 40,
                       "inspect v1 # inspection"):
            with self.subTest(script=script):
                api.validate_script(script)
        refused = ("wait 43201 minutes", "wait 721 hours",
                   "wait 720 hours\n" * 3 + "wait 1 seconds",
                   "wait 720 hours # wait 0\n" * 3 + "wait 1 seconds",
                   "wait 60s min", "wait 60 seconds extra", "wait 60 61s",
                   "wait 60 seconds#notacomment", "wait 60s#notacomment",
                   "wait 1e309 seconds", "wait -1 seconds", "# only comment")
        with patch.object(api, "run_engine", new_callable=AsyncMock) as engine:
            for script in refused:
                with self.subTest(script=script):
                    response = self.client.post("/run", headers=AUTH, json={"script": script})
                    self.assertEqual(response.status_code, 422)
            engine.assert_not_called()

    def test_multiword_identity_aliases_do_not_shift_loop_accounting(self):
        # The engine's blessed generated table is the single source of spellings.
        fixture = json.loads(Path(
            "crates/kerotakis-core/tests/fixtures/input-alias-coverage-20261027.json").read_text())
        aliases = {tuple(row["alias"].lower().split()): row["canonical"]
                   for row in fixture["aliases"] if len(row["alias"].split()) > 1}
        self.assertEqual(aliases, api.MULTIWORD_INGREDIENT_ALIASES)
        for script in ("stock sodium chloride 1mol", "stock carbon dioxide 1mol",
                       "stock Acetic Acid 1mol",
                       "titrate v1 sodium chloride 1M 1mL until ph 7 max 20",
                       "extract v1 v2 carbon dioxide 1mol stages 2"):
            api.validate_script(script)
        for script in ("titrate v1 sodium chloride 1M 1mL until ph 7 max 2001",
                       "titrate v1 sodium chloride 1M 1mL until ph 7 max 2000\n" * 3,
                       "extract v1 v2 carbon dioxide 1mol stages 17",
                       "extract v1 v2 carbon dioxide 1mol stages 2 stages 999999"):
            with self.subTest(script=script), self.assertRaises(HTTPException) as error:
                api.validate_script(script)
            self.assertEqual(error.exception.status_code, 422)

    def test_capabilities_match_enforced_policy_and_session_noops_refuse(self):
        with patch.object(api, "run_engine", new_callable=AsyncMock) as engine:
            response = self.client.get("/capabilities")
            self.assertEqual(response.status_code, 200)
            policy = response.json()
            self.assertEqual(set(policy["commands"]), api.VERBS)
            self.assertEqual(policy["limits"]["aggregate_loop_work"], api.MAX_LOOP_WORK)
            self.assertEqual(policy["limits"]["child_wall_seconds"], api.WALL_SECONDS)
            self.assertEqual(policy["limits"]["child_cpu_seconds"], api.CPU_SECONDS)
            self.assertEqual(response.headers["cache-control"], "no-store")
            for verb in api.UNAVAILABLE_SESSION_WORDS:
                response = self.client.post("/run", headers=AUTH, json={"script": verb + " SECRET"})
                self.assertEqual(response.status_code, 422)
                self.assertEqual(response.json()["detail"], "Command 1 is unavailable in batch JSON execution")
                self.assertNotIn("SECRET", response.text)
            for script in ("remove", "discard /etc/passwd", "stock water", "stock water 1mol extra",
                           "voltmeter v1", "voltmeter v1 v2 extra", "zoom v1 extra", "inspect /etc/passwd"):
                self.assertEqual(self.client.post("/run", headers=AUTH, json={"script": script}).status_code, 422)
            engine.assert_not_called()

    def test_solver_loop_limits_cannot_be_bypassed_by_suffixes_or_aggregation(self):
        scripts = [
            "extract v1 v2 hexane 1mol stages 0",
            "extract v1 v2 hexane 1mol stages 999999",
            "distil v1 v2 .5 stages 2 stages 999999",
            "titrate v1 NaOH 1M 1mL until ph 7 max 2 max 999999",
            "titrate v1 NaOH 1M 1mL until ph 7 max 2001",
            "titrate v1 NaOH 1M 1mL until ph 7 max 2000\n" * 3,
            "transport v1 v2 from v3 to v4 steps 257",
            "transport v1 from v2 to v3 steps 1 steps 999999",
            "transport v1 v2 v3 from v4 to v5 steps 256\n" * 6,
            "transport v1 from v2 to v3 steps 1 courant nan",
            "transport v1 from v2 to v3 steps 1 courant 1e999",
            "extract v1 v2 hexane 1mol stages " + "9" * 1000,
            "register lv4", "register lv3 extra", "explain /etc/passwd",
            "save /etc/passwd", "load /etc/passwd", "lang de", "quest start x",
            "wait 720h\nwait 720h\nwait 720h\nstir v1",
        ]
        with patch.object(api, "run_engine", new_callable=AsyncMock) as engine:
            for script in scripts:
                with self.subTest(script=script):
                    response = self.client.post("/run", headers=AUTH, json={"script": script})
                    self.assertEqual(response.status_code, 422)
                    self.assertNotIn(script, response.text)
            engine.assert_not_called()

    def test_signal_termination_is_neither_startup_nor_chemistry_refusal(self):
        for code in (-9, -11, -24):
            with self.subTest(code=code), self.assertLogs("kerotakis.api", level="WARNING") as logs:
                with self.assertRaises(HTTPException) as error:
                    api.build_result(self.cfg, "add v1 SECRET 1mol", time.monotonic(),
                                     "completed", b'{"partial":', b"SECRET private/path", code, "/private/input.lab")
                self.assertEqual(error.exception.status_code, 503)
                self.assertEqual(error.exception.detail,
                    {"code": "engine_terminated", "signal": -code, "cause": "not_determined"})
                self.assertEqual(len(logs.output), 1)
                self.assertTrue(logs.output[0].endswith("Engine terminated: signal=" + str(-code)))
                self.assertNotIn("SECRET", logs.output[0])
        # Ordinary engine refusals, malformed output, bootstrap errors and
        # supervisor-enforced timeouts retain distinct existing contracts.
        result = api.build_result(self.cfg, "inspect", time.monotonic(), "completed",
                                  b'{"partial":true}\n', b"kero: fixture refusal", 1, "/input.lab")
        self.assertEqual(result["execution_status"], "refused")
        for stdout, stderr, code, expected in ((b'bad json', b'', 0, 502),
                                               (b'', b'sandbox: seccomp', 1, 503)):
            with self.assertRaises(HTTPException) as error:
                api.build_result(self.cfg, "inspect", time.monotonic(), "completed", stdout, stderr, code, "/input.lab")
            self.assertEqual(error.exception.status_code, expected)
        result = api.build_result(self.cfg, "inspect", time.monotonic(), "timeout", b'',
                                  b'Execution limit reached', None, "/input.lab")
        self.assertEqual(result["execution_status"], "timeout")
        self.assertEqual(result["physical_verdict"], "not_assigned")

    def test_openapi_health_and_cors(self):
        schema = self.client.get("/openapi.json").json()
        self.assertIn("HTTPBearer", schema["components"]["securitySchemes"])
        self.assertIn("script", schema["paths"]["/run"]["post"]["requestBody"]["content"]["application/json"]["schema"]["properties"])
        health = self.client.get("/health")
        self.assertEqual(health.headers["cache-control"], "no-store")
        self.assertNotIn(self.tmp.name, health.text)
        for origin, allowed in (("https://client.example", True), ("https://evil.example", False)):
            response = self.client.options("/run", headers={"Origin": origin,
                "Access-Control-Request-Method": "POST", "Access-Control-Request-Headers": "Authorization,Content-Type"})
            self.assertEqual(response.headers.get("access-control-allow-origin") == origin, allowed)

    def test_atomic_concurrency_and_quotas_across_instances(self):
        stores = [api.Admission(self.cfg) for _ in range(8)]
        def acquire(store):
            try:
                return store.acquire("test")
            except HTTPException as exc:
                return exc.status_code
        with ThreadPoolExecutor(max_workers=8) as pool:
            results = list(pool.map(acquire, stores))
        leases = [value for value in results if isinstance(value, str)]
        self.assertEqual(len(leases), 1)
        self.assertEqual(results.count(429), 7)
        stores[0].release(leases[0])
        self.cfg.minute = 2
        lease = stores[0].acquire("test")
        stores[0].release(lease)
        with self.assertRaises(HTTPException) as err:
            stores[0].acquire("test")
        self.assertEqual(err.exception.status_code, 429)

    def test_admission_outage_fails_closed(self):
        self.cfg.state = str(Path(self.tmp.name) / "missing" / "state")
        with self.assertRaises(HTTPException) as err:
            api.Admission(self.cfg).acquire("test")
        self.assertEqual(err.exception.status_code, 503)

    def test_low_disk_refuses_before_execution(self):
        with patch.object(api.os, "getloadavg", return_value=(0, 0, 0)), \
             patch.object(api.Path, "read_text", return_value="MemAvailable: 10485760 kB"), \
             patch.object(api.shutil, "disk_usage", return_value=SimpleNamespace(free=500 * 1024 ** 2)):
            with self.assertRaises(HTTPException) as err:
                api.check_resources(self.cfg)
        self.assertEqual(err.exception.status_code, 503)

    def test_daily_quota_survives_store_recreation(self):
        self.cfg.daily = 1
        store = api.Admission(self.cfg)
        lease = store.acquire("test")
        store.release(lease)
        with self.assertRaises(HTTPException) as err:
            api.Admission(self.cfg).acquire("test")
        self.assertEqual(err.exception.status_code, 429)

    def test_chunked_body_cannot_bypass_limit(self):
        async def body():
            yield b" " * 10_000
            yield b" " * 10_001
        async def exercise():
            async with httpx.AsyncClient(transport=httpx.ASGITransport(app=api.create_app(self.cfg)), base_url="http://test") as client:
                return await client.post("/run", headers={**AUTH, "Content-Type": "application/json"}, content=body())
        self.assertEqual(asyncio.run(exercise()).status_code, 413)

    def test_lease_released_after_engine_failure_and_resource_refusal(self):
        with patch.object(api, "check_resources"), patch.object(api, "run_engine", new=AsyncMock(side_effect=HTTPException(503, "failure"))):
            self.assertEqual(self.client.post("/run", headers=AUTH, json={"script": "inspect v1"}).status_code, 503)
        store = api.Admission(self.cfg)
        lease = store.acquire("test")
        store.release(lease)
        with patch.object(api, "check_resources", side_effect=HTTPException(503, "resources")), patch.object(api, "run_engine", new_callable=AsyncMock) as engine:
            self.assertEqual(self.client.post("/run", headers=AUTH, json={"script": "inspect v1"}).status_code, 503)
            engine.assert_not_called()

    def test_gateway_deadline_bounds_total_drip_response_and_closes_stream(self):
        self.cfg.mode, self.cfg.upstream = "gateway", "https://worker.example"
        closed = []
        class Drip(httpx.AsyncByteStream):
            async def __aiter__(self):
                yield b'{'
                await asyncio.sleep(10)
                yield b'}'
            async def aclose(self):
                closed.append(True)
        async def handle(request):
            return httpx.Response(200, stream=Drip())
        original_client, original_timeout = httpx.AsyncClient, asyncio.timeout
        with patch.object(api.httpx, "AsyncClient", side_effect=lambda **kwargs:
                original_client(transport=httpx.MockTransport(handle), **kwargs)), \
             patch.object(api.asyncio, "timeout", side_effect=lambda seconds:
                original_timeout(.01 if seconds == 50 else seconds)):
            with self.assertRaises(HTTPException) as error:
                asyncio.run(api.forward(self.cfg, TOKEN, "inspect v1"))
        self.assertEqual(error.exception.status_code, 502)
        self.assertTrue(closed)

    def test_gateway_passes_token_only_to_fixed_destination_and_refuses_redirects(self):
        self.cfg.mode, self.cfg.upstream = "gateway", "https://worker.example"
        seen = []
        async def handle(request):
            seen.append(request)
            return httpx.Response(302, headers={"Location": "https://evil.example"})
        original = httpx.AsyncClient
        with patch.object(api.httpx, "AsyncClient", side_effect=lambda **kwargs: original(transport=httpx.MockTransport(handle), **kwargs)):
            response = self.client.post("/run", headers=AUTH, json={"script": "inspect v1"})
        self.assertEqual(response.status_code, 502)
        self.assertEqual(len(seen), 1)
        self.assertEqual(str(seen[0].url), "https://worker.example/run")
        self.assertEqual(seen[0].headers["Authorization"], AUTH["Authorization"])


class RedisTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.socket = str(Path(cls.tmp.name) / "redis.sock")
        cls.server = subprocess.Popen(["redis-server", "--port", "0", "--unixsocket", cls.socket,
            "--save", "", "--appendonly", "no", "--maxmemory", "32mb", "--maxmemory-policy", "noeviction"],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        for _ in range(100):
            if Path(cls.socket).exists():
                return
            time.sleep(.02)
        cls.server.terminate()
        raise RuntimeError("Redis fixture did not start")

    @classmethod
    def tearDownClass(cls):
        cls.server.terminate()
        cls.server.wait(timeout=5)
        cls.tmp.cleanup()

    def cli(self, *args):
        return subprocess.check_output(["redis-cli", "-s", self.socket, "--raw", *map(str, args)], text=True).strip()

    def test_atomic_shared_slots_and_principal_quota(self):
        self.cli("FLUSHDB")
        def acquire(n):
            return self.cli("EVAL", api.REDIS_ACQUIRE, 3, "{test}:leases", "{test}:user:minute",
                "{test}:user:day", "lease-" + str(n), 1, 2, 5, 90)
        with ThreadPoolExecutor(max_workers=8) as pool:
            results = list(pool.map(acquire, range(8)))
        self.assertEqual(results.count("1"), 1)
        self.assertEqual(results.count("2"), 7)
        self.cli("DEL", "{test}:leases")
        self.assertEqual(acquire(9), "1")
        self.cli("DEL", "{test}:leases")
        self.assertEqual(acquire(10), "3")
        self.assertEqual(self.cli("ZCARD", "{test}:leases"), "0")

    def test_shared_store_total_deadline_bounds_drip_and_release(self):
        cfg = api.Settings(redis_url="https://redis.example", redis_token="private-test-token")
        closed = []
        class Drip(httpx.AsyncByteStream):
            async def __aiter__(self):
                yield b'{"result":'
                await asyncio.sleep(10)
                yield b'1}'
            async def aclose(self):
                closed.append(True)
        async def handle(request):
            return httpx.Response(200, stream=Drip())
        original_client, original_timeout = httpx.AsyncClient, asyncio.timeout
        with patch.object(api.httpx, "AsyncClient", side_effect=lambda **kwargs:
                original_client(transport=httpx.MockTransport(handle), **kwargs)), \
             patch.object(api.asyncio, "timeout", side_effect=lambda seconds:
                original_timeout(.01 if seconds == 3 else seconds)):
            with self.assertRaises(HTTPException) as error:
                asyncio.run(api.RedisAdmission(cfg).acquire("test"))
            self.assertEqual(error.exception.status_code, 503)
            asyncio.run(api.RedisAdmission(cfg).release("expired-lease"))
        self.assertEqual(len(closed), 2)

    def test_shared_rest_outage_and_redirect_fail_closed(self):
        cfg = api.Settings(redis_url="https://redis.example", redis_token="private-test-token")
        original = httpx.AsyncClient
        async def handle(request):
            self.assertEqual(str(request.url), cfg.redis_url)
            return httpx.Response(302, headers={"Location": "https://evil.example"})
        with patch.object(api.httpx, "AsyncClient", side_effect=lambda **kwargs: original(transport=httpx.MockTransport(handle), **kwargs)):
            with self.assertRaises(HTTPException) as err:
                asyncio.run(api.RedisAdmission(cfg).acquire("test"))
        self.assertEqual(err.exception.status_code, 503)


class PackagingTests(unittest.TestCase):
    def test_same_project_gateway_mount_and_static_payload_preservation(self):
        import importlib.util
        with tempfile.TemporaryDirectory() as directory:
            payload = Path(directory)
            (payload / "index.html").write_text("preserved static app")
            artifacts = payload / "native"
            artifacts.mkdir()
            binary = Path("/usr/bin/true").read_bytes()
            (artifacts / "kero").write_bytes(binary)
            (artifacts / "manifest.json").write_text(json.dumps({"source_head": "a" * 40,
                "binary_sha256": hashlib.sha256(binary).hexdigest(), "cargo_lock_sha256": "b" * 64}))
            subprocess.run([sys.executable, "tools/package-rest-api.py", directory, str(artifacts)], check=True)
            self.assertEqual((payload / "index.html").read_text(), "preserved static app")
            config = json.loads((payload / "vercel.json").read_text())
            self.assertEqual(config["rewrites"][0]["source"], "/api/:path*")
            self.assertIn("headers", config)
            spec = importlib.util.spec_from_file_location("packaged_gateway", payload / "api/index.py")
            module = importlib.util.module_from_spec(spec)
            sys.modules[spec.name] = module
            self.addCleanup(sys.modules.pop, spec.name, None)
            settings = {"KERO_API_MODE": "vercel", "VERCEL": "1", "KERO_API_REDIS_URL": "https://redis.example",
                        "KERO_API_REDIS_TOKEN": "test-only-redis-credential",
                        "KERO_API_KEYS": json.dumps({"test": hashlib.sha256(TOKEN.encode()).hexdigest()})}
            with patch.dict(os.environ, settings, clear=True):
                spec.loader.exec_module(module)
            client = TestClient(module.app)
            self.assertEqual(client.get("/api/health").json()["mode"], "vercel")
            self.assertEqual(set(client.get("/api/capabilities").json()["commands"]), api.VERBS)
            self.assertRegex(client.get("/api/health").json()["revision"], r"^[a-f0-9]{40}(-dirty)?$")
            schema = client.get("/api/openapi.json").json()
            self.assertEqual(schema["servers"][0]["url"], "/api")
            self.assertEqual(client.post("/api/run", json={"script": "inspect v1"}).status_code, 401)
            binding = {"snapshot_id": "snap_fixture", "engine": json.loads((artifacts / "manifest.json").read_text())}
            for name in ("vm_job", "sandbox"):
                binding[name + "_sha256"] = hashlib.sha256((payload / "api/assets" / (name + ".runtime")).read_bytes()).hexdigest()
            binding_path = payload / "api/assets/microvm.json"
            with patch.dict(os.environ, {**settings, "KERO_API_EXECUTOR": "microvm", "KERO_API_SNAPSHOT": "snap_fixture"}, clear=True):
                binding_path.write_text(json.dumps(binding))
                self.assertEqual(module.Settings.from_env().mode, "vercel")
                for field in ("snapshot_id", "vm_job_sha256", "sandbox_sha256", "engine"):
                    tampered = {**binding, field: "incorrect"}
                    binding_path.write_text(json.dumps(tampered))
                    self.assertEqual(module.Settings.from_env().mode, "disabled", field)


class VmJobUnitTests(unittest.TestCase):
    def test_cpu_limit_requires_trusted_usage_and_matching_signal(self):
        job = importlib.import_module("services.rest_api.vm_job")
        self.assertEqual((job.CPU_SECONDS, job.WALL_SECONDS), (api.CPU_SECONDS, api.WALL_SECONDS))
        self.assertEqual((job.CPU_SECONDS, job.WALL_SECONDS), (12, 18))
        for status, code, user, system, expected in (
                ("completed", -9, 11.95, .07, True),
                ("completed", -9, 11.90, .01, False),
                ("completed", -11, 12.1, .1, False),
                ("completed", 0, 12.1, .1, False),
                ("output_limit", -9, 12.1, .1, False),
                ("timeout", None, 12.1, .1, False)):
            with self.subTest(status=status, code=code, user=user):
                usage = SimpleNamespace(ru_utime=user, ru_stime=system)
                self.assertEqual(job.cpu_limit_reached(status, code, usage), expected)


class SandboxTests(unittest.IsolatedAsyncioTestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.binary = Path(cls.tmp.name) / "fixture"
        source = cls.binary.with_suffix(".c")
        source.write_text(r'''
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <stdlib.h>
#include <sys/socket.h>
#include <signal.h>
#include <sys/resource.h>
int main(int argc,char **argv) {
  if(argc!=4 || strcmp(argv[1],"run") || !strstr(argv[2],"/input.lab") || strcmp(argv[3],"--json")) return 4;
  FILE *f=fopen(argv[2],"r"); char text[2000]={0}; fread(text,1,sizeof(text)-1,f); fclose(f);
  if(strstr(text,"999s")){ if(fork()==0) sleep(10); sleep(10); return 0; }
  if(strstr(text,"998s")){ for(;;) puts("{\"large\":\"abcdefghijklmnopqrstuvwxyz\"}"); }
  if(strstr(text,"997s")){ fputs("kero: fixture refusal\n",stderr); return 1; }
  if(strstr(text,"996s")){ puts("{\"bad\":NaN}"); return 0; }
  if(strstr(text,"995s")){ fputs("bwrap: fixture startup failure\n",stderr); return 1; }
  if(strstr(text,"993s")){ fputs("{\"partial\":",stdout); fflush(stdout); raise(SIGKILL); }
  if(strstr(text,"992s")){ struct rlimit cpu,mem; getrlimit(RLIMIT_CPU,&cpu); getrlimit(RLIMIT_AS,&mem);
    printf("{\"cpu_soft\":%llu,\"cpu_hard\":%llu,\"address_space\":%llu}\n",
      (unsigned long long)cpu.rlim_cur,(unsigned long long)cpu.rlim_max,(unsigned long long)mem.rlim_max); return 0; }
  if(strstr(text,"994s")){ printf("{\"network_denied\":%s,\"fork_denied\":%s,\"parent_signal_denied\":%s}\n",
     socket(AF_INET,SOCK_STREAM,0)<0?"true":"false",fork()<0?"true":"false",kill(getppid(),0)<0?"true":"false"); return 0; }
  char net[200]={0}; readlink("/proc/self/ns/net",net,sizeof(net)-1);
  FILE *host=fopen("/etc/passwd","r"); int readable=host!=NULL; if(host) fclose(host);
  printf("{\"host_passwd_visible\":%s,\"credential_present\":%s,\"network_namespace\":\"%s\"}\n",
    readable?"true":"false",getenv("KERO_API_KEYS")?"true":"false",net);
  return 0;
}''')
        subprocess.run(["cc", "-O0", str(source), "-o", str(cls.binary)], check=True)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def cfg(self):
        return api.Settings(binary=str(self.binary), engine={"binary_sha256": hashlib.sha256(self.binary.read_bytes()).hexdigest()})

    async def test_filesystem_credentials_and_network_namespace_isolation(self):
        with patch.dict(os.environ, {"KERO_API_KEYS": "must-not-reach-child"}):
            result = await api.run_engine(self.cfg(), "inspect v1")
        observation = result["observations"][0]
        self.assertFalse(observation["host_passwd_visible"])
        self.assertFalse(observation["credential_present"])
        self.assertNotEqual(observation["network_namespace"], os.readlink("/proc/self/ns/net"))

    async def test_deadline_kills_descendants_holding_pipes(self):
        started = time.monotonic()
        with patch.object(api, "WALL_SECONDS", .2):
            result = await api.run_engine(self.cfg(), "wait 999s")
        self.assertEqual(result["execution_status"], "timeout")
        self.assertLess(time.monotonic() - started, 2)

    async def test_output_exhaustion_and_chemistry_refusal_are_distinct(self):
        result = await api.run_engine(self.cfg(), "wait 998s")
        self.assertEqual(result["execution_status"], "output_limit")
        result = await api.run_engine(self.cfg(), "wait 997s")
        self.assertEqual(result["execution_status"], "refused")
        self.assertEqual(result["physical_verdict"], "not_assigned")

    async def test_cancellation_during_startup_reaps_process(self):
        original = asyncio.create_subprocess_exec
        spawned = []
        ready = asyncio.Event()
        async def delayed(*args, **kwargs):
            proc = await original(*args, **kwargs)
            spawned.append(proc)
            ready.set()
            await asyncio.sleep(.1)
            return proc
        with patch.object(api.asyncio, "create_subprocess_exec", side_effect=delayed):
            task = asyncio.create_task(api.run_engine(self.cfg(), "wait 999s"))
            await ready.wait()
            task.cancel()
            with self.assertRaises(asyncio.CancelledError):
                await task
        self.assertIsNotNone(spawned[0].returncode)

    async def test_binary_identity_and_sandbox_fail_closed(self):
        cfg = self.cfg()
        cfg.engine["binary_sha256"] = "0" * 64
        with self.assertRaises(HTTPException) as err:
            await api.run_engine(cfg, "inspect v1")
        self.assertEqual(err.exception.status_code, 503)

    async def test_nonfinite_output_and_bootstrap_failure_are_not_science_refusals(self):
        for script, expected in (("wait 996s", 502), ("wait 995s", 503)):
            with self.assertRaises(HTTPException) as err:
                await api.run_engine(self.cfg(), script)
            self.assertEqual(err.exception.status_code, expected)
        cfg = self.cfg()
        cfg.binary = "/usr/bin/false"
        cfg.engine["binary_sha256"] = hashlib.sha256(Path(cfg.binary).read_bytes()).hexdigest()
        with self.assertRaises(HTTPException) as err:
            await api.run_engine(cfg, "inspect v1")
        self.assertEqual(err.exception.status_code, 503)

    async def test_bootstrap_diagnostics_do_not_log_engine_output_or_script(self):
        with self.assertLogs("kerotakis.api", level="WARNING") as logs:
            with self.assertRaises(HTTPException) as err:
                await api.run_engine(self.cfg(), "wait 995s")
        self.assertEqual(err.exception.status_code, 503)
        self.assertEqual(len(logs.output), 1)
        self.assertTrue(logs.output[0].endswith("restriction=unclassified"))
        self.assertNotIn("995", logs.output[0])

    async def test_direct_serverless_sandbox_denies_host_reads_network_fork_and_parent_signals(self):
        cfg = self.cfg()
        cfg.mode = "vercel"
        result = await api.run_engine(cfg, "inspect v1")
        self.assertFalse(result["observations"][0]["host_passwd_visible"])
        self.assertFalse(result["observations"][0]["credential_present"])
        result = await api.run_engine(cfg, "wait 994s")
        self.assertTrue(all(result["observations"][0].values()))

    async def test_direct_serverless_deadline_and_output_limit(self):
        cfg = self.cfg()
        cfg.mode = "vercel"
        with patch.object(api, "WALL_SECONDS", .2):
            result = await api.run_engine(cfg, "wait 999s")
        self.assertEqual(result["execution_status"], "timeout")
        result = await api.run_engine(cfg, "wait 998s")
        self.assertEqual(result["execution_status"], "output_limit")

    async def test_direct_engine_with_explicit_matching_elf_loader(self):
        cfg = self.cfg()
        cfg.mode = "vercel"
        cfg.loader = "/lib64/ld-linux-x86-64.so.2"
        cfg.libraries = "/lib/x86_64-linux-gnu"
        result = await api.run_engine(cfg, "inspect v1")
        self.assertEqual(result["execution_status"], "completed")
        self.assertFalse(result["observations"][0]["host_passwd_visible"])

    async def test_actual_child_limits_and_signal_termination(self):
        for mode in ("worker", "vercel"):
            cfg = self.cfg()
            cfg.mode = mode
            with self.subTest(mode=mode):
                result = await api.run_engine(cfg, "wait 992s")
                self.assertEqual(result["observations"], [{"cpu_soft": 12, "cpu_hard": 12,
                                                          "address_space": 1024 ** 3}])
        # Direct execution (also used inside the Vercel VM) preserves the
        # native signal code; Bubblewrap is a separate supervising process.
        cfg = self.cfg()
        cfg.mode = "vercel"
        with self.assertRaises(HTTPException) as error:
            await api.run_engine(cfg, "wait 993s")
        self.assertEqual(error.exception.status_code, 503)
        self.assertEqual(error.exception.detail,
            {"code": "engine_terminated", "signal": 9, "cause": "not_determined"})

    async def test_microvm_refusal_and_deadline_destroy_the_isolated_job(self):
        from vercel import sandbox
        from vercel import api as vercel_api
        cfg = self.cfg()
        cfg.mode, cfg.executor, cfg.snapshot = "vercel", "microvm", "snap_fixture"
        envelope = dict(binary_sha256=cfg.engine["binary_sha256"], status="completed", code=0,
            stdout=base64.b64encode(b'{"answer":42}\n').decode(), stderr="")
        box = SimpleNamespace(fs=SimpleNamespace(write_text=AsyncMock()),
            run_process=AsyncMock(return_value=SimpleNamespace(returncode=0, stdout=json.dumps(envelope))))
        destroyed = []
        @asynccontextmanager
        async def vm(**kwargs):
            self.assertEqual(kwargs["env"], {})
            self.assertEqual(kwargs["ports"], [])
            try:
                yield box
            finally:
                destroyed.append(True)
        @asynccontextmanager
        async def session():
            yield
        with patch.object(sandbox, "create_sandbox", side_effect=vm), patch.object(vercel_api, "session", session):
            result = await api.run_engine(cfg, "inspect v1")
            self.assertEqual(result["observations"], [{"answer": 42}])
            envelope["binary_sha256"] = "0" * 64
            box.run_process.return_value.stdout = json.dumps(envelope)
            with self.assertRaises(HTTPException) as refused:
                await api.run_engine(cfg, "inspect v1")
            self.assertEqual(refused.exception.status_code, 503)
            original_timeout = asyncio.timeout
            async def blocked(*args, **kwargs):
                await asyncio.sleep(100)
            box.run_process.side_effect = blocked
            with patch.object(api.asyncio, "timeout", side_effect=lambda seconds: original_timeout(.01)):
                with self.assertRaises(HTTPException) as expired:
                    await api.run_engine(cfg, "inspect v1")
            self.assertEqual(expired.exception.status_code, 503)
        self.assertEqual(len(destroyed), 3)


if __name__ == "__main__":
    unittest.main()
