/**
 * Whether an in-flight command belongs to one deployed physical machine.
 *
 * `busy` cannot answer this: the engine is also busy while adding reagents,
 * measuring, changing register, restoring, and running unrelated apparatus.
 * Vessel matching prevents the machine on v1 from animating for the same verb
 * on v2. The burette is the physical implementation of the `titrate` verb.
 */
export function apparatusRunsCommand(
  command: string | null | undefined,
  tool: string | null | undefined,
  target: number | null | undefined,
): boolean {
  if (!command || !tool || target === null || target === undefined) return false;
  const words = command.trim().toLowerCase().split(/\s+/);
  const expectedVerb = tool === "burette" ? "titrate" : tool.toLowerCase();
  if (words[0] !== expectedVerb) return false;
  const targetToken = `v${target + 1}`;
  return words.slice(1).some((word) => word.replace(/[,:;]$/, "") === targetToken);
}
