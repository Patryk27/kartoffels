import type { Server, ServerMsg } from "@/logic/Server";
import init, { Sandbox } from "kartoffels-sandbox";
import wasmUrl from "kartoffels-sandbox/kartoffels_sandbox_bg.wasm?url";

export class LocalServer implements Server {
  private sandbox: Promise<Sandbox> | Sandbox;

  constructor(config: any) {
    this.recreate(config);
  }

  recreate(config: any): void {
    log("recreate()", config);

    this.sandbox = init(wasmUrl).then(() => {
      log("ready");

      return new Sandbox(config);
    });
  }

  async join(botId?: string): Promise<ReadableStream<ServerMsg>> {
    log("join()", botId);

    const sandbox = await this.getSandbox();
    const msgs = await sandbox.join(botId);

    // TODO handle cancellation
    return msgs;
  }

  async pause(paused: boolean): Promise<void> {
    log("pause()", paused);

    const sandbox = await this.getSandbox();

    await sandbox.pause(paused);
  }

  async close(): Promise<void> {
    log("close()");

    const sandbox = await this.getSandbox();

    await sandbox.close();

    this.sandbox = null;
  }

  uploadBot(file: File): Promise<{ id: string }> {
    log("uploadBot()");

    return new Promise((resolve, reject) => {
      const reader = new FileReader();

      reader.onload = async () => {
        if (!(reader.result instanceof ArrayBuffer)) {
          return;
        }

        const src = new Uint8Array(reader.result);
        const sandbox = await this.getSandbox();

        sandbox
          .upload_bot(src)
          .then((id) => {
            resolve({ id });
          })
          .catch(reject);
      };

      reader.readAsArrayBuffer(file);
    });
  }

  async spawnPrefabBot(ty: string): Promise<{ id: string }> {
    log("spawnPrefabBot()", ty);

    const sandbox = await this.getSandbox();
    const id = await sandbox.spawn_prefab_bot(ty);

    return id;
  }

  async destroyBot(id: string): Promise<void> {
    log("destroyBot()", id);

    const sandbox = await this.getSandbox();

    await sandbox.destroy_bot(id);
  }

  async restartBot(id: string): Promise<void> {
    log("restartBot()", id);

    const sandbox = await this.getSandbox();

    await sandbox.restart_bot(id);
  }

  async setSpawnPoint(x?: number, y?: number): Promise<void> {
    log("setSpawnPoint()", x, y);

    const sandbox = await this.getSandbox();

    await sandbox.set_spawn_point(x, y);
  }

  onReconnect(_: (status: string) => void): void {
    // no-op
  }

  private async getSandbox(): Promise<Sandbox> {
    if (this.sandbox instanceof Promise) {
      this.sandbox = await this.sandbox;
    }

    return this.sandbox;
  }
}

function log(...data: any[]) {
  console.log("[local-server]", ...data);
}
