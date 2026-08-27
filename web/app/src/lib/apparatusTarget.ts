export interface ApparatusDeployment {
  tool: string | null;
  target: number | null;
}

/** Selecting equipment on another vessel is an explicit move; selecting the
 * already-installed card on its own target puts it away. */
export function deploymentAfterChoice(
  currentTool: string | null,
  currentTarget: number | null,
  chosenTool: string,
  selectedVessel: number,
): ApparatusDeployment {
  return currentTool === chosenTool && currentTarget === selectedVessel
    ? { tool: null, target: null }
    : { tool: chosenTool, target: selectedVessel };
}

export function buretteTargetAfterChoice(
  currentTarget: number | null,
  selectedVessel: number,
): number | null {
  return currentTarget === selectedVessel ? null : selectedVessel;
}
