# One instrument surface

Design note for GUI-100 … GUI-103. The app offered instruments in three places
with three different mental models. This says what each was, why that hurt,
what replaced them, and in what order it happened. Sections 1–3 are written in
the present tense of the design; §4 records what shipped, and **§5 is where
the owner's answers superseded parts of §3** — the shelf list, the size of the
quick-access row and the kits' sixth shelf are all decided there, not here.

## 1. What exists today

### (a) The `MESSEN` strip — `InstrumentTray.svelte`

- **Lists** the 12 entries of `INSTRUMENTS` (`instruments.ts`): safe waft,
  thermometer, pH, balance, gas volume, conductivity, pressure, calorimeter,
  UV-Vis, look closely, chromatograph, Geiger.
- **Opened** never — it is always mounted, inside `Inspector.svelte` under the
  heading `t("measure")` → *MESSEN*, in the journal pane. The dock's
  *Messgeräte* button is what puts the journal pane on screen.
- **State**: none. Props are `vessel`, `busy`, `onmeasure`.
- **Emits** `instrumentCommand()`: `measure v1 <token>`, plus the two
  irregulars `smell v1` and `chromatograph v1`.

### (b) The Geräteschrank / Instrumentenwand — `EquipmentCabinet.svelte`

- **Lists** 34 things in four groups: `APPARATUS` (12 verbs), a
  `TRANSFER_TOOLS` array declared *inside the component* (filter, decant,
  drain, magnet, cell, distil), the same 12 `INSTRUMENTS` again, `burette`,
  `mix`, `transport`, `react`, and `KIDS_EQUIPMENT` (5 kits) on top.
- **Opened** as the *equipment* tab of the shelf pane, from four places:
  the tab itself, `VesselActionDock`'s *Geräteschrank*, `Bench`'s
  `onopencabinet`, and `UtilityStation`'s *power and apparatus*.
- **State**: a text filter and one open `(i)` panel; reads `catalog`, `scope`,
  `mode`, and every deployed flag (`apparatusOut`, `buretteOut`,
  `transferVerb`, `mixActive`) from `App.svelte`.
- **Emits** nothing directly except the instrument lines; everything else
  raises a handler that arms a second step — `onapparatus` opens
  `ApparatusForm` (which builds `heat … on burner|candle|hotplate`, `stir`,
  `cool`, `centrifuge`, `dilute`, `evaporate`, `electrolyse`, `grind`,
  `irradiate`, `regulate`, `sweep`, `ignite`), `onburette` arms `titrate`,
  `ontransfer` arms a two-vessel verb, `onmix` arms `mix`.

### (c) The dock and the apparatus panels

`VesselActionDock.svelte` is the third selector, not a panel: `directActions.ts`
gives it `look` / `temperature` / `ph` (three instruments again),
`stir` / `heat` / `cool` (three apparatus again) and `seal`/`open`, and it ends
with **two** buttons — *Messgeräte* and *Geräteschrank* — that lead to (a) and
(b).

`ApparatusForm`, `DeployedApparatus`, `StandaloneApparatus` and `Burette` are
**not** selection surfaces: they are what a selection becomes. They stay.
`Toolbox.svelte` is the named-relation calculator (Nernst, Arrhenius, …); it is
misnamed for a bench tool but holds no equipment and is out of scope.

## 2. The pain

- **Every instrument is listed twice, three of them three times.** The 12
  instruments are in (a) and (b); `eyes`, `thermometer` and `ph` are also in
  (c). `stir`/`heat`/`cool` are in (b) and (c). `chromatograph` is in (a), (b)
  and again as *paper chromatography kit*; `filter`, `magnet`, `regulate` and
  `bunsen` each appear once as equipment and once as a kit.
- **Three mental models.** (a) is "take a reading now"; (b) is "install on the
  bench"; (c) is "do a thing to this vessel". Which one a tool lives in is not
  predictable from what the tool is — `stir` is an install, `ph` is a reading,
  `filter` is a two-vessel arming step, and all three read as buttons.
- **The strip scrolls.** 12 pills in one non-wrapping row hides roughly half
  of them off the right edge on a phone, where the names are also clipped to
  glyphs. The calorimeter and the Geiger counter effectively do not exist.
- **The kits are a fourth vocabulary** for tools already present, in their own
  bordered box at the top of the cabinet, above the tools they alias.
- **The tally is over a variable list.** `allVerbs` includes `react` only when
  `reactAvailable`, so the "N/34" denominator changes size mid-session.
- **`TRANSFER_TOOLS` lives in a `.svelte` file**, so six verbs are the only
  equipment with no module and no unit test.

## 3. The target

**One cupboard.** `InstrumentCupboard.svelte`: a modal built from one model,
opened from one small button, showing rendered items on shelves grouped by
**what they do**.

