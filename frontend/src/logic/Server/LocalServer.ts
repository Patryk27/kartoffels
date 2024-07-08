import SandboxWorker from "./LocalServer/SandboxWorker?worker";
import type { Server, ServerUpdate } from "@/logic/Server";

export class LocalServer implements Server {
  private openFn?: () => void;
  private closeFn?: () => void;
  private errorFn?: () => void;
  private updateFn?: (msg: ServerUpdate) => void;

  private joinResponseFn?: (response: any) => void;
  private joinListenerIdx?: number;
  private uploadBotResponseFn?: (response: any) => void;

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
            this.joinResponseFn = undefined;
          }

          break;

        case "join.update":
          if (this.updateFn && this.joinListenerIdx == msg.listenerIdx) {
            this.updateFn(msg.event);
          }

          break;

        case "uploadBot.response":
          if (this.uploadBotResponseFn) {
            this.uploadBotResponseFn(msg.response);
            this.uploadBotResponseFn = undefined;
          }

          break;
      }
    };
  }

  join(_: string, botId?: string): void {
    // Control flow might seem a bit arbitrary in here, but just keep in mind we
    // are trying to replicate how `RemoteServer` works, which - in turn - works
    // the way it works, because WebSockets.
    //
    // So:
    // - a successful join will fire `openFn` once and then it will continuously
    //   fire `messageFn`,
    // - a failing join will fire `errorFn` once.

    this.worker.postMessage({
      op: "join",
      botId,
    });

    this.joinResponseFn = (response) => {
      if (response.status == "ok") {
        this.joinListenerIdx = response.result.listenerIdx;

        if (this.openFn) {
          this.openFn();
        }
      } else {
        if (this.errorFn) {
          this.errorFn();
        }
      }
    };
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

  leave(): void {
    this.worker.postMessage({
      op: "leave",
    });

    if (this.closeFn) {
      this.closeFn();
    }

    this.openFn = undefined;
    this.closeFn = undefined;
    this.updateFn = undefined;
    this.joinListenerIdx = undefined;
  }

  close(): void {
    this.worker.terminate();
  }

  onOpen(f: () => void): void {
    this.openFn = f;
  }

  onClose(f: () => void): void {
    this.closeFn = f;
  }

  onError(f: () => void): void {
    this.errorFn = f;
  }

  onUpdate(f: (msg: ServerUpdate) => void) {
    this.updateFn = f;
  }
}
