import type { IgnitionFlameUniforms } from "./ignitionFlameUniforms";

/**
 * Packed WGSL uniform layout (32 bytes): resolution, time, intensity, colour,
 * seed. Callers own and reuse the backing array; updating a frame allocates
 * nothing and requires no GPU readback or IPC.
 */
export const IGNITION_FLAME_UNIFORM_FLOATS = 8;
export const IGNITION_FLAME_UNIFORM_BYTES = IGNITION_FLAME_UNIFORM_FLOATS * Float32Array.BYTES_PER_ELEMENT;
/** Hard renderer-local limits; the visual tier never earns an unbounded target. */
export const IGNITION_FLAME_MAX_DIMENSION = 2048;
export const IGNITION_FLAME_TIME_PERIOD_SECONDS = 1024;

const finiteOr = (value: number, fallback: number): number => (Number.isFinite(value) ? value : fallback);
const unit = (value: number): number => Math.max(0, Math.min(1, finiteOr(value, 0)));

export function writeIgnitionFlameUniformBuffer(
  target: Float32Array,
  uniforms: Readonly<IgnitionFlameUniforms>,
  elapsedSeconds: number,
  width: number,
  height: number,
): void {
  if (target.length < IGNITION_FLAME_UNIFORM_FLOATS) {
    throw new RangeError(`ignition flame uniform buffer needs ${IGNITION_FLAME_UNIFORM_FLOATS} floats`);
  }

  target[0] = Math.max(1, Math.min(IGNITION_FLAME_MAX_DIMENSION, finiteOr(width, 1)));
  target[1] = Math.max(1, Math.min(IGNITION_FLAME_MAX_DIMENSION, finiteOr(height, 1)));
  const safeElapsed = Math.max(0, finiteOr(elapsedSeconds, 0));
  target[2] = safeElapsed % IGNITION_FLAME_TIME_PERIOD_SECONDS;
  target[3] = uniforms.active ? unit(uniforms.intensity) : 0;
  target[4] = unit(uniforms.colour[0]);
  target[5] = unit(uniforms.colour[1]);
  target[6] = unit(uniforms.colour[2]);
  target[7] = unit(uniforms.seed);
}

/**
 * Project-owned flame envelope. The advected, curling silhouette is inspired
 * by the visual principles in Nguyen, Fedkiw & Jensen, “Physically Based
 * Modeling and Animation of Fire” (SIGGRAPH 2002), DOI 10.1145/566654.566643.
 * This is an independent, compact analytic shader; no source or equations were
 * copied from that work or from the jeantimex/fluid ports.
 */
export const IGNITION_FLAME_WGSL = /* wgsl */ `
// Independent Kerotakis implementation inspired by Nguyen, Fedkiw & Jensen,
// "Physically Based Modeling and Animation of Fire" (SIGGRAPH 2002).
// No source code or equations were copied from the paper or other renderers.
struct FlameUniforms {
  resolution: vec2<f32>,
  time: f32,
  intensity: f32,
  colour: vec3<f32>,
  seed: f32,
};

@group(0) @binding(0) var<uniform> flame: FlameUniforms;

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
};

@vertex
fn vertex_main(@builtin(vertex_index) index: u32) -> VertexOutput {
  var positions = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(3.0, -1.0),
    vec2<f32>(-1.0, 3.0),
  );
  var output: VertexOutput;
  output.position = vec4<f32>(positions[index], 0.0, 1.0);
  output.uv = output.position.xy * 0.5 + vec2<f32>(0.5);
  return output;
}

fn hash21(point: vec2<f32>) -> f32 {
  let mixed = fract(point * vec2<f32>(123.34, 456.21));
  return fract((mixed.x + mixed.y) * (mixed.x * mixed.y + 45.32));
}

fn value_noise(point: vec2<f32>) -> f32 {
  let cell = floor(point);
  let local = fract(point);
  let blend = local * local * (vec2<f32>(3.0) - 2.0 * local);
  let lower = mix(hash21(cell), hash21(cell + vec2<f32>(1.0, 0.0)), blend.x);
  let upper = mix(hash21(cell + vec2<f32>(0.0, 1.0)), hash21(cell + vec2<f32>(1.0)), blend.x);
  return mix(lower, upper, blend.y);
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
  // Correct for non-square bounded canvases without sampling outside them.
  let aspect = flame.resolution.x / max(flame.resolution.y, 1.0);
  var point = vec2<f32>((input.uv.x - 0.5) * aspect, input.uv.y);
  let phase = flame.time * 1.7 + flame.seed * 31.0;
  let rise = point.y * 6.0 - phase;
  let curl = value_noise(vec2<f32>(point.x * 7.0 + flame.seed * 13.0, rise));
  point.x += (curl - 0.5) * (0.15 + point.y * 0.12);

  let body_width = mix(0.25, 0.035, smoothstep(0.05, 0.96, point.y));
  let edge = abs(point.x) / max(body_width, 0.001) + point.y * point.y;
  let envelope = (1.0 - smoothstep(0.68, 1.12, edge))
    * smoothstep(0.0, 0.08, point.y)
    * (1.0 - smoothstep(0.82, 1.0, point.y));
  let strength = envelope * clamp(flame.intensity, 0.0, 1.0);

  let hot_core = (1.0 - smoothstep(0.05, 0.62, edge)) * (1.0 - point.y * 0.55);
  let colour = mix(flame.colour * 0.48, flame.colour, hot_core)
    + vec3<f32>(1.0, 0.42, 0.08) * hot_core * 0.34;
  return vec4<f32>(clamp(colour * strength, vec3<f32>(0.0), vec3<f32>(1.0)), strength);
}
`;
