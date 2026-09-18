#!/usr/bin/env python3
"""How much of the ENGINE's own prose is translatable yet (I18N-5).

`tools/codex-locale-lint.py` measures the catalogue. This measures the
other half: the sentences `crates/kerotakis-core/src/render.rs` composes
itself, which no amount of `_de` keys in the codex can reach.

Two different numbers, and the second is the one that matters:

  reachable  — literals that go through `locale.t` / `locale.fill`, so a
               catalogue CAN translate them
  translated — of those, how many each shipped language actually carries

A literal still inside a bare `format!` is not merely untranslated; it is
untranslatABLE, and no amount of work in a .toml will change that. That is
why the two are counted separately: the first is a code change, the second
is a translation.

    python3 tools/engine-locale-lint.py
    python3 tools/engine-locale-lint.py --check   # non-zero if a key is orphaned

The check is deliberately NOT "fail below N% translated". A partial
translation is the intended state — every string falls back to English on
its own — so a coverage floor would only encourage machine-filling the
catalogue to make a number go green.
"""

from __future__ import annotations

import collections
import pathlib
import re
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent
RENDER = ROOT / "crates/kerotakis-core/src/render.rs"
# The grammar reads whole catalogue SECTIONS backwards, to learn what a
# learner may type. Those keys reach no call site in render.rs and are not
# orphans; this is where they are used.
SCRIPT = ROOT / "crates/kerotakis-core/src/script.rs"
# The bench's refusals do not go through `render.rs` at all: they are the
# reason an operator did NOTHING, so there is no event to render. They name
# their key at the point they are constructed, and the host renders them in
# the session's locale on the way out. Same catalogue, same fallback, other
# file — so this lint has to read two sources or it reports every refusal
# key as an orphan.
BENCH = ROOT / "crates/kerotakis-core/src/bench.rs"
# I18N-7/8. The sentences the engine BUILDS reach no call site in
# `render.rs` either: `appearance.rs` composes what a vessel looks like and
# the solvers compose why a metal did not react, each as a `Phrase` — a
# key, its English, and the slots — emitted in the EVENT and rendered by
# whichever host is reading. Before this list existed, every `look` line
# and every inert verdict was invisible to this lint in both directions:
# unreachable prose it did not count, and catalogue keys it would have
# called orphans.
COMPOSERS = [
    ROOT / "crates/kerotakis-core/src/appearance.rs",
    ROOT / "crates/kerotakis-core/src/displacement.rs",
    ROOT / "crates/kerotakis-core/src/solve.rs",
    ROOT / "crates/kerotakis-core/src/nonaqueous.rs",
    # I18N-11. `scene.rs` composes the sentences the WEB bench paints from
    # — the caption under a drawn vessel and its accessibility text — and
    # those never pass through `render.rs` at all. Ten of them were still a
    # bare `format!` after I18N-7 had made the clause they were glued to
    # translatable, which is exactly the shape this lint exists to count.
    ROOT / "crates/kerotakis-core/src/scene.rs",
    # I18N-10, tranche by tranche. A file joins this list on the commit
    # that gives its first `Event::NotYetModeled` a `reason`.
    ROOT / "crates/kerotakis-core/src/selectivity.rs",
    ROOT / "crates/kerotakis-core/src/gas_tests.rs",
    ROOT / "crates/kerotakis-core/src/clock.rs",
    ROOT / "crates/kerotakis-core/src/family.rs",
    ROOT / "crates/kerotakis-core/src/bench.rs",
    # I18N-10's tail. `states.rs` and `volatility.rs` compose a refusal a
    # solver passes through; `aqueous.rs` and `phase_diagnostics.rs` are
    # the aqueous crate's own nine and one; `family_oracle.rs` is where
    # the structural oracle's refusal is written, one crate away from the
    # router that speaks it. A file joins this list on the commit that
    # gives its first refusal a `Phrase` — leave one off and its keys are
    # reported as orphans in one direction and vanish from the
    # denominator in the other, which is #505's scar exactly.
    ROOT / "crates/kerotakis-core/src/states.rs",
    ROOT / "crates/kerotakis-core/src/volatility.rs",
    ROOT / "crates/kerotakis-phreeqc/src/aqueous.rs",
    ROOT / "crates/kerotakis-phreeqc/src/phase_diagnostics.rs",
    ROOT / "crates/kerotakis-org/src/family_oracle.rs",
    ROOT / "crates/kerotakis-core/src/kinetics.rs",
    # `Provenance.routing` — why one dataset answered and not another —
    # was the third instance of the same defect, and it is composed in
    # four files rather than one: the aqueous router chooses the dataset,
    # the electrode pass nests that choice inside its own sentence, and
    # the two combustion routes each say why they answered. A file that
    # composes a routing clause belongs here for the same reason a file
    # that composes a refusal does.
    ROOT / "crates/kerotakis-core/src/combustion.rs",
    ROOT / "crates/kerotakis-cea/src/thermal.rs",
    # The particle drawing's captions. `Census::render` took a `Register`
    # and no `Locale` at all until 2026-09-18, so a German session drew its
    # particles under English captions — and because this file was not in
    # this list, the lint could not see that either. A surface invisible to
    # the instrument that counts surfaces is the #505 shape once more, and
    # it is why the tail `", and {n} more"` outlived the I18N-10 sweep.
    ROOT / "crates/kerotakis-core/src/particles.rs",
    # `kero explain` is a CLI command whose labels live in the ENGINE
    # catalogue under `[explain]`, because a CLI catalogue of its own would
    # make adding French three files per language instead of two. It is
    # here so those keys are COUNTED — the unreachable-literal report reads
    # `render.rs` alone, so the CLI's remaining English (every other command
    # it prints) is not reported from here and remains its own question.
    ROOT / "crates/kerotakis-cli/src/main.rs",
]
# `phrase.rs` asks the catalogue for the list grammar and the punctuation
# by name, the ordinary `locale.t` way.
PHRASE = ROOT / "crates/kerotakis-core/src/phrase.rs"
CATALOGUES = ROOT / "crates/kerotakis-core/i18n"

