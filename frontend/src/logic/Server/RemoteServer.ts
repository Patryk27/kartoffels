import type { Server, ServerConnMsg } from "@/logic/Server";

export class RemoteServer implements Server {
  private httpUrl: string;
  private wsUrl: string;
  private socket?: Socket;
  private reconnectFn?: (status: string) => void;

  constructor(worldId: string) {
    this.httpUrl = `${import.meta.env.VITE_HTTP_URL}/worlds/${worldId}`;
    this.wsUrl = `${import.meta.env.VITE_WS_URL}/worlds/${worldId}`;

    log(`httpUrl = ${this.httpUrl}`);
    log(`wsUrl = ${this.wsUrl}`);
  }

  async join(botId?: string): Promise<ReadableStream<ServerConnMsg>> {
    log("join()", botId);

    const socket = new Socket(
      botId == null ? `${this.wsUrl}` : `${this.wsUrl}/bots/${botId}`,
    );

    socket.onReconnect((status) => {
      if (this.reconnectFn) {
        this.reconnectFn(status);
      }
    });

    const msgs = new ReadableStream({
      start(ctrl) {
        socket.onMessage((event) => {
          ctrl.enqueue(JSON.parse(event.data));
        });
      },

      cancel() {
        socket.close();
      },
    });

    await socket.connect();

    this.socket = socket;

    return msgs;
  }

  async close(): Promise<void> {
    log("close()");

    if (this.socket) {
      this.socket.close();
    }
  }

  async createBot(src: File): Promise<{ id: string }> {
    log("uploadBot()");

    const response = await fetch(`${this.httpUrl}/bots`, {
      method: "POST",
      body: src,
    });

    if (response.status == 200) {
      return await response.json();
    } else {
      throw await response.text();
    }
  }

  onReconnect(f: (status: string) => void): void {
    this.reconnectFn = f;
  }
}

class Socket {
  private url: string;
  private isClosing: boolean;
  private socket?: WebSocket;
  private messageFn?: (ev: MessageEvent) => void;
  private reconnectFn?: (status: string) => void;

  constructor(url: string) {
    this.url = url;
    this.isClosing = false;
  }

  connect(): Promise<void> {
    log("connecting");

    if (this.socket) {
      this.socket.onclose = null;
      this.socket.onerror = null;
      this.socket.onmessage = null;
      this.socket.close();
    }

    this.socket = new WebSocket(this.url);

    return new Promise((resolve, reject) => {
      this.socket.onopen = () => {
        log("connected");

        resolve(null);

        this.socket.onmessage = (ev) => {
          if (this.messageFn) {
            this.messageFn(ev);
          }
        };

        this.socket.onclose = () => {
          this.reconnect();
        };

        this.socket.onerror = () => {
          this.reconnect();
        };
      };

      this.socket.onclose = reject;
      this.socket.onerror = reject;
    });
  }

  close(): void {
    log("closing");

    this.isClosing = true;
    this.socket.close();
  }

  onMessage(f: (ev: MessageEvent) => void): void {
    this.messageFn = f;
  }

  onReconnect(f: (status: string) => void): void {
    this.reconnectFn = f;
  }

  private async reconnect(): Promise<void> {
    if (this.isClosing) {
      return;
    }

    if (this.reconnectFn) {
      this.reconnectFn("reconnecting");
    }

    while (true) {
      try {
        await this.connect();
        break;
      } catch (err) {
        await new Promise((resolve) => {
          setTimeout(resolve, 250);
        });
      }
    }

    if (this.reconnectFn) {
      this.reconnectFn("connected");
    }
  }
}

function log(...data: any[]) {
  console.log("[remote-server]", ...data);
}
