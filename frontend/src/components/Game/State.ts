import {
  type BotEvent,
  type Server,
  type ServerBotsUpdate,
  type ServerConnectedBotUpdate,
  type ServerMessage,
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

  async join(
    server: Server,
    playerBots: PlayerBots,
    botId?: string,
  ): Promise<void> {
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

    server.onMessage((msg) => {
      // This is a bit sideways, but currently the backend isn't able to
      // directly tell us whether our bot exists or not - we can only infer this
      // by looking at the first message we get: if it contains the bot, the bot
      // exists; otherwise it doesn't.
      if (this.bot.value?.status == "unknown" && !msg.bot) {
        alert(`couldn't find bot \`${botId}\``);
        this.bot.value = null;
      }

      this.handleMessage(playerBots, msg);
    });

    server.onStatusChange((status) => {
      if (status == "reconnecting") {
        this.status.value = "reconnecting";
      } else {
        this.status.value = "connected";
      }
    });

    await server.join(botId);

    this.status.value = "connected";
  }

  private handleMessage(playerBots: PlayerBots, msg: ServerMessage): void {
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
