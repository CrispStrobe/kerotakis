"""Offline replay verification; never launches the application.

Checks bounded represented observables against immutable original forecasts.
Expected invalid refusals, incomplete author protocols, model qualifications and
corrected followups remain separate. A successful verifier is not a claim that
all fifty physical expectations or post-abort atomicity were established.
"""
import argparse
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
import subprocess

SUITE = Path(__file__).resolve().parent
REPO = SUITE.parents[1]
INVALID = {
    "A04": "source and target vessel are the same",
    "A05": "fraction must be within 0..=1",
    "A12": "headspace", "A13": "pressure", "A18": "time",
    "A20": "irradiance", "A22": "duration", "A25": "not empty",
    "B04": "positive", "B05": "positive", "B07": "temperature",
    "B08": "finite", "B09": "finite",
}
PROTOCOL = {"B13": "mix", "B14": "mix", **{f"B{i:02}": "particles" for i in range(19, 26)},
            "B13C": "same", "B14C": "same"}
QUALIFIED = {"A11", "A19", "B11", "B12", "B16", "B17", "B18", "B20", "B21", "B22", "B23", "B24", "B25", "B14"}


def strict(text):
    def floating(raw):
        value = float(raw)
        if not math.isfinite(value):
            raise ValueError("nonfinite JSON number: " + raw)
        return value
    return json.loads(text, parse_float=floating,
                      parse_constant=lambda x: (_ for _ in ()).throw(ValueError(x)))


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def confined(base, name):
    path = (base / name).resolve()
    if not path.is_relative_to(base.resolve()):
        raise ValueError("artifact escapes evidence directory: " + name)
    return path


def near(a, b, rel=1e-8, absolute=0.0):
    return abs(a-b) <= max(absolute, rel*max(abs(a), abs(b)))


def events(rows, kind=None):
    return [e for r in rows for e in r.get("events", []) if kind is None or e.get("event") == kind]


def measurements(rows, instrument):
    return [e["value"] for e in events(rows, "measured") if e["instrument"] == instrument]


def vessel(row, index=0):
    return next(v for v in row["bench"]["vessels"] if v["id"] == index)


def stock(v, species, phase=None):
    return sum(p["moles"] for p in v["contents"] if p["species"] == species and (phase is None or p["phase"] == phase))


