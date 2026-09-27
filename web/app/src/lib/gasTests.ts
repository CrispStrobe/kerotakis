/**
 * What a gas test is CALLED, from whichever token names it.
 *
 * Two vocabularies reach this. The Inspector sends the command's word
 * (`splint`, `litmus`); the engine reports back its own (`glowing_splint`,
 * `damp_litmus`). Both used to go straight to `t()`, which had two
 * consequences.
 *
 * The obvious one is that a wire token is not a sentence — GUI-124's
 * lesson, in a second place.
 *
 * The quiet one is that **`limewater` became the dictionary key for both
 * the substance and the test for it.** `i18n.svelte` merges `terms` under
 * `messages`, so the test's name won: German called a bottle of limewater
 * *Kalkwasserprobe*, the test. `data/kids/experiments-v1.json` lists
 * `limewater` as an INGREDIENT of the breath test, so that name was
 * reaching a shelf and a materials list, where you do not pour a test
 * into a beaker.
 *
 * One English string, one meaning. The test's key says it is a test, and
 * `limewater` goes back to being the substance it always was in `terms`.
 */
export const GAS_TEST_LABEL: Readonly<Record<string, string>> = {
  pop: "pop test",
  splint: "glowing splint test",
  glowing_splint: "glowing splint test",
  limewater: "limewater test",
  litmus: "damp litmus test",
  damp_litmus: "damp litmus test",
};

/**
 * A gas test's name, or the token's own words for one nobody has named.
 *
 * An unknown token renders as words rather than as a tag — the same
 * degradation `expectationLabel` uses, and for the same reason: a missing
 * name should cost one label, never the row it sits in.
 */
export function gasTestLabel(token: string): string {
  return GAS_TEST_LABEL[token] ?? token.replace(/_/g, " ");
}