# I18N-10. `Event::NotYetModeled.what` is a finished English sentence, the
# same defect `Inert.why` was one event along, and it is built at sites
# scattered across four crates. The denominator is therefore the SOURCE —
# every construction of the variant outside a test module — and never the
# number of `refusal.*` rows a catalogue happens to carry, which is the
# denominator that let #505 report `models.toml` at 100% German over 325
# English strings. A site counts as done when it carries `reason: Some(…)`.
REFUSAL_EVENT = "Event::NotYetModeled {"

# `locale.t("vessel.open", ", open to atmosphere")` and the fill() form.
CALL = re.compile(r'locale\s*\.\s*(?:t|fill)\s*\(\s*"([^"]+)"\s*,\s*"((?:[^"\\]|\\.)*)"')
# A bare user-facing literal. Newlines are excluded from the character
# class on purpose: `[^"\\]` matches them, so the pattern would span from
# the end of one string to the start of the next across a multi-line
# format! and report the code between them as prose.
LITERAL = re.compile(r'"((?:[^"\\\n]|\\.){6,})"')

# `locale.lookup(&format!("glassware.{}", …))`: keys under a prefix looked
# up this way never appear as a literal at a call site, so they are used
# even though nothing names them. Reporting them as orphans is the lint
# being wrong about a legitimate pattern, and a lint that cries wolf on a
# legitimate pattern is one people learn to ignore.
DYNAMIC = re.compile(r'locale\s*\.\s*lookup\s*\(\s*&?\s*format!\s*\(\s*"([\w.-]+)\.\{')

# `locale.section("script-verb")` — the whole section, read backwards by
# the command grammar. Every key under such a prefix is used by
# definition, and none of them is ever named at a call site.
SECTION = re.compile(r'\.\s*section\s*\(\s*"([\w.-]+)"')
# …and the sections it reads from a list rather than one at a time. Every
# section the grammar owns is named `script-*`, which is the convention
# that makes this answerable at all.
SECTION_LITERAL = re.compile(r'"(script-[\w-]+)"')

# `Refusal::new("error.no-such-vessel", "no vessel {vessel} — …")` — the
# same (key, English source) pair `locale.t` takes, written at the point
# the refusal is made rather than at the point it is rendered.
REFUSAL = re.compile(
    r'Refusal::new\(\s*"([^"]+)"\s*,\s*"((?:[^"\\]|\\.)*)"', re.S
)

