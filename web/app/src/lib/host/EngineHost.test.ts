import { describe, expect, it } from "vitest";
import { EngineError, RequestChannel, type MessagePortLike } from "./EngineHost";
import { WorkerHost } from "./WorkerHost";

/** A port whose far end is scripted by the test. */
class FakePort implements MessagePortLike {
  sent: Array<Record<string, unknown>> = [];
  handler: ((ev: { data: unknown }) => void) | null = null;
  set onmessage(h: ((ev: { data: unknown }) => void) | null) {
    this.handler = h;
  }
  postMessage(data: unknown) {
    this.sent.push(data as Record<string, unknown>);
  }
  reply(data: unknown) {
    this.handler?.({ data });
  }
}

describe("RequestChannel", () => {
  it("correlates responses by id, out of order", async () => {
    const port = new FakePort();
    const channel = new RequestChannel(port);
    const a = channel.request("scene");
    const b = channel.request("state");
    const [idA, idB] = [port.sent[0]!.id, port.sent[1]!.id];
    port.reply({ id: idB, type: "done", result_json: '"state"' });
    port.reply({ id: idA, type: "done", result_json: '"scene"' });
    expect(await b).toBe('"state"');
    expect(await a).toBe('"scene"');
  });

  it("streams progress without resolving, then resolves on done", async () => {
    const port = new FakePort();
    const channel = new RequestChannel(port);
    const seen: number[] = [];
    const p = channel.request("run_script", { script: "…" }, (f) => seen.push(f));
    const id = port.sent[0]!.id;
    port.reply({ id, type: "progress", fraction: 0.5, message: "equilibrating" });
    port.reply({ id, type: "done", result_json: "{}" });
    expect(await p).toBe("{}");
    expect(seen).toEqual([0.5]);
  });

  it("turns error envelopes into EngineError with its kind", async () => {
    const port = new FakePort();
    const channel = new RequestChannel(port);
    const p = channel.request("step", { operator_json: "nonsense" });
    port.reply({ id: port.sent[0]!.id, type: "error", message: "no verb", kind: "parse" });
    await expect(p).rejects.toMatchObject({ name: "EngineError", kind: "parse" });
  });

  it("ignores stray ids and malformed frames", async () => {
    const port = new FakePort();
    const channel = new RequestChannel(port);
    const p = channel.request("scene");
    port.reply(null);
    port.reply({ id: 999, type: "done", result_json: "{}" });
    port.reply({ id: port.sent[0]!.id, type: "done", result_json: "1" });
    expect(await p).toBe("1");
  });

  it("abandon() rejects everything in flight", async () => {
    const port = new FakePort();
    const channel = new RequestChannel(port);
    const p = channel.request("scene");
    channel.abandon("host disposed");
    await expect(p).rejects.toBeInstanceOf(EngineError);
  });
});

describe("WorkerHost", () => {
  it("initializes the worker, then speaks the command table", async () => {
    const port = new FakePort();
    const host = new WorkerHost(port);
    // First envelope is init with the engine base.
    expect(port.sent[0]).toMatchObject({ cmd: "init" });

    const scene = host.scene();
    const sceneReq = port.sent.find((m) => m.cmd === "scene")!;
    port.reply({
      id: sceneReq.id,
      type: "done",
      result_json: JSON.stringify({ scene: 1, vessels: [] }),
    });
    expect(await scene).toEqual({ scene: 1, vessels: [] });

    const step = host.step('{"op":"look"}');
    const stepReq = port.sent.find((m) => m.cmd === "step")!;
    expect(stepReq.operator_json).toBe('{"op":"look"}');
    port.reply({
      id: stepReq.id,
      type: "done",
      result_json: JSON.stringify({ events: [], rendered: ["nothing happens"] }),
    });
    expect((await step).rendered).toEqual(["nothing happens"]);
  });
});
