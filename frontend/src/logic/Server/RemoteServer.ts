import type { Server, ServerMessage } from "@/logic/Server";

export class RemoteServer implements Server {
  private worldId?: string;

  private messageFn?: (msg: ServerMessage) => void;
  private statusChangeFn?: (status: string) => void;

  private socket?: WebSocket;

  constructor(worldId: string) {
    this.worldId = worldId;
  }

  join(botId?: string): Promise<void> {
    this.socket = new WebSocket(
      botId == null
        ? `${import.meta.env.VITE_WS_URL}/worlds/${this.worldId}`
        : `${import.meta.env.VITE_WS_URL}/worlds/${this.worldId}/bots/${botId}`,
    );

    this.socket.onmessage = (event) => {
      if (this.messageFn) {
        this.messageFn(JSON.parse(event.data));
      }
    };

    return new Promise((resolve, reject) => {
      this.socket.onopen = () => {
        const reconnect = async () => {
          if (this.statusChangeFn) {
            this.statusChangeFn("reconnecting");
          }

          while (true) {
            try {
              await this.join(botId);
              break;
            } catch (err) {
              await new Promise((resolve) => {
                setTimeout(resolve, 250);
              });
            }
          }

          if (this.statusChangeFn) {
            this.statusChangeFn("connected");
          }
        };

        this.socket.onclose = reconnect;
        this.socket.onerror = reconnect;

        resolve();
      };

      this.socket.onclose = () => {
        // Prevent the other handler from firing `reject()` again
        this.socket.onerror = null;

        reject();
      };

      this.socket.onerror = () => {
        // Prevent the other handler from firing `reject()` again
        this.socket.onclose = null;

        reject();
      };
    });
  }

  leave(): void {
    this.close();
  }

  close(): void {
    if (this.socket) {
      this.socket.close();
      this.socket = null;
    }
  }

  async uploadBot(file: File): Promise<{ id: string }> {
    var response = await fetch(
      `${import.meta.env.VITE_HTTP_URL}/worlds/${this.worldId}/bots`,
      {
        method: "POST",
        body: file,
      },
    );

    if (response.status == 200) {
      return await response.json();
    } else {
      throw await response.text();
    }
  }

  onMessage(f: (msg: ServerMessage) => void) {
    this.messageFn = f;
  }

  onStatusChange(f: (status: string) => void): void {
    this.statusChangeFn = f;
  }
}
