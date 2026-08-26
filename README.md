# Kerotakis

A virtual chemistry laboratory that computes real chemistry.

Offline-first, cross-platform, no runtime Python. One simulation, rendered at
whatever level of detail the reader wants. Nothing in it is a lookup table:
every number below came out of a thermodynamic database at run time.

It emphasises the **models** that make facts predictable, because an engine
that actually computes can do something a textbook cannot: show you a model
working, and then show you the same model failing.

Named for the sealed reflux vessel invented by Maria the Jewess in Alexandria,
1st–3rd century CE — the first named alchemist in recorded history, who also
gave us the bain-marie, and whose airtight seal is the origin of "hermetically
sealed". The name describes the architecture: a sealed vessel you put things
into, and reactions happen.

## Try it in a browser

The bench runs live at
**[crispstrobe.github.io/kerotakis/app](https://crispstrobe.github.io/kerotakis/app/)** —
drawn glassware, drag-and-drop reagents, the register dial (lv1 "what you
see" → lv3 "everything"), lessons, undo-by-replay, and the real aqueous
engine solving in a worker. The
[console](https://crispstrobe.github.io/kerotakis/) is the same engine as a
terminal. Both are offline-capable after the first visit.

It is also an installable app. Your browser will offer to install the bench;
it then runs from its own window with the engine, the thermodynamic
databases and the whole lesson library cached locally, and works with the
network switched off. On iOS, *Share → Add to Home Screen*.

The same bench is packaged for macOS and iOS as a native app, where the
solver is linked in-process rather than run in a worker — same UI, same
answers, no network at all. See [PACKAGING.md](PACKAGING.md) for how the
three are built and how they stay identical.

## What works today

```console
$ kero run lessons/silver-and-salt.lab
  You add water to v1.
  You add sodium chloride to v1.
  The sodium chloride disappears into the water in v1!
  ...
  It went cloudy in v1! A white solid appears at the bottom — that's called a precipitate.
```

The same bench, same solvers, at lv3:

```console
  v1 (beaker) — 25.00 °C, 201.7 g, 200.0 mL liquid, pH 7.10, I = 0.0502 m
        0.0099 mol  silver chloride    Solid
      speciation (mol/kgw · activity · γ):
        Na+             1.0040e-1    7.8550e-2   γ=0.782
        Ag+             9.5630e-6    7.4690e-6   γ=0.781
        AgCl            3.2140e-7    3.2890e-7   γ=1.023   ← the neutral complex
```

Computed, not scripted:

- **Dissolution and precipitation** with real solubility limits — 8 mol of salt
  in a litre of water leaves ~1.9 mol undissolved.
- **Acids and bases** from charge balance: strong (HCl → pH 3.0), weak
  (acetic → 2.88), polyprotic (phosphoric, all three pKa's), and **buffers**
  that resist acid where plain water crashes.
- **Titration curves** walked with a burette, to equivalence and past it.
- **Explicit gas boundaries** — vinegar and baking soda can vent CO₂ to room
  air, retain it at fixed volume, expand a pressure-controlled headspace, or
  lose it under a nitrogen sweep. Use `seal v1 500mL`,
  `regulate v1 1.5bar 500mL`, `sweep v1 1bar`, and `open v1`; gas/liquid
  partitioning, pressure, volume, energy, and the balance respond to the
  constraint. Rigid trapped gas uses its constant-volume heat capacity, while
  a pressure-controlled headspace includes expansion work.
- **Bidirectional gas transfer** — `add v1 CO2 0.01mol` bubbles a finite dose
  through an open vessel. Limewater turns milky as calcite forms; continued
  excess CO₂ redissolves it, with absorbed and vented carbon accounted for.
- **Heat**: dissolution enthalpies drive the vessel temperature, so calcium
  chloride is a +20 K hot pack and potassium chloride a −4 K cold pack.
- **Hard-water chemistry**: chalk, limescale, and gypsum binding its two
  waters of crystallisation into the crystal.
- **Separations**: filter a precipitate off, evaporate brine to crystals.
- **Fire**: chalk calcines to quicklime when heated hard enough (the
  decomposition temperature is computed, not assumed), and magnesium burns at
  ~3000 K, *gaining* mass because the oxygen came from the air.
- **The activity series, computed**: zinc pushes copper out of solution
  (218.7 kJ/mol of heat with it), copper shrugs off acid but trades
  itself for silver, and a sacrificial zinc block guards iron — each
  verdict, including the refusals, decided by Nernst over computed
  activities rather than a stored table of who beats whom. Two
  half-cells make a battery: the Daniell cell offers 1.104 V
  open-circuit, tenfold dilution costs 18.2 mV rather than the
  textbook 29.6 (the activities are not the concentrations), and two
  beakers differing only in concentration read a real 18 mV from
  imbalance alone.
- **Colour, from absorption spectra rather than a tint per substance.**
  Absorbances add and Beer–Lambert is applied over the CIE 1931 observer,
  so mixtures compose the way a beaker composes them, the depth of liquid
  matters, and *concentration changes hue*: one permanganate spectrum reads
  pink at 10⁻⁵ M and purple at 10⁻³ M.
- **Hazards that teach**: bleach + ammonia warns precisely and *then shows*
  the chloramine forming and leaving the beaker. Prohibition teaches nothing.
  Hold a flame to salt and it does not burn — it gives the sodium flame test.

Equations are balanced by the engine, not by memory — the coefficients are
the null space of the element-count matrix, charge included:

```console
$ kero balance "Cr2O7-2 + Fe+2 + H+ -> Cr+3 + Fe+3 + H2O"
  Cr2O7-2 + 6 Fe+2 + 14 H+ → 2 Cr+3 + 6 Fe+3 + 7 H2O
```

That makes "balance this equation" an exercise the lab can mark, and it
turned into a lint: every codex equation is now checked for atom *and*
charge balance, which caught two wrong ones on its first run.

And you can look at the particles — the ratios are solved, not drawn:

```console
kero> particles v1
  v1 — what the particles are doing:
  ······   H2O  (solvent)
  ●●●●●●●●●●●   Na+  (positive ion)
  ○○○○○○○○○○   Cl-  (negative ion)
  ▪   AgCl  (solid)
  one ● ≈ 4.388e-2 mol/kgw; the water is drawn sparsely, not to scale
  present below one glyph, so not drawn: AgCl2- (1.31e-5), AgCl (2.96e-7), …
```

That last line is the point. A species too dilute to earn a dot is *named*
rather than dropped, because a picture that silently omits the neutral
complex teaches that the complex is not there.

And every answer can explain itself:

```console
kero> explain v1
  v1: answered by PHREEQC (IPhreeqc, USGS) using pitzer.dat
    model:   Pitzer specific-ion-interaction model (valid at high ionic strength)
    routing: chosen because the solution is concentrated (~16.0 mol/kgw)
  the same question, asked of every dataset:
    wateq4f.dat    pH 7.059 · I = 6.4224 m · Halite 1.5969 mol
    minteq.v4.dat  pH 6.855 · I = 3.6940 m · Halite 4.3171 mol
    pitzer.dat     pH 6.469 · I = 6.1108 m · Halite 1.9075 mol
```

Three thermodynamic datasets, three answers, each with its model's validity
range stated. **Showing the disagreement is the lesson** — that is philosophy
of science as a computed result rather than a paragraph, and it falls out of
the engine rather than being narrated.

## Why it is built this way

Chemistry lives at three levels — what you *see*, the *particles* underneath,
and the *symbols* we write — and the research literature is clear that
novices fail because instruction moves between them without saying so. Our
engine computes the particle level for real (that AgCl(aq) complex above is
an answer, not an illustration), so one vessel state can render consistently
at all three.

Detail and representation are separate axes: `lv1|lv2|lv3` sets how much
detail, and macroscopic/submicroscopic/symbolic sets *what kind of picture*.
`particles` is the submicroscopic one — dots drawn at solved ratios rather
than an artist's impression — and every cell of that grid renders the same
solved state.

From that follow the rest of the design commitments:

- **Models are content, not background.** The codex carries 28 model
  entries whose most important field is `fails_at` — a model shown without
  its boundary is shown as truth, which is why the next model feels
  arbitrary instead of necessary. The lint refuses a model with an empty
  `fails_at`. They form eight supersession chains, so a learner meets
  Bohr's atom as the thing that fixed Rutherford's and then meets helium,
  which breaks Bohr's.
- **Prediction comes before observation.** Entries carry a question whose
  wrong options are the mistakes learners actually make; the engine is the
  arbiter. Because it computes rather than looks up, a *quantitative*
  prediction can be checked — which is what makes working out the moles
  load-bearing rather than ritual.
- **Order comes from prerequisites, not school years.** Years are an artefact
  of national administration and differ by country; the dependency structure
  of the ideas does not. Curriculum placements stay on each entry so a
  learner who needs their syllabus topic can still find it.
- **Honesty is a feature, and silence is not honesty.** Solver failures are
  surfaced, unmodelled states are named as such, and every answer says which
  engine, dataset and model produced it. The rule we had to learn: wherever
  the engine declines to model something it must *say so*, because a silent
  filter reads as a fact. Holding a flame to ethanol once reported "nothing
  ignited" — which was not an observation about ethanol but the absence of a
  solver. It now says which it is.
- **Guided, not a sandbox.** Minimally guided discovery is the
  best-documented failure mode in science education, so the codex leads with
  a paradigm case, a committed prediction and a model with a stated boundary;
  the free REPL is the faded end of the scaffold, not the front door.

## Try it in a browser

**[crispstrobe.github.io/kerotakis](https://crispstrobe.github.io/kerotakis/)** —
the bench, with PHREEQC compiled to WebAssembly and solving in the tab. Not a
recording: type something nobody anticipated and it computes an answer, or
tells you it cannot. The header says which of the two it is.

Experiments are shareable as links:
[silver meets salt](https://crispstrobe.github.io/kerotakis/#run=add%20v1%20water%20200mL;add%20v1%20NaCl%200.1mol;add%20v1%20AgNO3%200.01mol;look%20v1;particles%20v1).

## Try it locally

```bash
git clone --recurse-submodules https://github.com/CrispStrobe/kerotakis
cd kerotakis
cargo run -p kerotakis-cli -- run lessons/fizz.lab   # or: cargo run -p kerotakis-cli
```

`kero` with no arguments opens the bench as a REPL. `help` lists the
operators; `register lv1|lv2|lv3` sets how much detail; `explain` traces an answer;
`--json` on a script emits one JSON object per step (that stream is the API
contract the future UI consumes).

## Status

The feasibility gate is passed: PHREEQC cross-compiles and runs identically
natively, in **WebAssembly** (Emscripten, no filesystem), and on
**aarch64-apple-ios**, so the offline premise holds on every target. The
aqueous layer and the thermal layer both compute; **the whole bench runs in a
browser** (`kerotakis-wasm`), with thermal chemistry live and aqueous answers
from pre-warmed results — a state nobody pre-computed is reported as a stated
miss rather than guessed at.

The codex holds **103 reaction entries, 28 models and 155 concepts**, anchored
to a 189-topic CC0 curriculum spine (48 covered; `kero codex gaps` prints the
rest). Every numeric claim in it is replayed through the real solvers by
`kero codex lint` in CI, so a curation error cannot merge and a solver change
that breaks a lesson is caught the same day.

| Crate | Role |
|---|---|
| `kerotakis-core` | Bench and vessel state machine, operators, energy balance, solver router, `.lab` grammar, levels |
| `kerotakis-phreeqc` | IPhreeqc FFI, embedded thermodynamic databases, the aqueous equilibrator (engine optional at compile time) |
| `kerotakis-cea` | NASA-9 thermochemistry and the Gibbs minimiser: heating, calcining, burning |
| `kerotakis-safety` | L0 reactivity screen — runs before any chemistry |
| `kerotakis-codex` | Curated reactions, their concepts, and the lint that replays every claim through the solvers |
| `kerotakis-cli` | `kero`: REPL, batch runner, JSON interface, cache pre-warmer, codex lint |
| `kerotakis-wasm` | The same bench in a browser |

See [PLAN.md](PLAN.md) for the architecture, verified engine and licence audit,
[CAPABILITIES.md](CAPABILITIES.md) for solver capability tasks, and
[BREADTH.md](BREADTH.md) for the dependency-ordered programme that grows
familiar substances, materials, reaction families, biochemistry, crystals and
tactile/scientific views.

## Licence

Code: AGPL-3.0-or-later, with an App Store / Google Play additional permission
for binaries published by the copyright holders. See [LICENSE](LICENSE) and
[NOTICE](NOTICE).

Curated data is licensed separately and openly (CC BY-SA 4.0), and every
dataset we use is named with its own terms — including where an upstream's
licence claim looks wrong to us, in which case we honour the original.

Contributions are welcome under AGPL-3.0-or-later **plus** the store
permission — see [CONTRIBUTING.md](CONTRIBUTING.md) before your first PR.
