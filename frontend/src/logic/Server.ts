import { LocalServer } from "./Server/LocalServer";
import { RemoteServer } from "./Server/RemoteServer";

// Connection, either to the remote server (via Web Sockets) or simualted within
// the web browser (aka sandbox).
//
// @see kartoffels::Handle
//
// TODO clean-up the control flow (make `join()` return just a Promise?)
export interface Server {
  join(worldId: string, botId?: string): void;
  leave(): void;
  close(): void;
  uploadBot(file: File): Promise<{ id: string }>;

  onOpen(f: () => void): void;
  onClose(f: () => void): void;
  onError(f: () => void): void;
  onUpdate(f: (msg: ServerUpdate) => void): void;
}

export interface ServerUpdate {
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
