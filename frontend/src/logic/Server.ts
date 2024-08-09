import { LocalServer } from "./Server/LocalServer";
import { RemoteServer } from "./Server/RemoteServer";

// Connection, either to the remote server (via Web Sockets) or simulated within
// the browser (aka sandbox).
//
// @see kartoffels::Handle
export interface Server {
  join(botId?: string): Promise<ReadableStream<ServerConnMsg>>;
  close(): Promise<void>;
  createBot(src: File): Promise<{ id: string }>;

  onReconnect(f: (status: string) => void): void;
}

export type ServerEvent = {
  ty: "bot-killed";
  id: string;
};

export interface ServerConnMsg {
  map?: ServerConnMapMsg;
  mode?: ServerConnModeMsg;
  bots?: ServerConnBotsMsg;
  bot?: ServerConnJoinedBotMsg;
}

export interface ServerConnMapMsg {
  size: [number, number];
  tiles: number[];
}

export interface ServerConnModeMsg {
  //
}

export interface ServerConnBotsMsg {
  [index: string]: ServerConnBotMsg;
}

export interface ServerConnBotMsg {
  pos: [number, number];
  dir: "^" | ">" | "v" | "<";
  age: number;
}

export type ServerConnJoinedBotMsg =
  | {
      status: "alive";
      age: number;
      serial: [number];
      events: ServerBotEvent[];
    }
  | {
      status: "dead";
      events: ServerBotEvent[];
    }
  | {
      status: "queued";
      place: number;
      requeued: boolean;
      events: ServerBotEvent[];
    }
  | { status: "unknown" };

export interface ServerBotEvent {
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

export interface ServerBotInfo {
  id: string;
}

export { LocalServer, RemoteServer };
