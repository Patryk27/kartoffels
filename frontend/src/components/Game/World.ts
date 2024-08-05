import {
  type ServerBotEvent,
  type Server,
  type ServerConnBotsMsg,
  type ServerConnJoinedBotMsg,
  type ServerConnMsg,
  type ServerConnMapMsg,
  type ServerConnModeMsg,
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
  private abort?: AbortController;

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
    botId: string | null,
    onBotNotFound: () => void,
  ): Promise<void> {
    this.leave();

    if (botId) {
      this.bot.value = {
        id: botId,
        following: true,
        status: null,
        events: [],
      };
    } else {
      this.bot.value = null;
    }

    this.status.value =
      this.status.value == "reconnecting" ? "reconnecting" : "connecting";

    server.onReconnect((status) => {
      if (status == "reconnecting") {
        this.status.value = "reconnecting";
      } else {
        this.status.value = "connected";
      }
    });

    const abort = new AbortController();
    const msgs = await server.join(botId);

    new Promise(async () => {
      let isFirstMessage = true;

      for await (const msg of msgs) {
        if (abort.signal.aborted) {
          break;
        }

        if (isFirstMessage && this.bot.value && msg.bot?.status == "unknown") {
          onBotNotFound();
        }

        this.handleMsg(playerBots, msg);

        isFirstMessage = false;
      }

      msgs.cancel();
    });

    this.abort = abort;
    this.status.value = "connected";
  }

  leave(): void {
    if (this.abort) {
      this.abort.abort();
      this.abort = null;
    }
  }

  private handleMsg(playerBots: PlayerBots, msg: ServerConnMsg): void {
    if (msg.map) {
      this.handleMsgMap(msg.map);
    }

    if (msg.mode) {
      this.handleMsgMode(msg.mode);
    }

    if (msg.bots) {
      this.handleMsgBots(playerBots, msg.bots);
    }

    if (msg.bot) {
      this.handleMsgBot(msg.bot);
    }
  }

  private handleMsgMap(msgMap: ServerConnMapMsg): void {
    this.map.value = {
      size: msgMap.size,
      tiles: msgMap.tiles,
      bots: [],
    };

    this.camera.value = {
      x: Math.round(msgMap.size[0] / 2),
      y: Math.round(msgMap.size[1] / 2),
    };
  }

  private handleMsgMode(msgMode: ServerConnModeMsg): void {
    this.mode.value = msgMode;
  }

  private handleMsgBots(
    playerBots: PlayerBots,
    msgBots: ServerConnBotsMsg,
  ): void {
    let mapBots: GameMapBot[] = [];

    for (const [botId, bot] of Object.entries(msgBots)) {
      const tileIdx = bot.pos[1] * this.map.value.size[0] + bot.pos[0];

      mapBots[tileIdx] = {
        id: botId,
        known: playerBots.has(botId),
      };
    }

    this.bots.value = msgBots;
    this.map.value.bots = mapBots;

    if (this.bot.value?.following) {
      const botEntry = msgBots[this.bot.value.id];

      if (botEntry) {
        this.camera.value = {
          x: botEntry.pos[0] + 1,
          y: botEntry.pos[1] + 1,
        };
      }
    }
  }

  private handleMsgBot(msgBot: ServerConnJoinedBotMsg): void {
    const currBot = this.bot.value;

    if (!this.bot.value) {
      return;
    }

    if (msgBot.status == "unknown") {
      this.bot.value = null;
      return;
    }

    // ---

    let events = currBot.events.concat(
      msgBot.events.map((event: any) => {
        return {
          at: new Date(event.at),
          msg: event.msg,
        };
      }),
    );

    events.sort((a, b) => {
      return b.at.getTime() - a.at.getTime();
    });

    events = events.slice(0, 64);

    // ---

    let state: GameBotState;

    switch (msgBot.status) {
      case "alive":
        state = {
          status: "alive",
          age: msgBot.age,
          serial: msgBot.serial,
        };

        break;

      case "dead":
        state = {
          status: "dead",
        };

        break;

      case "queued":
        state = {
          status: "queued",
          place: msgBot.place,
          requeued: msgBot.requeued,
        };

        break;
    }

    // ---

    this.bot.value = {
      ...state,
      id: currBot.id,
      following: currBot.following,
      events,
    };
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
  events: ServerBotEvent[];
} & GameBotState;

export type GameBotState =
  | {
      status: "alive";
      age: number;
      serial: [number];
    }
  | {
      status: "dead";
    }
  | {
      status: "queued";
      place: number;
      requeued: boolean;
    };

export interface GameTableBot {
  id: string;
  age: number;
  score: number;
  known: boolean;
  nth: number;
}

export type GameBots = ServerConnBotsMsg;

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
