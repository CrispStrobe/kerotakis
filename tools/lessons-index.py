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
                      "titration", "titration-manual", "endpoint-is-not-a-full-drop",
                      "two-roads", "there-and-back", "equilibrium-can-run-backward",
                      "two-limits-one-carbon-dioxide-ledger"],
    "heat & fire": ["calorimetry", "fire", "grit", "boiling-curve",
                    "grouping-does-not-change-water-heat",
                    "liquid-nitrogen-freezes-ethanol"],
    "redox & electricity": ["spannungsreihe", "electrode", "electrolysis",
                            "lemon-cell", "water-electrolysis", "counting-in-fives",
                            "permanganate-standardisation",
                            "current-time-and-electrolysis-yield",
                            "equal-charge-different-clocks"],
    "water chemistry": ["hard-water", "limewater", "conductivity",
                        "hard-water-soap-boundary",
                        "saline-scaling-preserves-conductivity",
                        "neutral-sugars-are-conductivity-controls"],
    "corrosion & materials": ["rusting", "copper-patina", "instant-snow"],
    "density & buoyancy": ["density", "whole-object-buoyancy"],
    "tests & indicators": ["starch-iodine-test"],
    "crystals & solubility": ["rock-candy", "borax-snowflake", "blue-crystals",
                                "salt-from-brine"],
    "gases & pressure": ["sealed-gas", "warm-the-gas-stop-before-burst"],
    "rates": ["elephant-toothpaste", "elephant-toothpaste-catalyst-dose",
              "yeast-fermentation", "rates",
              "luminol-temperature", "longer-waits-advance-peroxide-kinetics"],
    "separations": ["water-filter", "spirit-still", "transport-column",
                    "repeated-liquid-extraction", "three-components-one-cut",
                    "follow-the-salt-through-staged-transfers",
                    "filter-then-concentrate-the-brine",
                    "track-precipitate-and-filtrate-through-drying"],
    "safety": ["never-mix"],
    # Two topics with no curated members. They exist because the catalogue
    # sends lessons here (see CATALOGUE_TOPIC below) and a topic has to have
    # a position in the picker's order before anything can land in it.
    "food & life": [],
    "everyday materials": [],
}

# A lesson absent from TOPICS used to fall into "more", and 47 of 113 did.
# That bucket was the picker disagreeing with the catalogue about the same
# curriculum: every one of those 47 carries authored `topics` in
# `data/kids/experiments-v1.json`, so the grouping was already written down
# somewhere — just not here.
#
# This maps the catalogue's learner topics onto the picker's shelves, and it
# is ORDERED: the first catalogue topic a lesson carries that appears in this
# list decides its shelf. The order is the editorial judgement. `proteins`
# outranks `heat` so that heating egg white is food rather than thermochemistry;
# `heat` outranks `food` so that invisible ink is heat rather than cooking;
# `acids` outranks `food` so that kitchen pH is acid-base chemistry. Change
# the order and lessons move, which is the point — it is one list to argue
# about rather than 113 filenames to maintain.
CATALOGUE_TOPIC = [
    ("safety", "safety"),
    ("fire", "heat & fire"),
    ("crystals", "crystals & solubility"),
    ("electrochemistry", "redox & electricity"),
    ("enzymes", "food & life"),
    ("proteins", "food & life"),
    ("rates", "rates"),
    ("heat", "heat & fire"),
    ("acids", "acids & bases"),
    ("indicators", "tests & indicators"),
    ("tests", "tests & indicators"),
    ("density", "density & buoyancy"),
    ("pressure", "gases & pressure"),
    ("food", "food & life"),
    ("gases", "gases & pressure"),
    ("separations", "separations"),
    ("redox", "redox & electricity"),
    ("water", "water chemistry"),
    ("metals", "redox & electricity"),
    ("polymers", "everyday materials"),
    ("mixtures", "everyday materials"),
    ("materials", "corrosion & materials"),
    ("solutions", "water chemistry"),
    ("colour", "tests & indicators"),
    ("equilibrium", "acids & bases"),
    ("measurement", "start here"),
]

CATALOGUE = pathlib.Path(__file__).resolve().parents[1] / "data/kids/experiments-v1.json"


def catalogue_topics() -> dict[str, list[str]]:
    """Lesson stem to the authored topics of the catalogue row that runs it.

    Missing or unreadable is not fatal: the payload builds have to keep
    working from a lessons directory alone, and a lesson with no row simply
    keeps the "more" shelf it had.
    """
    try:
        rows = json.loads(CATALOGUE.read_text())["experiments"]
    except (OSError, ValueError, KeyError):
        return {}
    return {
        row["lesson"][: -len(".lab")]: row.get("topics", [])
        for row in rows
        if row.get("lesson", "").endswith(".lab")
    }

# Learning progress is authored independently of district unlocking.  Most of
# the older missions predate this metadata; new promotions state it explicitly
# instead of deriving it from apparatus, safety, or the learner's age.
PROGRESS = {
    "follow-the-salt-through-staged-transfers": "intermediate",
    "warm-the-gas-stop-before-burst": "intermediate",
    "current-time-and-electrolysis-yield": "advanced",
    "filter-then-concentrate-the-brine": "intermediate",
    "track-precipitate-and-filtrate-through-drying": "advanced",
    "two-limits-one-carbon-dioxide-ledger": "intermediate",
    "longer-waits-advance-peroxide-kinetics": "intermediate",
    "saline-scaling-preserves-conductivity": "intermediate",
    "neutral-sugars-are-conductivity-controls": "intermediate",
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


def topic_for(stem: str, topic_of: dict[str, str], authored: dict[str, list[str]]) -> str:
    """The curated shelf if there is one, else the catalogue's own answer."""
    if stem in topic_of:
        return topic_of[stem]
    carried = authored.get(stem, [])
    for name, shelf in CATALOGUE_TOPIC:
        if name in carried:
            return shelf
    return "more"


def index(directory: pathlib.Path) -> list[dict]:
    topic_of = {stem: topic for topic, stems in TOPICS.items() for stem in stems}
    order = {stem: i for stems in TOPICS.values() for i, stem in enumerate(stems)}
    authored = catalogue_topics()

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
            "topic": topic_for(p.stem, topic_of, authored),
            # Enables generated element-to-lesson links without downloading
            # and reparsing every lesson in the browser. The .lab file stays
            # authoritative; this field is rebuilt for every payload.
            "kit": lesson_kit(text),
        }
        if p.stem in PROGRESS:
            entry["progress"] = PROGRESS[p.stem]
        entry.update(COLLECTIONS.get(p.stem, {}))
        out.append(entry)
    topics = list(TOPICS) + ["more"]
    out.sort(key=lambda e: (topics.index(e["topic"]), order.get(e["file"][:-4], 99)))
    return out


if __name__ == "__main__":
    directory = pathlib.Path(sys.argv[1])
    (directory / "index.json").write_text(json.dumps(index(directory)))
    print(f"   {len(list(directory.glob('*.lab')))} lessons indexed")
