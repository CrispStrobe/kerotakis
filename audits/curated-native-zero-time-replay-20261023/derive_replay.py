"""Read frozen native traces; compute a bookkeeping relation, not a new solve."""
from decimal import Decimal, localcontext
from pathlib import Path
import hashlib
import json
import re

B = Path(__file__).resolve().parent


def main():
    result = json.loads((B / "result-projection.json").read_text())
    review = json.loads((B / "independent-saved-link-review.json").read_text())
    for suffix in ("stdout", "stderr"):
        raw = (B / ("diagnostic." + suffix)).read_bytes()
        digest = hashlib.sha256(raw).hexdigest()
        assert digest == result["diagnostic"][suffix + "_sha256"] == review["diagnostic_" + suffix + "_sha256"]
        assert len(raw) == result["diagnostic"][suffix + "_bytes"]
    assert result["diagnostic"]["exit_status"] == review["diagnostic_exit_status"] == 101
    assert result["scientific_or_regression_acceptance"] is False
    assert result["target_binary"]["binary_sha256"] == review["captured_test_binary_raw_sha256_remote_claim"]
    blocks = (B / "diagnostic.stderr").read_text().split("--- PHREEQC input ---")
    assert len(blocks) == 33

    def input_value(index, name):
        matches = re.findall(r"^    " + re.escape(name) + r"\s+([^\s]+)\s*$", blocks[index], re.M)
        assert len(matches) == 1
        return Decimal(matches[0])

    def output_value(index, name):
        matches = re.findall(r"^   " + re.escape(name) + r"\s+([^\s]+)\s*$", blocks[index], re.M)
        assert len(matches) == 1
        return Decimal(matches[0])

    with localcontext() as context:
        context.prec = 50
        before_c = input_value(23, "water") * input_value(23, "C")
        after_c = input_value(24, "water") * input_value(24, "C")
        inorganic_loss = before_c - after_c
        fraction = output_value(22, "m_HCO3-") / sum(output_value(22, n) for n in ("m_CO2", "m_H2CO3", "m_HCO3-", "m_CO3-2"))
        bicarbonate_share = before_c * fraction
        error = inorganic_loss - bicarbonate_share
        assert inorganic_loss > 0 and abs(error) < Decimal("1e-18")
        water_gain = input_value(24, "water") - input_value(23, "water")
        assert water_gain > 0
        print(json.dumps({"inorganic_C_loss_mol": str(inorganic_loss), "prior_canonical_HCO3_share_mol": str(bicarbonate_share), "difference_mol": str(error), "water_gain_kg": str(water_gain), "native_inputs": [23, 24], "preceding_native_readback": 22, "scientific_pass": False}, indent=2))


if __name__ == "__main__":
    main()