- **One model** — `equipmentCatalogue.ts` merges `INSTRUMENTS`, `APPARATUS`,
  the transfer verbs (lifted out of the component into `transferTools.ts`),
  `burette`/`mix`/`transport`/`react`, and `KIDS_EQUIPMENT`. Each entry:
  `{ id, group, name, blurb, render, action, info }`, where `id` is already
  the catalog id space `equipmentAccess()` keys on (`measure:<token>` for
  instruments, the bare verb otherwise) and `render` is either a `ToolIcon`
  name or an instrument glyph. `action` is a five-way union that maps one to
  one onto the handlers `App.svelte` already passes down:
  `measure` · `install` · `transfer` · `mix` · `burette`. Nothing new is
  routed; the cupboard is a different way to reach today's paths.
- **Five shelves**, by what the thing does:
  *messen* (12 instruments) · *erwärmen & kühlen* (bunsen, hotplate, cooling
  bath, evaporating dish) · *enthalten & verbinden* (balloon, carrier-gas
  line, burette, mixer, wash bottle) · *trennen* (filter, decant, drain,
  magnet, still, column train, mortar) · *antreiben* (stirrer, electrodes,
  lamp, centrifuge, voltmeter, curated reaction). Plus a sixth **sets** shelf
  for the 5 kits, which become alias entries pointing at an existing `id`
  rather than a parallel list — so a kit inherits its availability and its
  action from the tool it is a skin over, and only overrides name, icon and
  `preset`.
- **Each item is a rendered picture, its name below it, and an `(i)`.** The
  `(i)` reuses `InfoToggle`/`InfoPanel` (#453, already the kit and shelf
  pattern) and answers the two questions a card cannot: *what it models* and
  *what it does not*. Kits already carry that sentence (`boundary`); apparatus
  and instruments gain one, keyed in `de.json` like every other string.
- **Selecting one does exactly what it does today**: a measurement runs now on
  the selected vessel and closes the cupboard; an apparatus installs on the
  bench and opens its form; a transfer or the mixer arms and sends the learner
  to the bench to pick vessels.
- **One button, no new row.** It sits at the right end of the MESSEN strip,
  inside `InstrumentTray`'s own row (sticky right, so it never scrolls away),
  and costs no vertical space. `Inspector.svelte` is owned by another lane, so
  the tray raises the request through a tiny `instrumentSurface.svelte.ts`
  state module that `App.svelte` mounts the modal from — no prop-drill through
  `Inspector`. The dock keeps both of its buttons, but *Geräteschrank* now
  opens the cupboard rather than switching a tab, and *Messgeräte* still opens
  the inspector — which is where the quick-access row lives.
- **The MESSEN strip stays, as quick access.** It shows the learner's ~6 most
  recently used instruments, fed from the same model and persisted in
  `localStorage` (`kerotakis.instruments.recent`), seeded with the six most
  common measurements (look, thermometer, pH, balance, gas volume,
  conductivity). Six pills fit without scrolling; everything else is one tap
  away in the cupboard, which is where the full list belongs.
- **Locked and loaned shown in place**, unchanged: `equipmentAccess()` +
  `requirement()`, the `⌁ nach N Missionen` label and the *Missionsset* badge,
  on the item where it is, not by hiding it.

## 4. Migration

All three shipped on 2026-09-06.

| PR | GUI | Did | Deleted |
| --- | --- | --- | --- |
| 1 | GUI-100 | this document, roadmap entries | — |
| 2 | GUI-101 | `equipmentCatalogue.ts`, `transferTools.ts`, `instrumentRecents.ts`, `instrumentSurface.svelte.ts`, `InstrumentCupboard.svelte`, the button in the MESSEN row, the quick-access strip; `EquipmentCabinet` stayed mounted but rendered from the same model | the in-component `TRANSFER_TOOLS` |
| 3 | GUI-102 | the shelf pane is the reagent shelf and its *equipment* button opens the cupboard; the bench button, the mission debrief, the utility station and the dock all open the same modal; UX gate and docs updated | `EquipmentCabinet.svelte`, `cabinetTab` |

Two things the plan named and the work did not do, both on purpose.
`InstrumentTray.svelte` was **not** renamed to `QuickInstruments.svelte`: it
is mounted by `Inspector.svelte`, which belongs to another lane, and a rename
buys nothing its own documentation does not. And the dock's `look` /
`temperature` / `pH` were left alone — that is open question 2 below, which is
the owner's to answer rather than a deletion PR's.

`tools/test-ux-quality.mjs` asserted only pane geometry (`nav.shelf-pane`,
`.bench-pane`, `main > aside`), so PR 3 added cupboard assertions rather than
repairing broken ones: it opens on desktop and on a phone, groups its contents
on shelves, holds the whole catalogue, names every item and gives every item
an `(i)`.

**Tests pinning the contract** (all in `web/app/src/lib`):

- `equipmentCatalogue.test.ts` — every `INSTRUMENTS` token, `APPARATUS` verb,
  transfer verb, special and kit appears **exactly once**; every entry has a
  group and an action; ids are unique; every kit alias resolves to a real id;
  each group is non-empty.
