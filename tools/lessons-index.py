#!/usr/bin/env python3
"""Write `index.json` for a directory of `.lab` lessons — the picker's menu.

Shared by every payload the lesson player can read from: the web build
(`tools/build-web.sh`) and the desktop/mobile shell's bundled payload
(`tools/build-shell-payload.sh`). One copy, because a topic grouping that
drifted between the two would show a different curriculum per platform.

Usage: python3 tools/lessons-index.py <dir-of-.lab-files>
"""

import json
import pathlib
import re
import sys

# Topic grouping for the picker — the curated order the console page's
# example buttons had, which a flat alphabetical list lost.
TOPICS = {
    "start here": ["silver-and-salt", "first-warmth", "one-thing-at-a-time",
                   "pepper-and-soap", "oil-water-colour", "magic-milk"],
    "acids & bases": ["fizz", "chalk-vinegar", "neutral-moves", "three-protons", "buffer",
                      "titration", "titration-manual", "two-roads",
                      "there-and-back"],
    "heat & fire": ["calorimetry", "fire", "grit"],
    "redox & electricity": ["spannungsreihe", "electrode", "electrolysis",
                            "lemon-cell", "water-electrolysis", "counting-in-fives",
                            "permanganate-standardisation"],
    "water chemistry": ["hard-water", "limewater"],
    "corrosion & materials": ["rusting", "copper-patina", "instant-snow"],
    "crystals & solubility": ["rock-candy", "borax-snowflake", "blue-crystals",
                                "salt-from-brine"],
    "gases & pressure": ["sealed-gas"],
    "rates": ["elephant-toothpaste", "yeast-fermentation", "rates",
              "luminol-temperature"],
    "separations": ["spirit-still", "transport-column"],
    "safety": ["never-mix"],
}

# A collection says what the existing lesson can demonstrate and, separately,
# where its authority stops.  These are display facts, not new chemistry.
COLLECTIONS = {
    "rock-candy": {
        "collection": "crystal lab",
        "outcome_note": "computed saturation and seeded crystal yield",
        "boundary_note": "no crystal size, habit, purity, or growth clock",
    },
    "borax-snowflake": {
        "collection": "crystal lab",
        "outcome_note": "computed cooling yield with a declared phase stand-in",
        "boundary_note": "no snowflake shape; anhydrous borax stands in for the decahydrate",
    },
    "blue-crystals": {
        "collection": "crystal lab",
        "outcome_note": "computed crystal amount, hydration, and solution colour",
        "boundary_note": "no crystal faces, specimen size, or week-long growth",
    },
    "salt-from-brine": {
        "collection": "crystal lab",
        "outcome_note": "computed evaporation and salt recovery",
        "boundary_note": "no crystal habit, grain size, or growth time",
    },
}

REAGENT = re.compile(r"^(?:add|titrate|grind)\s+\S+\s+(\S+)")


def lesson_kit(text: str) -> list[str]:
    """Shelf keys used by the same commands the lesson player executes."""
    return sorted({match.group(1) for line in text.splitlines()
                   if (match := REAGENT.match(line.strip()))})


def index(directory: pathlib.Path) -> list[dict]:
    topic_of = {stem: topic for topic, stems in TOPICS.items() for stem in stems}
    order = {stem: i for stems in TOPICS.values() for i, stem in enumerate(stems)}

    out = []
    for p in sorted(directory.glob("*.lab")):
        text = p.read_text()
        # The first comment line is the lesson's own description.
        blurb = next(
            (l.lstrip("#").strip() for l in text.splitlines() if l.startswith("#")),
            "",
        )
        entry = {
            "file": p.name,
            "name": p.stem.replace("-", " "),
            "blurb": blurb,
            "topic": topic_of.get(p.stem, "more"),
            # Enables generated element-to-lesson links without downloading
            # and reparsing every lesson in the browser. The .lab file stays
            # authoritative; this field is rebuilt for every payload.
            "kit": lesson_kit(text),
        }
        entry.update(COLLECTIONS.get(p.stem, {}))
        out.append(entry)
    topics = list(TOPICS) + ["more"]
    out.sort(key=lambda e: (topics.index(e["topic"]), order.get(e["file"][:-4], 99)))
    return out


if __name__ == "__main__":
    directory = pathlib.Path(sys.argv[1])
    (directory / "index.json").write_text(json.dumps(index(directory)))
    print(f"   {len(list(directory.glob('*.lab')))} lessons indexed")
