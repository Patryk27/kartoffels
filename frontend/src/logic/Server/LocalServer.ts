import type {
  Server,
  ServerEvent,
  ServerConnMsg,
  ServerBotInfo,
} from "@/logic/Server";
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

  async listen(): Promise<ReadableStream<ServerEvent>> {
    log("listen()");

    const sandbox = await this.getSandbox();
    const events = await sandbox.listen();

    return events;
  }

  async join(botId?: string): Promise<ReadableStream<ServerConnMsg>> {
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

  createBot(src: File): Promise<{ id: string }> {
    log("createBot()");

    return new Promise((resolve, reject) => {
      const reader = new FileReader();

      reader.onload = async () => {
        if (!(reader.result instanceof ArrayBuffer)) {
          return;
        }

        const src = new Uint8Array(reader.result);
        const sandbox = await this.getSandbox();

        sandbox
          .create_bot(src)
          .then((id) => {
            resolve({ id });
          })
          .catch(reject);
      };

      reader.readAsArrayBuffer(src);
    });
  }

  async createPrefabBot(
    ty: string,
    x?: number,
    y?: number,
    ephemeral?: boolean,
  ): Promise<{ id: string }> {
    log("createPrefabBot()", ty, x, y, ephemeral);

    const sandbox = await this.getSandbox();
    const id = await sandbox.create_prefab_bot(ty, x, y, ephemeral);

    return { id };
  }

  async destroyBot(id: string): Promise<void> {
    log("destroyBot()", id);

    const sandbox = await this.getSandbox();

    await sandbox.destroy_bot(id);
  }

  async destroyAllBots(): Promise<void> {
    for (const bot of await this.getBots()) {
      await this.destroyBot(bot.id);
    }
  }

  async restartBot(id: string): Promise<void> {
    log("restartBot()", id);

    const sandbox = await this.getSandbox();

    await sandbox.restart_bot(id);
  }

  async getBots(): Promise<ServerBotInfo[]> {
    log("getBots()");

    const sandbox = await this.getSandbox();

    return await sandbox.get_bots();
  }

  async setSpawnPoint(x?: number, y?: number): Promise<void> {
    log("setSpawnPoint()", x, y);

    const sandbox = await this.getSandbox();

    sandbox.set_spawn_point(x, y);
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
