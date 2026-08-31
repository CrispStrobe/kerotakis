// The adapter owns the browser-only dynamic import. Keeping this entry tiny
// lets the disposable route remain SSR-safe and load 3Dmol only on demand.
export { threeDMolAdapter as adapter, threeDMolAdapter as default } from "./3dmolAdapter";
