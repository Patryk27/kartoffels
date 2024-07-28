import SandboxWorker from "./LocalServer/SandboxWorker?worker";
import type { Server, ServerMessage } from "@/logic/Server";

export class LocalServer implements Server {
  private messageFn?: (msg: ServerMessage) => void;

  private joinResponseFn?: (response: any) => void;
  private joinListenerIdx?: number;
  private uploadBotResponseFn?: (response: any) => void;
  private spawnPrefabBotResponseFn?: (response: any) => void;

  private worker: Worker;

  constructor(config: any) {
    this.recreate(config);
  }

  recreate(config: any): void {
    if (this.worker) {
      this.worker.terminate();
    }

    this.worker = new SandboxWorker();

    this.worker.postMessage({
      op: "init",
      config,
    });

    this.worker.onmessage = (event: any): void => {
      const msg = event.data;

      switch (msg.op) {
        case "join.response":
          if (this.joinResponseFn) {
            this.joinResponseFn(msg.response);
            this.joinResponseFn = null;
          }

          break;

        case "join.update":
          if (this.messageFn && this.joinListenerIdx == msg.listenerIdx) {
            this.messageFn(msg.event);
          }

          break;

        case "uploadBot.response":
          if (this.uploadBotResponseFn) {
            this.uploadBotResponseFn(msg.response);
            this.uploadBotResponseFn = null;
          }

          break;

        case "spawnPrefabBot.response":
          if (this.spawnPrefabBotResponseFn) {
            this.spawnPrefabBotResponseFn(msg.response);
            this.spawnPrefabBotResponseFn = null;
          }
      }
    };
  }

  async join(botId?: string): Promise<void> {
    this.worker.postMessage({
      op: "join",
      botId,
    });

    return new Promise((resolve, reject) => {
      this.joinResponseFn = (response) => {
        if (response.status == "ok") {
          this.joinListenerIdx = response.result.listenerIdx;

          resolve();
        } else {
          reject();
        }
      };
    });
  }

  pause(paused: boolean): void {
    this.worker.postMessage({
      op: "pause",
      paused,
    });
  }

  leave(): void {
    this.worker.postMessage({
      op: "leave",
    });

    this.messageFn = null;
    this.joinListenerIdx = null;
  }

  close(): void {
    this.worker.terminate();
  }

  uploadBot(file: File): Promise<{ id: string }> {
    const reader = new FileReader();

    reader.onload = () => {
      if (reader.result instanceof ArrayBuffer) {
        this.worker.postMessage({
          op: "uploadBot",
          src: new Uint8Array(reader.result),
        });
      }
    };

    reader.readAsArrayBuffer(file);

    return new Promise((resolve, reject) => {
      this.uploadBotResponseFn = (response) => {
        if (response.status == "ok") {
          resolve(response.result);
        } else {
          reject(response.error);
        }
      };
    });
  }

  spawnPrefabBot(ty: string): Promise<{ id: string }> {
    this.worker.postMessage({
      op: "spawnPrefabBot",
      ty,
    });

    return new Promise((resolve, reject) => {
      this.spawnPrefabBotResponseFn = (response) => {
        if (response.status == "ok") {
          resolve(response.result);
        } else {
          reject(response.error);
        }
      };
    });
  }

  destroyBot(id: string): void {
    this.worker.postMessage({
      op: "destroyBot",
      id,
    });
  }

  restartBot(id: string): void {
    this.worker.postMessage({
      op: "restartBot",
      id,
    });
  }

  setSpawnPoint(x?: number, y?: number): void {
    this.worker.postMessage({
      op: "setSpawnPoint",
      x,
      y,
    });
  }

  onMessage(f: (msg: ServerMessage) => void) {
    this.messageFn = f;
  }

  onStatusChange(_: (status: string) => void): void {
    // no-op
  }
}
