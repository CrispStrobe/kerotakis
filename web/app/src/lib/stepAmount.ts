/** Step an amount by something proportional to its size.
 *
 * A fixed step is wrong at both ends of this range: 0.5 → 1.5 in one tap
 * overshoots what anyone meant, and 100 → 101 is a tap that achieves
 * nothing. Amounts here span roughly 0.001 g to 1000 mL.
 *
 * So the step is a round number near a tenth of the current value —
 * 1, 2, 5 × a power of ten — which keeps the result readable rather than
 * landing on 1.0999999999999999.
 */
export function stepAmount(value: number, direction: 1 | -1): number {
  const current = Number.isFinite(value) && value > 0 ? value : 1;
  const magnitude = Math.pow(10, Math.floor(Math.log10(current / 10)));
  const scaled = current / 10 / magnitude;
  const step = (scaled >= 5 ? 5 : scaled >= 2 ? 2 : 1) * magnitude;

  // Stepping down from exactly one step must not reach zero: an amount of
  // nothing is not a thing anyone wants to add, and the input rejects it.
  const next = current + direction * step;
  if (next <= 0) return current / 2;

  // Rounded to the step's own precision, which kills the float dust that
  // would otherwise show 1.0999999999999999 in a field the reader is
  // looking at. Deliberately NOT snapped to a grid: snapping made the
  // buttons asymmetric — 2.5 up then down came back 2.6, because 2.5 is
  // not on the 0.2 grid that its own magnitude implies. A control that
  // does not return where it started is worse than a coarse one.
  const decimals = Math.max(0, -Math.floor(Math.log10(step)) + 1);
  return Number(next.toFixed(decimals));
}
