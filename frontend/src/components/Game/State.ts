import {
  type BotEvent,
  type Server,
  type ServerBotsUpdate,
  type ServerConnectedBotUpdate,
  type ServerUpdate,
} from "@/logic/Server";
import type { PlayerBots } from "@/logic/State";
import type { ComputedRef } from "vue";
import { computed, ref, type Ref } from "vue";

export class GameWorld {
  public id: string;
  public name: string;
  public map: Ref<GameMap>;
  public mode: Ref<any>;
  public bot: Ref<GameBot>;
  public bots: Ref<GameBots>;
  public botsTable: ComputedRef<GameTableBot[]>;
  public camera: Ref<GameCamera>;
  public status: Ref<GameConnectionStatus>;

  constructor(id: string, name: string, playerBots: PlayerBots) {
    this.id = id;
    this.name = name;
    this.map = ref<GameMap>(null);
    this.mode = ref(null);
    this.bot = ref<GameBot>(null);
    this.bots = ref<GameBots>(null);
    this.camera = ref<GameCamera>(null);
    this.status = ref<GameConnectionStatus>("connecting");

    this.botsTable = computed(() => {
      let result: GameTableBot[] = [];

      for (const [id, bot] of Object.entries(this.bots.value ?? {})) {
        result.push({
          id,
          age: bot.age,
          score: (this.mode.value ?? {}).scores[id] ?? 0,
          known: playerBots.has(id),
          nth: 0,
        });
      }

      result.sort((a, b) => {
        if (a.score != b.score) {
          return b.score - a.score;
        }

        if (a.age == b.age) {
          return b.age - a.age;
        }

        return b.id.localeCompare(a.id);
      });

      for (let i = 0; i < result.length; i += 1) {
        result[i].nth = i + 1;
      }

      return result;
    });
  }

  join(server: Server, playerBots: PlayerBots, botId?: string): Promise<void> {
    return new Promise((resolve, reject) => {
      const retryJoin = () => {
        this.join(server, playerBots, botId).then(resolve).catch(reject);
      };

      server.onClose(null);
      server.leave();

      this.map.value = null;
      this.mode.value = null;
      this.bot.value = null;
      this.bots.value = null;
      this.camera.value = null;

      this.status.value =
        this.status.value == "reconnecting" ? "reconnecting" : "connecting";

      if (botId) {
        this.bot.value = {
          id: botId,
          following: true,
          status: "unknown",
          events: [],
        };
      }

      server.join(botId);

      server.onOpen(() => {
        this.handleServerOpen(resolve);
      });

      server.onError(() => {
        this.handleServerError(server, botId, reject, retryJoin);
      });

      server.onUpdate((msg) => {
        this.handleServerUpdate(playerBots, msg);
      });

      server.onClose(() => {
        this.handleServerClose(retryJoin);
      });
    });
  }

  private handleServerOpen(resolve: () => void): void {
    this.status.value = "connected";

    resolve();
  }

  private handleServerError(
    server: Server,
    botId: string,
    reject: () => void,
    retryJoin: () => void,
  ): void {
    server.onError(null);
    server.onClose(null);

    if (this.status.value == "reconnecting") {
      setTimeout(retryJoin, 250);
    } else {
      if (botId) {
        alert(`couldn't find bot ${botId}`);

        // LocalServer needs an extra tick before we're able to join() again
        setTimeout(retryJoin, 0);
      } else {
        reject();
      }
    }
  }

  private handleServerUpdate(playerBots: PlayerBots, msg: ServerUpdate): void {
    if (msg.map) {
      this.map.value = {
        size: msg.map.size,
        tiles: msg.map.tiles,
        bots: [],
      };

      this.camera.value = {
        x: Math.round(msg.map.size[0] / 2),
        y: Math.round(msg.map.size[1] / 2),
      };
    }

    if (msg.mode) {
      this.mode.value = msg.mode;
    }

    if (msg.bots) {
      let mapBots: GameMapBot[] = [];

      for (const [botId, bot] of Object.entries(msg.bots)) {
        const tileIdx = bot.pos[1] * this.map.value.size[0] + bot.pos[0];

        mapBots[tileIdx] = {
          id: botId,
          known: playerBots.has(botId),
        };
      }

      this.bots.value = msg.bots;
      this.map.value.bots = mapBots;

      if (this.bot.value?.following) {
        const botEntry = msg.bots[this.bot.value.id];

        if (botEntry) {
          this.camera.value = {
            x: botEntry.pos[0] + 1,
            y: botEntry.pos[1] + 1,
          };
        }
      }
    }

    if (this.bot.value) {
      const old = this.bot.value;

      const events = (msg.bot?.events ?? []).map((event: any) => {
        return {
          at: new Date(event.at),
          msg: event.msg,
        };
      });

      this.bot.value = {
        ...msg.bot,
        ...{
          id: old.id,
          events: (old.events ?? []).concat(events),
          following: old.following,
        },
      };

      this.bot.value.events.sort((a, b) => {
        return b.at.getTime() - a.at.getTime();
      });

      this.bot.value.events = this.bot.value.events.slice(0, 64);
    }
  }

  private handleServerClose(retryJoin: () => void): void {
    if (this.status.value == "connected" || this.status.value == "connecting") {
      this.status.value = "reconnecting";

      setTimeout(retryJoin, 250);
    }
  }
}

export interface GameMap {
  size: [number, number];
  tiles: number[];
  bots: GameMapBot[];
}

export interface GameMapBot {
  id: string;
  known: boolean;
}

export type GameBot = {
  id: string;
  following: boolean;
} & (ServerConnectedBotUpdate | { status: "unknown"; events: BotEvent[] });

export interface GameTableBot {
  id: string;
  age: number;
  score: number;
  known: boolean;
  nth: number;
}

export type GameBots = ServerBotsUpdate;

export interface GameCamera {
  x: number;
  y: number;
}

export type GameConnectionStatus =
  | "connecting"
  | "reconnecting"
  | "connected"
  | "closing";

export type GameDialogId = "help" | "tutorial" | "summary" | "sandboxConfig";