def observable_contract(key, rows):
    """Partial observable claims only; no inferred total ledger from unknown species."""
    base = key[:3]
    m = measurements(rows, "balance")
    t = measurements(rows, "thermometer")
    pressure = measurements(rows, "pressure_gauge")
    ph = measurements(rows, "ph_meter") or measurements(rows, "ph")
    final = rows[-1] if rows else None
    assertions = []
    def note(label, passed):
        assertions.append(dict(claim=label, passed=bool(passed)))
    def masses(expected):
        note("represented measured masses", len(m) == len(expected) and all(near(a,b) for a,b in zip(m, expected)))
    if base == "A01": masses([80,0])
    elif base == "A02": masses([20,60,20,60])
    elif base == "A03":
        masses([80]); note("receiver temperature interior and near40C", len(t)==1 and 15<t[0]<65 and abs(t[0]-40)<1)
        note("donor empty", not vessel(final)["contents"])
    elif base == "A06": masses([40,0])
    elif base == "A07": masses([0,0])
    elif base == "A08":
        note("filtration transfers dissolved represented contents", not vessel(final)["contents"] and stock(vessel(final,1),"water")>0 and not any(p["phase"]=="solid" for p in vessel(final,1)["contents"]))
    elif base == "A09":
        masses([1,1]); note("only iron is magnetic receiver inventory", stock(vessel(final,1),"Fe","solid")>0 and all(p["species"]=="Fe" for p in vessel(final,1)["contents"]))
    elif base == "A10": masses([25,0])
    elif base == "A11": note("sealed pressure rises",len(pressure)==2 and pressure[1]>pressure[0])
    elif base == "A14": note("open ambient pressure",len(pressure)==1 and near(pressure[0],101.325))
    elif base == "A15": note("two approximately one-mole water masses",len(m)==2 and all(abs(a-18.01528)<.001 for a in m))
    elif base == "A16":
        note("third stock draw explicitly refused",bool(events(rows,"stock_exhausted")))
        note("accepted two moles retained",near(stock(vessel(final),"water"),2))
    elif base in {"A17","A19","A21","B06","B15"}:
        op = {"A17":"electrolyse","A19":"irradiate","A21":"wait","B06":None,"B15":"wait"}[base]
        indexes=[i for i,r in enumerate(rows) if r["operator"]["op"]==op or (base=="B06" and r["operator"]["op"] in {"heat","cool"})]
        note("operation executes",bool(indexes))
        for i in indexes:
            before,after=vessel(rows[i-1]),vessel(rows[i])
            fields=["contents","temperature"] + ([] if base=="A19" or key=="B15C" else ["elapsed_seconds"])
            note("represented physical state unchanged at operation",all(before.get(f)==after.get(f) for f in fields))
        if key=="B15C": note("tiny positive wait is separate control",True)
    elif base == "A23":
        masses([60,60]);note("donor order thermal agreement",len(t)==2 and abs(t[0]-t[1])<=.01)
    elif base == "A24":
        masses([0]); discards=[r for r in rows if r["operator"]["op"]=="discard"]
        note("second discard leaves donor empty",len(discards)==2 and not vessel(discards[0])["contents"] and not vessel(discards[1])["contents"])
    elif base in {"B01","B02"}:
        note("equal energy temperature agreement",len(t)==2 and abs(t[0]-t[1])<(.01 if base=="B01" else .05))
        if base=="B01": masses([100,100])
    elif base == "B03":
        note("heat/cool thermal roundtrip",len(t)==2 and abs(t[0]-t[1])<.05);masses([100,100])
    elif base == "B10":
        masses([50]);note("freezing plateau and coexistence",len(t)==2 and abs(t[-1])<.1 and stock(vessel(final),"water","liquid")>0 and stock(vessel(final),"water","solid")>0)
    elif base == "B11":
        note("boiling plateau",len(t)==2 and abs(t[-1]-100)<.5)
        note("bounded explicit evaporation",len(m)==2 and 0<m[-1]<m[0] and any(e.get("species")=="water" and e.get("moles",0)>0 for e in events(rows,"gas_evolved")))
    elif base == "B12": note("ethanol warms more",len(t)==2 and 20<t[0]<t[1]<78)
    elif key == "B13D":
        masses([20,0]);note("mixed temperature interior",len(t)==1 and 20<t[0]<80)
    elif key == "B14D":
        masses([20,20]);note("both mixed temperatures interior",len(t)==2 and all(20<a<80 for a in t))
    elif base in {"B16","B17","B18"}:
        note("acid/base direction",len(ph)==1 and ({"B16":6<ph[0]<8,"B17":ph[0]<3,"B18":ph[0]>11}[base]))
    elif base == "B19":
        note("AgCl majority precipitate remains",.0009<stock(vessel(final),"AgCl","solid")<.00101)
        if key.endswith("C"): note("filter mass sum",len(m)==3 and near(m[0],m[1]+m[2]))
    elif base == "B20":
        note("positive carbonate gas identification",any(e.get("event")=="gas_tested" and e.get("positive") is True for e in events(rows)))
        if key.endswith("C"):note("pressure increases before consuming assay",len(pressure)==3 and pressure[1]>pressure[0])
    elif base == "B21": note("bounded positive copper displacement",0<stock(vessel(final),"Cu","solid")<=.001*(1+1e-8))
    elif base == "B22": note("reverse displacement absent",stock(vessel(final),"Fe","solid")==0 and near(stock(vessel(final),"Cu","solid"),.001))
    elif base == "B23": note("substantial saturation residue",stock(vessel(final),"NaCl","solid")*58.443>10)
    elif base == "B24":
        conductivity=measurements(rows,"conductivity_meter") or measurements(rows,"conductivity")
        note("dilution increases mass",len(m)==2 and m[1]>m[0])
        note("dilution lowers conductivity",len(conductivity)==2 and 0<conductivity[1]<conductivity[0])
    elif base == "B25":
        v=vessel(final); oxygen=stock(v,"O2");peroxide=stock(v,"H2O2")
        note("peroxide oxygen stoichiometric bound",0<=oxygen<=.0005 and near(.001-peroxide,2*oxygen,rel=1e-6,absolute=1e-15))
    return assertions


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence",type=Path)
    parser.add_argument("--validation",type=Path,required=True)
    parser.add_argument("--validation-logs",type=Path,help="Directory containing the exact referenced full-validation logs")
    parser.add_argument("--baseline",action="store_true",help="Record known pre-repair failures without nonzero exit")
    parser.add_argument("--require-followups",action="store_true")
    args=parser.parse_args()
    checks=[]; assessments=[]
    def check(name, passed, **details): checks.append(dict(name=name,passed=bool(passed),**details))
    validation=strict(args.validation.read_text())
    receipt=strict((args.evidence/"execution.json").read_text())
    check("successful validation binds replay binary and source",validation.get("passed") is True and all(s["exit_code"]==0 for s in validation["stages"]) and validation["commit"]==receipt["binary_source_commit"] and validation["binary_sha256"]==receipt["binary_sha256"])
    check("validation stages nonempty unique",bool(validation["stages"]) and len({s["name"] for s in validation["stages"]})==len(validation["stages"]))
    check("full scientific validation stages present", {"thermo","codex","cea","core","phreeqc","cli","wasm","replay"}.issubset({stage["name"] for stage in validation["stages"]}))
    check("source binding present",bool(validation.get("source_hashes")))
    for name, expected in validation.get("source_hashes",{}).items():
        data=subprocess.check_output(["git","show",validation["commit"]+":"+name],cwd=REPO)
        check("source digest: "+name,hashlib.sha256(data).hexdigest()==expected)
    for stage in validation["stages"]:
        check("validation log digest: "+stage["name"],digest(confined(args.validation_logs or args.validation.parent,stage["log"]))==stage["log_sha256"])
    review={}
    for name in ["part-a-baseline-review.json","part-b-baseline-review.json"]:
        document=strict((SUITE/name).read_text())
        for c in document["cases"]: review[c["id"]]=dict(classification=c.get("classification",c.get("status")),judgment=c.get("judgment",c.get("notes",[])))
    def suite(directory,evidence,receipt,followup=False):
        freeze=strict((directory/"freeze.json").read_text()); predictions=strict((directory/"predictions.json").read_text())
        check(str(directory.name)+" forecast identity",digest(directory/"predictions.json")==freeze["predictions_sha256"]==receipt["predictions_sha256"])
        for name,expected in {**freeze.get("parts",{}),**freeze.get("source_manifests",{})}.items():check("frozen part: "+name,digest(directory/name)==expected)
        check("unique frozen cases",len(predictions)==freeze["count"] and len({c["id"] for c in predictions})==len(predictions))
        check("exact case/mode coverage",sorted((r["id"],r["mode"]) for r in receipt["runs"])==sorted((c["id"],m) for c in predictions for m in ["text","json"]))
        for case in predictions:
            key=case["id"];script=directory/"scripts"/(key+".lab")
            check(key+" script unchanged",script.read_text()==case["script"] and (not case.get("sha256") or digest(script)==case["sha256"]))
            runs=[r for r in receipt["runs"] if r["id"]==key]
            for run in runs:
                command=run["command"]
                expected_command=[receipt["binary"],"run",command[2]] + (["--json"] if run["mode"]=="json" else [])
                check(key+" "+run["mode"]+" exact replay command",command==expected_command and command[2].endswith("/"+str(directory.relative_to(REPO))+"/scripts/"+key+".lab"))
                for stream in ["stdout","stderr"]:check(key+" "+run["mode"]+" "+stream+" digest",digest(confined(evidence,run[stream]))==run[stream+"_sha256"])
            jr=next(r for r in runs if r["mode"]=="json")
            rows=[strict(line) for line in confined(evidence,jr["stdout"]).read_text().splitlines()]
            error=confined(evidence,jr["stderr"]).read_text()
            check(key+" modes same process outcome",len({r["exit_code"] for r in runs})==1)
            check(key+" sequential output",[r["output_sequence"] for r in rows]==list(range(len(rows))))
            check(key+" finite nonnegative represented state",all(math.isfinite(v["temperature"]) and v["temperature"]>0 and math.isfinite(v["pressure"]) and v["pressure"]>=0 and all(math.isfinite(p["moles"]) and p["moles"]>=0 for p in v["contents"]) for r in rows for v in r["bench"]["vessels"]))
            check(key+" finite available readings",all(math.isfinite(e["value"]) for e in events(rows,"measured")))
            if key in INVALID:
                category="expected_invalid_input_refusal";check(key+" explicit expected refusal",jr["exit_code"]==1 and INVALID[key].lower() in error.lower())
                claims=[]; limits=["CLI abort does not expose a post-refusal snapshot; atomicity and solver nonadvancement are not established by this replay."]
            elif key in PROTOCOL:
                category="author_protocol_error";check(key+" original protocol error retained",jr["exit_code"]==1 and PROTOCOL[key].lower() in error.lower())
                claims=observable_contract(key,rows);limits=["Partial observations do not satisfy the complete original protocol. Corrected followups are separate evidence."]
            else:
                check(key+" completed protocol",jr["exit_code"]==0)
                expected_rows=len([line for line in case["script"].splitlines() if line.strip() and not line.lstrip().startswith(("#","register"))])
                check(key+" complete operation frames",len(rows)==expected_rows)
                claims=observable_contract(key,rows)
                category="qualified_observable_agreement" if key[:3] in QUALIFIED else "bounded_observable_agreement"
                limits=["Only the listed observable claims are checked; no complete scientific validation or solver nonadvancement inferred."]
                if key=="A24":limits.append("CLI bench snapshots omit waste inventory: exactly-once waste accounting is not established here.")
                if key=="B14D":limits.append("Interior temperatures alone do not validate receiver/glass energy attribution or donor order invariance.")
                if key[:3] in QUALIFIED:limits.append("Model, reaction energy, kinetics, density or coverage qualifications remain; see preserved manual review and emitted support metadata.")
            if category in {"bounded_observable_agreement","qualified_observable_agreement"} and (jr["exit_code"]!=0 or any(not c["passed"] for c in claims)):
                category="unmet_observable_expectation"
            if key[:3]=="B21" and rows:
                text_run=next(r for r in runs if r["mode"]=="text")
                text=confined(evidence,text_run["stdout"]).read_text().lower()
                remaining_copper=sum(p["moles"] for p in vessel(rows[-1])["contents"] if p["species"] in {"Cu+","Cu+1","Cu+2","CuSO4"} and p["phase"]=="aqueous")
                claims.append(dict(claim="copper exhaustion narration does not overstate remaining copper inventory",passed=not (remaining_copper>1e-12 and "all the copper has plated out" in text)))
            if category in {"bounded_observable_agreement", "qualified_observable_agreement"} and any(not c["passed"] for c in claims):
                category = "unmet_observable_expectation"
            for claim in claims:check(key+" "+claim["claim"],claim["passed"])
            if not claims and key not in INVALID:limits.append("No physical claim checked: refusal/protocol integrity only.")
            assessments.append(dict(id=key,original_id=case.get("original_id",key),kind="corrected_followup" if followup else "original_frozen_case",full_forecast_established=False,category=category,forecast=case.get("physical_expectation",case.get("original_physical_expectation")),exit_code=jr["exit_code"],claims=claims,limitations=limits,historical_baseline_review=review.get(key[:3]),emitted_model_qualifications=[e for e in events(rows) if e.get("event") in {"not_yet_modeled","solver_failed"}],measurement_support=[e.get("model_support") for e in events(rows,"measured")]))
    suite(SUITE,args.evidence,receipt)
    follow=args.evidence/"followups"
    if follow.exists():
        fr=strict((follow/"execution.json").read_text())
        check("followups bind same validated binary/source",fr["binary_sha256"]==receipt["binary_sha256"] and fr["binary_source_commit"]==receipt["binary_source_commit"])
        suite(SUITE/"followups",follow,fr,True)
    elif args.require_followups:check("required corrected followups present",False)
    result=dict(passed=all(c["passed"] for c in checks),policy="A verifier pass is bounded artifact/observable agreement, never all50 expectations met. Original author errors, expected invalid refusals and corrected followups remain separate.",binary_source_commit=receipt["binary_source_commit"],binary_sha256=receipt["binary_sha256"],validation_sha256=digest(args.validation),review_sha256={n:digest(SUITE/n) for n in ["part-a-baseline-review.json","part-b-baseline-review.json"]},counts=dict(Counter(a["category"] for a in assessments if a["kind"]=="original_frozen_case")),checks=checks,cases=assessments)
    (args.evidence/"verification.json").write_text(json.dumps(result,indent=2)+"\n")
    print(json.dumps({k:v for k,v in result.items() if k not in {"checks","cases"}},indent=2))
    if not result["passed"] and not args.baseline:raise SystemExit(1)


if __name__=="__main__":main()
