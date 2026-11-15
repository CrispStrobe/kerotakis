"""Inspect actual retained-positive trace; never qualify or infer absent rows."""
import argparse
from decimal import Decimal as D
import hashlib
import json
from pathlib import Path
import re


def inspect(root):
    started=json.loads((root/"STARTED.json").read_text())
    completed=json.loads((root/"COMPLETED.json").read_text())
    assert started["source"]=="5271b14894f68d91e472acca3aa6bae982e4712a"
    assert started["binary_sha256"]=="9370afba7b99d8d6d100768608a7392431158f35f9c0780b042589083a65b41a"
    for name in ["stdout","stderr"]:
        assert hashlib.sha256((root/name).read_bytes()).hexdigest()==completed[name+"_sha256"]
    lines=(root/"stderr").read_text().splitlines()
    input_water=None
    blocks=[]
    current=None
    for line in lines:
        match=re.match(r"\s*water\s+([0-9.eE+\-]+)",line)
        if match:
            input_water=D(match[1])
        match=re.match(r"readback: kgw_out=([0-9.eE+\-]+)",line)
        if match:
            current={"printed_solved_water_kg":D(match[1]),"input_water_kg":input_water,"rows":{}}
            blocks.append(current)
            continue
        if current is not None:
            match=re.match(r"\s+(mass_H2O|(?:Fe|Mn)\(-?\d+\))\s+([0-9.eE+\-]+)\s*$",line)
            if match:
                assert match[1] not in current["rows"]
                current["rows"][match[1]]=D(match[2])
            elif "elements=" in line:
                current=None
    candidates=[]
    for block in blocks:
        rows=block["rows"]
        # Only captured solved rows can establish this cause. The fallback
        # or the prior v1 residual cannot substitute for a coupled readback.
        if not all(k in rows for k in ["mass_H2O","Fe(2)","Fe(3)","Mn(2)","Mn(7)"]):
            continue
        water=rows["mass_H2O"]
        assert water.is_finite() and water>0
        if rows["Fe(3)"]*water <= D("1e-10") or rows["Mn(2)"]*water <= D("1e-10"):
            continue
        total_fe=sum((value*water for key,value in rows.items()if key.startswith("Fe(")),D(0))
        total_mn=sum((value*water for key,value in rows.items()if key.startswith("Mn(")),D(0))
        if abs(total_fe-D(".005"))>D("1e-10")or abs(total_mn-D(".0002"))>D("1e-10"):
            continue
        input_water=block["input_water_kg"]
        if input_water is None:
            continue
        weighted=sum((D(re.search(r"\((-?\d+)\)",key)[1])*value
                      for key,value in rows.items()if key!="mass_H2O"),D(0))
        target=D(".005")*2+D(".0002")*7
        wrong=weighted*input_water
        owned=weighted*water
        candidates.append(dict(input_water_kg=str(input_water),solved_water_kg=str(water),
             Fe_mol=str(total_fe),Mn_mol=str(total_mn),input_weighted_electron_sum=str(wrong),
             solved_weighted_electron_sum=str(owned),target=str(target),
             input_weighted_residual=str(wrong-target),solved_weighted_residual=str(owned-target),
             confirms_wrong_owner=abs(wrong-target)<=D("1e-12")and abs(owned-target)>D("1e-10")and water!=input_water))
    return dict(source=started["source"],exit=completed["exit"],readback_blocks=len(blocks),
         candidates=candidates,causal_confirmation=any(x["confirms_wrong_owner"]for x in candidates),
         absent_coupled_rows_are_not_proof=True,qualification=False,new_originals=0)


if __name__=="__main__":
    parser=argparse.ArgumentParser();parser.add_argument("root",type=Path);parser.add_argument("out",type=Path);args=parser.parse_args()
    result=inspect(args.root)
    with args.out.open("x")as f:json.dump(result,f,indent=2);f.write("\n")
    print(json.dumps(result,indent=2))