- `instrumentRecents.test.ts` — most-recent-first, capped at six, unknown ids
  dropped, the default seed used when storage is empty, and a throwing
  `localStorage` still yields the default.
- availability — `equipmentAccess(catalog, entry.id)` answers for every entry,
  including the ungated-verb case (`cool` reachable in Sandbox) that
  `catalogProgress.ts` documents.

## 5. Decisions (2026-09-06)

The owner answered all five. GUI-103 implemented them in one PR; each answer
is followed by the reason it was given, and the code carries the same reason
at the place it constrains.

1. **The MESSEN strip survives, at four, and the dock's three are not
   candidates for it.** Quick access is worth one row when the row is quick:
   membership by recency, position by catalogue order (unchanged), but four
   pills rather than six, because the row also carries the cupboard door and
   a fifth pill pushes that door off a 320 px screen — and the door is what
   the row exists to lead to. `DOCK_INSTRUMENTS` (`look`, `thermometer`,
   `pH`) are excluded from the candidates entirely, so a strip of four
   carries strictly more than the old six did: nine instruments compete for
   four slots instead of six slots of which three restated the dock. The
   seed became `balance` · `volume` · `conductivity` · `pressure`.

2. **The dock keeps its fixed `look` / `temperature` / `pH`.** They were the
   obvious thing to replace with quick access, which is exactly why they
   stay: a landmark that moves is not a landmark. These three sit in the same
   place on every vessel in every session whatever was measured last, and the
   strip's whole contract is that its membership changes with use. The
   duplication that argument would otherwise leave is answered by decision 1
   — the strip excludes them, so nothing is offered twice on one screen.

3. **The kits are a filter chip in the cupboard header — not a shelf, not a
   mode.** As a shelf they showed the candle twice, which is the bug this
   whole document is about. As a mode they would hide every tool no kit
   happens to name. The chip (default off, remembered in `localStorage` under
   `kerotakis.equipment.sets`) renames the slots the sets stand for — name,
   picture, parts, purpose and `preset`, because "Kerze und Docht" opening
   the flame panel on a laboratory burner's default would be a candle in name
   only — and leaves every other tool exactly as it is. The `aliasOf` entries
   stay the data source; they are simply drawn INTO a tool's slot rather than
   beside it, so the candle and the balloon have one slot in either state.

4. **Five shelves.** *antreiben* was the heading that did not answer "what do
   I want to do": a stirrer, a centrifuge, a lamp, a pair of electrodes and
   the reaction studio share a word, not a purpose. Its members went to the
   neighbours that already described them, and each shelf now carries one
   sentence saying what lives on it, on the heading's tooltip and in the tip
   strip a touch screen can reach.

   | Shelf | German | What stands on it |
   | --- | --- | --- |
   | observe and measure | Beobachten und messen | the 12 instruments |
   | heat and cool | Erhitzen und Kühlen | bunsen, heat, cool, evaporate |
   | prepare and convert | Vorbereiten und Umsetzen | dilute, grind, stir, centrifuge, irradiate, react |
   | contain and connect | Einschließen und Verbinden | regulate, sweep, electrolyse, cell, burette, mix, transport |
   | transfer and separation | Überführen und Trennen | filter, decant, drain, magnet, distil |

   The mortar moved off separation because grinding is what a learner does
   *before* a separation; electrolysis and the half-cell sit with the tubing
   and the gas line because the thing to get right about them is the wiring;
   and the separation shelf is now exactly the two-vessel verbs, minus `cell`
   — which moves nothing and only reads what is between two half-cells.

5. **Both defects are fixed here.** `directActions.ts` names the heat source
   even when it is the bench default (`heat v1 10kJ on burner`, from the same
   `heatSource()` table `ApparatusForm` builds its line from) rather than
   claiming a burner by omission; `cool` keeps a bare line, because the
   grammar has no clause to name a cooling bath with. And the cupboard's
   denominator is a constant `GATED_IDS` over every tool a learner can ever
   have, `react` included whether or not this session offers a curated
   reaction — the old list dropped it, so the fraction read "31/33" and then
   "32/34" one command later. The fraction is now printed only while
   something is still locked: in Sandbox everything is reachable and "34/34"
   is a fact rather than progress.

**Tests added with the decisions.** `instrumentRecents.test.ts` — four, the
dock's three excluded from candidates, seed and row order, and the dock's
rendered `measure` lines pinned to `DOCK_INSTRUMENTS` so the two hand-written
lists cannot drift. `equipmentCatalogue.test.ts` — five shelves each with a
sentence, the moved verbs at their new addresses, a kit standing on its
tool's shelf, the candle exactly once in both chip states, a tool no set
names untouched by the chip, and the denominator's independence from `react`.
`directActions.test.ts` — the `on burner` clause and the bare `cool` line.
`tools/test-ux-quality.mjs` — five shelves, every shelf explained, a
catalogue of 34, the tally shown only while something is locked, the chip
renaming rather than adding a slot, and a MESSEN strip of at most four with
the cupboard door and none of the dock's three.
