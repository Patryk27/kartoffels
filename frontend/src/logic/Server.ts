import { LocalServer } from "./Server/LocalServer";
import { RemoteServer } from "./Server/RemoteServer";

// Connection, either to the remote server (via Web Sockets) or simulated within
// the browser (aka sandbox).
//
// @see kartoffels::Handle
export interface Server {
  join(botId?: string): Promise<ReadableStream<ConnectionUpdate>>;
  close(): Promise<void>;
  uploadBot(src: File): Promise<{ id: string }>;

  onReconnect(f: (status: string) => void): void;
}

export interface ConnectionUpdate {
  map?: ConnectionMapUpdate;
  mode?: ConnectionModeUpdate;
  bots?: ConnectionBotsUpdate;
  bot?: ConnectionJoinedBotUpdate;
}

export interface ConnectionMapUpdate {
  size: [number, number];
  tiles: number[];
}

export interface ConnectionModeUpdate {
  //
}

export interface ConnectionBotsUpdate {
  [index: string]: ConnectionBotUpdate;
}

export interface ConnectionBotUpdate {
  pos: [number, number];
  dir: "^" | ">" | "v" | "<";
  age: number;
}

export type ConnectionJoinedBotUpdate =
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

export type ServerEvent = {
  ty: "bot-killed";
  id: string;
};

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
