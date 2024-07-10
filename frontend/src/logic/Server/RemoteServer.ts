import type { Server, ServerUpdate } from "@/logic/Server";

export class RemoteServer implements Server {
  private socket?: WebSocket;
  private worldId?: string;

  join(worldId: string, botId?: string): void {
    this.socket = new WebSocket(
      botId == null
        ? `${import.meta.env.VITE_WS_URL}/worlds/${worldId}`
        : `${import.meta.env.VITE_WS_URL}/worlds/${worldId}/bots/${botId}`,
    );

    this.worldId = worldId;
  }

  leave(): void {
    this.close();
  }

  close(): void {
    if (this.socket) {
      this.socket.close();
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

  onOpen(f: () => void): void {
    if (this.socket) {
      this.socket.onopen = f;
    }
  }

  onClose(f: () => void): void {
    if (this.socket) {
      this.socket.onclose = f;
    }
  }

  onError(f: () => void): void {
    if (this.socket) {
      this.socket.onerror = f;
    }
  }

  onUpdate(f: (msg: ServerUpdate) => void): void {
    if (this.socket) {
      this.socket.onmessage = (event) => {
        f(JSON.parse(event.data));
      };
    }
  }
}