# `Phrase::new("look.deposit-dry", "there is {what} in the beaker", …)` and
# the no-slot `Phrase::bare` form — the same (key, English) pair, written
# where the sentence is composed rather than where it is rendered.
PHRASE_CALL = re.compile(
    r'Phrase::(?:new|bare)\(\s*"([^"]+)"\s*,\s*"((?:[^"\\]|\\.)*)"', re.S
)
# `Phrase::new(kerotakis_core::family::UNNAMEABLE_PRODUCT, "…", …)`. A key
# two crates share is named by a `const` so the two cannot drift apart,
# and a lint that only reads literals would then drop it out of the
# DENOMINATOR — which is #505's failure in the other direction: the number
# goes green because the work left it. The const's value is resolved from
# the source, never assumed.
PHRASE_CONST = re.compile(
    r'Phrase::(?:new|bare)\(\s*(?:[A-Za-z_][\w]*::)*([A-Z][A-Z0-9_]+)\s*,\s*"((?:[^"\\]|\\.)*)"',
    re.S,
)
CONST_KEY = re.compile(r'const\s+([A-Z][A-Z0-9_]+)\s*:\s*&\s*str\s*=\s*"([^"]+)"')
# `Phrase::bare("material-assumption.whole_milk-casein", &sentence)` — a
# key named at the call site whose ENGLISH is data, quoted from a material
# recipe. The source text cannot be read out of the Rust, but the key can,
# and a key this lint cannot see is a key it reports as an orphan while
# leaving the row out of the denominator.
PHRASE_KEY_ONLY = re.compile(r'Phrase::(?:new|bare)\(\s*"([^"]+)"\s*,\s*[^"\s]')
# `Phrase::bare(&format!("unspeciated-acid.{key}"), why)` — a CURATED row
# keyed by its place in a table, the shape `inert-in-solvent` introduced.
# The prefix is a dynamic section like any other, and reading it out of
# the source rather than listing prefixes by hand is what keeps the next
# one from being silently orphaned.
PHRASE_DYNAMIC = re.compile(
    r'Phrase::(?:new|bare)\(\s*&?\s*format!\s*\(\s*"([\w.-]+)\.\{'
)


def notmodeled_sites() -> tuple[int, int, list[str]]:
    """(carrying a Phrase, still a finished sentence, where the rest are).

    Braces are matched rather than regexed: the variant is constructed
    across as many as ten lines and a line-based count would miss most of
    them. Test modules are cut the same way the rest of this lint cuts
    them — nobody reads a fixture on a screen.
    """
    done = todo = 0
    remaining: collections.Counter[str] = collections.Counter()
    for path in sorted(ROOT.glob("crates/*/src/**/*.rs")):
        # `ops.rs` DEFINES the event and its constructor. The struct
        # literal inside `Event::not_modeled` is the one place that is not
        # a site, and counting it would have the helper report itself as a
        # migrated call site.
        if path.name == "ops.rs":
            continue
        text = without_test_modules(path.read_text())
        # A CONVERTED site is a call to `Event::not_modeled`, which
        # generates `what` from the recipe. It is no longer a struct
        # literal, so it would otherwise leave the denominator entirely
        # and make the percentage go up by deleting its own numerator.
        done += text.count("Event::not_modeled(")
        i = 0
        while True:
            at = text.find(REFUSAL_EVENT, i)
            if at == -1:
                break
            j, depth = at + len(REFUSAL_EVENT), 1
            while j < len(text) and depth:
                if text[j] == "{":
                    depth += 1
                elif text[j] == "}":
                    depth -= 1
                j += 1
            block, i = text[at:j], j
            # A pattern, not a construction. `Event::NotYetModeled { .. }`
            # in a `matches!` binds fields rather than filling them, and
            # `{ vessel, .. }` in a `retain` looks exactly like a shorthand
            # construction until you notice the rest-pattern.
            if ".." in block:
                continue
            if "vessel:" not in block and "vessel," not in block:
                continue
            if "what:" not in block and "what," not in block:
                continue
            # The last shape a pattern can wear: `Event::NotYetModeled {
            # vessel, what, cause } => …` in `localize_event` names every
            # field and takes no rest-pattern, so it reads as a shorthand
            # construction right up to the fat arrow after it.
            if text[j:].lstrip().startswith("=>"):
                continue
            # A pass-through site — `localize_event` rebuilding the
            # event, `phase_diagnostics` re-emitting one — carries the
            # reason on rather than composing one, and is done when it
            # stops dropping it. `reason: None` is the unconverted state.
            has_field = re.search(r"(?<![A-Za-z0-9_])reason:", block) is not None
            if has_field and "reason: None" not in block:
                done += 1
            else:
                todo += 1
                remaining[str(path.relative_to(ROOT))] += 1
    return done, todo, [f"{n:>4}  {f}" for f, n in remaining.most_common()]


