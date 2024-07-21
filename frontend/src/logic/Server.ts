import { LocalServer } from "./Server/LocalServer";
import { RemoteServer } from "./Server/RemoteServer";

// Connection, either to the remote server (via Web Sockets) or simulated within
// the browser (aka sandbox).
//
// @see kartoffels::Handle
export interface Server {
  join(botId?: string): Promise<void>;
  leave(): void;
  close(): void;
  uploadBot(file: File): Promise<{ id: string }>;

  onMessage(f: (msg: ServerMessage) => void): void;
  onStatusChange(f: (status: string) => void): void;
}

export interface ServerMessage {
  map?: ServerMapUpdate;
  mode?: ServerModeUpdate;
  bots?: ServerBotsUpdate;
  bot?: ServerConnectedBotUpdate;
}

export interface ServerMapUpdate {
  size: [number, number];
  tiles: number[];
}

export interface ServerModeUpdate {
  //
}

export interface ServerBotsUpdate {
  [index: string]: ServerBotUpdate;
}

export interface ServerBotUpdate {
  pos: [number, number];
  dir: "^" | ">" | "v" | "<";
  age: number;
}

export type ServerConnectedBotUpdate =
  | {
      status: "queued";
      place: number;
      requeued: number;
      events: BotEvent[];
    }
  | {
      status: "alive";
      age: number;
      serial: [number];
      events: BotEvent[];
    }
  | {
      status: "dead";
      events: BotEvent[];
    };

export interface BotEvent {
  at: Date;
  msg: string;
}

export interface ServerGetWorldsResponse {
  worlds: ServerWorld[];
}

export interface ServerWorld {
  id: string;
  name: string;
  mode: string;
  theme: string;
}

export { LocalServer, RemoteServer };
