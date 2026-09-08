#!/usr/bin/env python3
"""Independent, process-isolated chemistry probes; Python standard library only.

Run from the checkout:
    python3 tools/chemistry-audit/run.py --binary target/debug/kero --out <fresh dir>

The recorded output of the original runs is not in the repository; it is in the
evidence archive named in README.md. --out must therefore be given explicitly.
Each script and its full CLI stdout/stderr are retained. A successful process
is NOT a scientific pass. Interpret observations against the stated questions.
"""
import argparse
import collections
import hashlib
import json
import pathlib
import subprocess
import time

ROOT = pathlib.Path(__file__).resolve().parents[2]


def case(key, question, script):
    return {"id": key, "question": question,
            "script": "register lv3\n" + script.strip() + "\ninspect\n"}


CASES = [
    case("01-water", "Does pure water give a finite near-neutral baseline?", "add v1 water 1L\nmeasure v1 ph"),
    case("02-acid-dilution", "Does tenfold dilution of dilute HCl raise pH by about one?", "add v1 water 100mL\nadd v1 HCl 0.0001mol\nmeasure v1 ph\nadd v1 water 900mL\nmeasure v1 ph"),
    case("03-base-dilution", "Does tenfold dilution of dilute NaOH lower pH by about one?", "add v1 water 100mL\nadd v1 NaOH 0.0001mol\nmeasure v1 ph\nadd v1 water 900mL\nmeasure v1 ph"),
    case("04-neutral-acid-first", "Do equal strong-acid/base amounts neutralize and warm the water?", "add v1 water 100mL\nadd v1 HCl 0.005mol\nadd v1 NaOH 0.005mol\nmeasure v1 ph\nmeasure v1 temp"),
    case("05-neutral-base-first", "Does reversing neutralization order preserve the final composition and heat?", "add v1 water 100mL\nadd v1 NaOH 0.005mol\nadd v1 HCl 0.005mol\nmeasure v1 ph\nmeasure v1 temp"),
    case("06-acetate-buffer", "Does an equimolar acetate buffer resist a small acid addition?", "add v1 water 1L\nadd v1 CH3COOH 0.02mol\nadd v1 NaOAc 0.02mol\nmeasure v1 ph\nadd v1 HCl 0.002mol\nmeasure v1 ph"),
    case("07-buffer-control", "Does the same acid dose cause a much larger pH change in water?", "add v1 water 1L\nmeasure v1 ph\nadd v1 HCl 0.002mol\nmeasure v1 ph"),
    case("08-phosphate", "Are successive half/equivalence points of phosphoric acid distinct?", "add v1 water 1L\nadd v1 H3PO4 0.01mol\nadd v1 NaOH 0.005mol\nmeasure v1 ph\nadd v1 NaOH 0.005mol\nmeasure v1 ph\nadd v1 NaOH 0.005mol\nmeasure v1 ph\nadd v1 NaOH 0.005mol\nmeasure v1 ph"),
    case("09-ultradilute-acid", "Does 1e-9 mol HCl per litre stay near neutrality rather than pH 9?", "add v1 water 1L\nadd v1 HCl 0.000000001mol\nmeasure v1 ph"),
    case("10-silver-chloride", "Does near-stoichiometric dilute Ag/Cl precipitate AgCl?", "add v1 water 200mL\nadd v1 NaCl 0.002mol\nadd v1 AgNO3 0.002mol"),
    case("11-silver-reverse", "Is AgCl yield invariant to reversing reagent order?", "add v1 water 200mL\nadd v1 AgNO3 0.002mol\nadd v1 NaCl 0.002mol"),
    case("12-barium-sulfate", "Does mixing barium chloride and sodium sulfate give insoluble barite?", "add v1 water 200mL\nadd v1 BaCl2 0.002mol\nadd v1 Na2SO4 0.002mol"),
    case("13-copper-hydroxide", "Does CuSO4 plus hydroxide form a copper precipitate with an honest phase identity?", "add v1 water 200mL\nadd v1 CuSO4 0.002mol\nadd v1 NaOH 0.004mol"),
    case("14-salt-saturation", "Does excess NaCl leave solid and select a suitable high-strength model?", "add v1 water 100mL\nadd v1 NaCl 0.8mol\nexplain v1"),
    case("15-salt-dilute-again", "Does adding water redissolve saturated salt?", "add v1 water 100mL\nadd v1 NaCl 0.8mol\nadd v1 water 200mL"),
    case("16-hot-pack", "Does dissolving CaCl2 warm water?", "add v1 water 200mL\nadd v1 CaCl2 0.04mol\nmeasure v1 temp"),
    case("17-cold-pack", "Does dissolving KCl cool water?", "add v1 water 200mL\nadd v1 KCl 0.04mol\nmeasure v1 temp"),
    case("18-copper-crystals", "Does a heat/cool cycle reversibly change hydrated copper sulfate solid?", "add v1 water 100mL\nadd v1 CuSO4 25g\ninspect v1\nheat v1 15kJ\ninspect v1\ncool v1 15kJ"),
    case("19-fizz-open", "How much mass and CO2 leave an open acid/bicarbonate vessel?", "add v1 water 200mL\nadd v1 NaHCO3 0.01mol\nmeasure v1 balance\nadd v1 CH3COOH 0.012mol\nmeasure v1 balance"),
    case("20-fizz-sealed", "Does the same fizz retain gas, raise pressure and conserve closed-system mass?", "add v1 water 200mL\nadd v1 NaHCO3 0.01mol\nseal v1 250mL\nmeasure v1 balance\nadd v1 CH3COOH 0.012mol\nmeasure v1 balance\nmeasure v1 pressure"),
    case("21-fizz-regulated", "Does pressure-controlled fizz change headspace volume rather than pressure?", "add v1 water 200mL\nadd v1 NaHCO3 0.01mol\nregulate v1 1.2bar 250mL\nadd v1 CH3COOH 0.012mol\nmeasure v1 pressure\nmeasure v1 volume"),
    case("22-limewater-excess", "Can a finite initial CO2 dose precipitate chalk and an excess dissolve it?", "add v1 water 500mL\nadd v1 Ca(OH)2 0.005mol\nadd v1 CO2 0.005mol\ninspect v1\nadd v1 CO2 0.025mol"),
    case("23-zinc-copper", "Does Zn displace copper from CuSO4 with bounded stoichiometry?", "add v1 water 200mL\nadd v1 CuSO4 0.003mol\nadd v1 Zn 0.004mol"),
    case("24-copper-zinc-control", "Does copper fail to displace Zn from ZnSO4?", "add v1 water 200mL\nadd v1 ZnSO4 0.003mol\nadd v1 Cu 0.004mol"),
    case("25-magnesium-acid", "Does Mg consume two equivalents of acid per H2?", "add v1 water 200mL\nadd v1 HCl 0.006mol\nadd v1 Mg 0.002mol"),
    case("26-copper-acid-control", "Is copper in nonoxidizing HCl refused for a chemical reason?", "add v1 water 200mL\nadd v1 HCl 0.006mol\nadd v1 Cu 0.002mol"),
    case("27-daniell-cell", "Does a Zn/Cu cell report roughly 1.1 V and provenance?", "add v1 water 200mL\nadd v1 ZnSO4 0.02mol\nadd v1 Zn 0.01mol\nnew\nadd v2 water 200mL\nadd v2 CuSO4 0.02mol\nadd v2 Cu 0.01mol\ncell v1 v2"),
    case("28-electrolysis-sulfate", "Does 60 C generate H2/O2 in a 2:1 ratio?", "add v1 water 200mL\nadd v1 Na2SO4 0.5g\nelectrolyse v1 0.1A 10min"),
    case("29-electrolysis-water-control", "Does pure-water electrolysis explain its conductivity boundary?", "add v1 water 200mL\nelectrolyse v1 0.1A 10min"),
    case("30-permanganate-dilution", "Does dilution reduce permanganate absorption without losing material?", "add v1 water 100mL\nadd v1 KMnO4 0.0001mol\nlook v1\nadd v1 water 900mL\nlook v1"),
    case("31-filter-precipitate", "Does filtration preserve total material and separate AgCl from solution?", "add v1 water 200mL\nadd v1 NaCl 0.002mol\nadd v1 AgNO3 0.002mol\nnew\nfilter v1 v2"),
    case("32-evaporate-brine", "Does removing 90 percent solvent concentrate salt into crystals?", "add v1 water 200mL\nadd v1 NaCl 0.2mol\nevaporate v1 0.9"),
    case("33-distil-ethanol", "Does a low-fraction distillation enrich ethanol in the receiver?", "add v1 water 4mol\nadd v1 ethanol 0.4mol\nnew\ndistil v1 v2 0.15"),
    case("34-esterification-passive", "Does simply mixing aqueous acid/alcohol honestly distinguish kinetics from no reaction?", "add v1 water 100mL\nadd v1 CH3COOH 0.02mol\nadd v1 ethanol 0.02mol\nwait 1h"),
    case("35-esterification-explicit", "What yield and confidence does an explicitly requested esterification claim?", "add v1 water 100mL\nadd v1 CH3COOH 0.02mol\nadd v1 ethanol 0.02mol\nreact v1 esterification"),
    case("36-fermentation-control", "Does sugar alone remain unchanged over ten minutes?", "add v1 water 100mL\nadd v1 table_sugar 5g\nwait 600s"),
    case("37-fermentation-yeast", "Does adding yeast produce a conserved, explicitly bounded fermentation result?", "add v1 water 100mL\nadd v1 table_sugar 5g\nadd v1 dry_yeast 0.5g\nwait 600s"),
    case("38-magnesium-fire", "Does dry magnesium combustion consume air oxygen and increase residue mass?", "add v1 Mg 0.01mol\nmeasure v1 balance\nignite v1\nmeasure v1 balance"),
    case("39-salt-fire-control", "Does noncombustible salt give a flame test without inventing combustion?", "add v1 NaCl 0.01mol\nignite v1"),
    case("40-iron-thiocyanate", "Can the registry and solver express the classic Fe(III)/thiocyanate complex?", "add v1 water 100mL\nadd v1 FeCl3 0.0001mol\nadd v1 KSCN 0.0003mol\nlook v1"),
    case("41-ammonia-copper", "Does excess ammonia describe the copper ammine complex and colour?", "add v1 water 100mL\nadd v1 CuSO4 0.001mol\nadd v1 NH3 0.02mol\nlook v1"),
    case("42-neutralization-titration", "Does an automatic titration reach the strong-acid endpoint within one increment?", "add v1 water 200mL\nadd v1 HCl 0.002mol\ntitrate v1 NaOH 0.1M 1mL until ph 7"),
    case("43-acid-mass-ledger", "Follow-up: does adding 0.1 mol HCl increase balance mass by its full molecular mass?", "add v1 water 1L\nmeasure v1 balance\nadd v1 HCl 0.1mol\nmeasure v1 balance"),
    case("44-base-mass-ledger", "Follow-up: does adding 0.1 mol NaOH retain hydroxide mass?", "add v1 water 1L\nmeasure v1 balance\nadd v1 NaOH 0.1mol\nmeasure v1 balance"),
    case("45-yeast-mass-ledger", "Follow-up: does the balance include 0.5 g unresolved yeast solids?", "add v1 water 100mL\nmeasure v1 balance\nadd v1 dry_yeast 0.5g\nmeasure v1 balance"),
    case("46-ammonia-extended-route", "Follow-up: does a trace acetate forcing the extended database change copper/ammonia predictions?", "add v1 water 100mL\nadd v1 NaOAc 0.000000001mol\nadd v1 CuSO4 0.001mol\nadd v1 NH3 0.02mol\nlook v1"),
    case("47-limewater-wait", "Follow-up: does limewater clearing persist after the finite CO2 dose has ended?", "add v1 water 500mL\nadd v1 Ca(OH)2 0.005mol\nadd v1 CO2 0.005mol\nadd v1 CO2 0.025mol\ninspect v1\nwait 1s"),
    case("48-fizz-sealed-from-start", "Follow-up: when sealed before bicarbonate addition, is the entire added mass retained?", "add v1 water 200mL\nseal v1 250mL\nmeasure v1 balance\nadd v1 NaHCO3 0.01mol\nadd v1 CH3COOH 0.012mol\nmeasure v1 balance\nwait 1s\nmeasure v1 balance"),
    case("49-water-clock", "Follow-up: can waiting cause pure water to be characterized?", "add v1 water 1L\nwait 1s\nmeasure v1 ph"),
    case("50-sugar-ph", "Follow-up: is the pH of a nonionic sugar solution characterized?", "add v1 water 1L\nadd v1 table_sugar 5g\nmeasure v1 ph"),
]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=pathlib.Path, default=ROOT / "target/debug/kero")
    parser.add_argument("--out", required=True, type=pathlib.Path,
                        help="new, empty output directory; never reuse one, so a replay "
                             "can never overwrite an earlier run's raw evidence")
    parser.add_argument("--only", help="case id prefix")
    args = parser.parse_args()
    binary = args.binary.resolve()
    if args.out.exists() and any(args.out.iterdir()):
        parser.error('output directory is not empty; preserve earlier evidence and choose a new directory')
    args.out.mkdir(parents=True, exist_ok=True)
    # A diff digest cannot reconstruct a dirty source revision. Keep the
    # patch and new runtime/catalog files as well, without recursively copying
    # previous audit output directories into the next audit.
    source_patch = subprocess.check_output(["git", "diff", "HEAD"], cwd=ROOT)
    (args.out / "source.patch").write_bytes(source_patch)
    new_files = subprocess.check_output(
        ["git", "ls-files", "--others", "--exclude-standard", "crates", "data",
         "provenance", "lessons", "codex", "web", "Cargo.lock"], cwd=ROOT, text=True).splitlines()
    snapshots = {name: (ROOT / name).read_text() for name in new_files}
    (args.out / "new-source-files.json").write_text(json.dumps(snapshots, indent=2) + "\n")
    metadata = {"commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
                "complete_source_patch_sha256": hashlib.sha256(source_patch).hexdigest(),
                "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
                "lock_sha256": hashlib.sha256((ROOT / "Cargo.lock").read_bytes()).hexdigest(),
                "submodules": subprocess.check_output(["git", "submodule", "status"], cwd=ROOT, text=True),
                "source_diff_sha256": hashlib.sha256(subprocess.check_output(
                    ["git", "diff", "HEAD", "--", "crates", "data", "Cargo.toml", "deny.toml"], cwd=ROOT)).hexdigest(),
                "untracked_source_sha256": {
                    name: hashlib.sha256((ROOT / name).read_bytes()).hexdigest()
                    for name in subprocess.check_output(
                        ["git", "ls-files", "--others", "--exclude-standard", "crates"], cwd=ROOT, text=True).splitlines()
                    if name.endswith('.rs')},
                "cases": []}
    for spec in CASES:
        if args.only and not spec["id"].startswith(args.only):
            continue
        directory = args.out / spec["id"]
        directory.mkdir(parents=True, exist_ok=True)
        script = directory / "experiment.lab"
        script.write_text("# " + spec["question"] + "\n" + spec["script"])
        start = time.monotonic()
        try:
            process = subprocess.run([str(binary), "run", str(script.resolve()), "--json"],
                                     cwd=directory, capture_output=True, text=True, timeout=90)
            stdout, stderr, rc = process.stdout, process.stderr, process.returncode
        except subprocess.TimeoutExpired as exc:
            stdout = (exc.stdout or b"").decode() if isinstance(exc.stdout, bytes) else exc.stdout or ""
            stderr = (exc.stderr or b"").decode() if isinstance(exc.stderr, bytes) else exc.stderr or ""
            rc = "timeout"
        (directory / "stdout.ndjson").write_text(stdout)
        (directory / "stderr.txt").write_text(stderr)
        rows, invalid = [], []
        for line in stdout.splitlines():
            try:
                rows.append(json.loads(line))
            except ValueError:
                invalid.append(line)
        events = [event for row in rows for event in row.get("events", [])]
        final = next((row["bench"] for row in reversed(rows) if "bench" in row), None)
        record = {"id": spec["id"], "question": spec["question"], "exit": rc,
                  "seconds": round(time.monotonic() - start, 3), "steps": len(rows),
                  "non_json_lines": invalid, "event_counts": dict(collections.Counter(e["event"] for e in events)),
                  "diagnostics": [e for e in events if e["event"] in
                                  ("not_yet_modeled", "solver_failed", "hazard_warning")],
                  "measurements": [e for e in events if "measur" in e["event"]],
                  "final": final, "stderr": stderr}
        metadata["cases"].append(record)
        print(spec["id"], "exit=", rc, "steps=", len(rows), "events=", record["event_counts"], flush=True)
    (args.out / "summary.json").write_text(json.dumps(metadata, indent=2) + "\n")


if __name__ == "__main__":
    main()