def unwrap(text: str) -> str:
    """A Rust string literal's `\\`-at-end-of-line continuation, undone."""
    return re.sub(r"\\\n\s*", "", text)


def without_test_modules(text: str) -> str:
    """Every `#[cfg(test)] mod … { … }` removed, braces matched.

    Cutting at the FIRST `#[cfg(test)]` is what this file used to do, and
    it is right only for a file whose tests are all at the bottom.
    `kinetics.rs` has a test module at line 1294 and a thousand lines of
    engine after it, so the cut hid `proton_consumption_boundary`
    entirely: its key was reported as an orphan while its row sat outside
    the denominator. A lint that cannot see part of the source is one
    whose percentage means nothing, which is #505's lesson in its
    sharpest form.
    """
    out, i = [], 0
    while True:
        at = text.find("#[cfg(test)]", i)
        if at == -1:
            out.append(text[i:])
            return "".join(out)
        out.append(text[i:at])
        brace = text.find("{", at)
        if brace == -1:
            return "".join(out)
        j, depth = brace + 1, 1
        while j < len(text) and depth:
            if text[j] == "{":
                depth += 1
            elif text[j] == "}":
                depth -= 1
            j += 1
        i = j


def uncommented(text: str) -> str:
    """Whole-line `//` comments dropped.

    A comment sits between `Phrase::new(` and its key often enough —
    saying WHY that key and not another — that a pattern which cannot
    step over one silently loses the call, and a lost call is a row this
    lint then reports as an orphan while dropping it from the
    denominator. Whole lines only: a `//` inside a string literal is not
    at the start of its line, and a comment that is would be.
    """
    return "\n".join(
        line for line in text.splitlines() if not line.lstrip().startswith("//")
    )

# A dotted key named anywhere in the file, which covers the case where the
# key is chosen by a match arm rather than passed literally:
#
#   locale.t(match p.phase { Phase::Gas => "phase.gas", … }, …)
#
# Weaker than "named at a call site", deliberately. The match is the
# clearest way to write a four-way choice, and a lint that is wrong about
# correct code is a lint people stop reading.
MENTIONED = re.compile(r'"([a-z][\w-]*(?:\.[\w -]+)+)"')
PROSE = re.compile(r"[A-Za-z]{3,}\s+[A-Za-z]{2,}")


def load_catalogue(path: pathlib.Path) -> dict[str, str]:
    doc = tomllib.load(open(path, "rb"))
    out: dict[str, str] = {}
    for section, body in doc.items():
        if isinstance(body, dict):
            for k, v in body.items():
                if isinstance(v, str):
                    out[f"{section}.{k}"] = v
        elif isinstance(body, str):
            out[section] = body
    return out


def main() -> int:
    src = RENDER.read_text()
    # Stop at the test module. Its fixtures and expected strings are prose
    # by every measure this script applies — "Some of the magnesium in v1
    # is used up." — but nobody reads them on a screen, and counting them
    # inflated the remaining work by more than half.
    cut = src.find("\n#[cfg(test)]")
    if cut != -1:
        src = src[:cut]
    used = {m.group(1): m.group(2) for m in CALL.finditer(src)}
    bench = BENCH.read_text()
    bench_cut = bench.find("\n#[cfg(test)]")
    if bench_cut != -1:
        bench = bench[:bench_cut]
    refusals = {m.group(1): unwrap(m.group(2)) for m in REFUSAL.finditer(bench)}
    used.update(refusals)
    # Every `const … : &str = "…"` the workspace declares, so a key named
    # by a constant is still counted where it is used.
    const_keys: dict[str, str] = {}
    for path in sorted(ROOT.glob("crates/*/src/**/*.rs")):
        for m in CONST_KEY.finditer(path.read_text()):
            const_keys[m.group(1)] = m.group(2)
    composed: dict[str, str] = {}
    for path in COMPOSERS:
        text = uncommented(without_test_modules(path.read_text()))
        # A composer file may also ask the catalogue DIRECTLY, the ordinary
        # `locale.t` / `locale.fill` way, when what it builds is a string
        # rather than an event — `particles.rs` draws the census and hands
        # back text. Scanning these files for `Phrase` alone reported six
        # real keys as orphans on 2026-09-18, which is this lint's own
        # denominator failing in the direction #505 failed: a surface
        # invisible to the instrument that counts surfaces.
        for m in CALL.finditer(text):
            composed[m.group(1)] = unwrap(m.group(2))
        for m in PHRASE_CALL.finditer(text):
            composed[m.group(1)] = unwrap(m.group(2))
        for m in PHRASE_KEY_ONLY.finditer(text):
            composed.setdefault(m.group(1), "")
        for m in PHRASE_CONST.finditer(text):
            key = const_keys.get(m.group(1))
            if key:
                composed[key] = unwrap(m.group(2))
    # `phrase.rs` asks for the list grammar and the punctuation the
    # ordinary `locale.t` way — and composes one clause of its own,
    # `look.sentence-join`, which a CALL-only read of this file could not
    # see. A key the lint cannot see is a key outside the denominator.
    phrase_src = uncommented(without_test_modules(PHRASE.read_text()))
    for m in CALL.finditer(phrase_src):
        composed[m.group(1)] = m.group(2)
    for m in PHRASE_CALL.finditer(phrase_src):
        composed[m.group(1)] = unwrap(m.group(2))
    used.update(composed)
    grammar = SCRIPT.read_text()
    dynamic = {m.group(1) for m in DYNAMIC.finditer(src)}
    # `Slot::Term { section, en }` looks a term up by VALUE under a section
    # chosen at runtime, and the curated solvent verdicts build their key
    # out of the table row they came from. Neither can be named at a call
    # site, which is the same legitimate pattern the glassware and species
    # tables use.
    for path in COMPOSERS + [BENCH, RENDER]:
        text = path.read_text()
        dynamic |= {
            m.group(1) for m in re.finditer(r'Slot::term\(\s*"([\w.-]+)"', text)
        }
        dynamic |= {m.group(1) for m in PHRASE_DYNAMIC.finditer(text)}
        dynamic |= {m.group(1) for m in DYNAMIC.finditer(text)}
    dynamic |= {m.group(1) for m in DYNAMIC.finditer(grammar)}
    dynamic |= {m.group(1) for m in SECTION.finditer(grammar)}
    dynamic |= {m.group(1) for m in SECTION_LITERAL.finditer(grammar)}
    mentioned = {m.group(1) for m in MENTIONED.finditer(src)}
    mentioned |= {m.group(1) for m in MENTIONED.finditer(grammar)}
    # A composer file may choose its key by a match arm too — `particles.rs`
    # picks one of six `census.kind.*` for the word beside each drawn row.
    # Scanning only `render.rs` for that shape reported all six as orphans
    # on 2026-09-18, the same direction this lint's own denominator failed
    # in an hour earlier with `locale.t` in a composer.
    for path in COMPOSERS:
        mentioned |= {
            m.group(1)
            for m in MENTIONED.finditer(uncommented(without_test_modules(path.read_text())))
        }

    # Everything that looks like prose, minus what already goes through a
    # call. Rough by design: it over-reports rather than under-reports,
    # because a missed line is one nobody knows is English.
    reachable_texts = set(used.values())

    # A literal that is a lookup KEY under a dynamic prefix — the
    # instrument, verb, species and glassware tables build their English
    # name and then ask the catalogue for it by that name.
    for path in CATALOGUES.glob("*.toml"):
        cat = load_catalogue(path)
        for k in cat:
            head, _, tail = k.partition(".")
            if head in dynamic and tail:
                reachable_texts.add(tail)

    # A literal passed to locale.t / locale.fill in any shape — including
    # the ones bound by a `let` first, or chosen by an `if` inside the
    # argument list, which the CALL pattern cannot see.
    for m in re.finditer(r"locale\s*\.\s*(?:t|fill)\s*\(", src):
        depth, i = 1, m.end()
        while i < len(src) and depth:
            if src[i] == "(":
                depth += 1
            elif src[i] == ")":
                depth -= 1
            i += 1
        for lit in re.findall(r'"((?:[^"\\\n]|\\.)*)"', src[m.end():i]):
            reachable_texts.add(lit)
    # And the `let en = if … { "…" } else { "…" };` form, whose strings sit
    # outside the call entirely.
    for m in re.finditer(r"let\s+\w+\s*=\s*if[^;]{0,400}?;", src, re.S):
        if "locale." in src[m.end():m.end() + 200]:
            for lit in re.findall(r'"((?:[^"\\\n]|\\.)*)"', m.group(0)):
                reachable_texts.add(lit)

    bare = set()
    for m in LITERAL.finditer(src):
        text = m.group(1)
        if text in reachable_texts or not PROSE.search(text):
            continue
        # Doc comments and attributes are not output.
        line_start = src.rfind("\n", 0, m.start()) + 1
        line = src[line_start : m.start()]
        if line.lstrip().startswith(("//", "#[", "///")):
            continue
        bare.add(text)

    # A key used by two DIFFERENT templates is the worst failure this
    # file can have: not a missing translation but a wrong sentence, since
    # whichever German lands in the catalogue renders for both. Each bulk
    # converter checked its own output for collisions and neither checked
    # the file, so `event.smelled.lv3` was created twice.
    per_key = collections.defaultdict(set)
    for m in CALL.finditer(src):
        per_key[m.group(1)].add(m.group(2))
    for m in REFUSAL.finditer(bench):
        per_key[m.group(1)].add(unwrap(m.group(2)))
    for path in COMPOSERS:
        for m in PHRASE_CALL.finditer(path.read_text()):
            per_key[m.group(1)].add(unwrap(m.group(2)))
    shared = {k: v for k, v in per_key.items() if len(v) > 1}
    if shared:
        print("KEY USED BY TWO DIFFERENT SENTENCES:")
        for k, texts in sorted(shared.items()):
            print(f"   {k}")
            for x in sorted(texts):
                print(f"      {x[:70]}")

    print(f"{'engine prose in render.rs':<34}")
    print(f"   reachable by a catalogue : {len(used) - len(refusals):>4} keys")
    print(f"   still inside a bare format!: {len(bare):>4} literals")
    print(f"{'bench refusals in bench.rs':<34}")
    print(f"   reachable by a catalogue : {len(refusals):>4} keys")
    print(f"{'sentences the engine composes':<34}")
    print(f"   reachable by a catalogue : {len(composed):>4} keys")
    print(f"   ({', '.join(p.name for p in COMPOSERS)}, phrase.rs)")
    done, todo, where = notmodeled_sites()
    print(f"{'refusals in NotYetModeled.what':<34}")
    print(f"   carrying a Phrase        : {done:>4} sites")
    print(f"   still a finished sentence: {todo:>4} sites")
    for line in where:
        print(f"   {line}")

    problems = len(shared)
    print()
    print(f"{'language':<12} {'translated':>10} {'of':>4} {'reachable':>10}   coverage")
    for path in sorted(CATALOGUES.glob("*.toml")):
        code = path.stem
        cat = load_catalogue(path)
        hit = sum(1 for k in used if k in cat)
        pct = 100 * hit / len(used) if used else 100.0
        dyn = sum(1 for k in cat if k.split(".")[0] in dynamic)
        arm = sum(1 for k in cat if k not in used and k.split(".")[0] not in dynamic
                  and k in mentioned)
        parts = []
        if dyn:
            parts.append(f"+{dyn} looked up by value")
        if arm:
            parts.append(f"+{arm} chosen by a match arm")
        extra = f"   ({', '.join(parts)})" if parts else ""
        print(f"{code:<12} {hit:>10} {'/':>4} {len(used):>10}   {pct:5.1f}%{extra}")

        # A key in the catalogue that no call site asks for is dead weight,
        # and usually a rename nobody finished.
        for k in sorted(set(cat) - set(used)):
            if k.split(".")[0] in dynamic or k in mentioned:
                continue
            print(f"   ORPHAN: {code}.toml has '{k}', which nothing asks for")
            problems += 1

    if bare:
        print(f"\nnot yet reachable — each needs a locale.t() at its call site:")
        for text in sorted(bare)[:12]:
            print(f"   {text[:76]}")
        if len(bare) > 12:
            print(f"   … and {len(bare) - 12} more")

    if problems:
        print(f"\n{problems} problem(s)")
    return 1 if (problems and "--check" in sys.argv) else 0


if __name__ == "__main__":
    raise SystemExit(main())
