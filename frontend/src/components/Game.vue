<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { storeSession } from "@/logic/Session";
import {
  LocalServer,
  type BotEvent,
  type Server,
  type ServerBotsUpdate,
  type ServerConnectedBotUpdate,
} from "@/logic/Server";
import Canvas from "./Game/Canvas.vue";
import Nav from "./Game/Nav.vue";
import SandboxConfig from "./Game/SandboxConfig.vue";
import Side from "./Game/Side.vue";
import Summary from "./Game/Summary.vue";

export interface GameMap {
  size: [number, number];
  tiles: number[];
  bots: { id: string }[];
}

export type GameBot = {
  id: string;
  following: boolean;
} & (ServerConnectedBotUpdate | { status: "unknown"; events: BotEvent[] });

export interface GameTableBot {
  id: string;
  age: number;
  score: number;
  nth: number;
}

export type GameBots = ServerBotsUpdate;

export interface GameCamera {
  x: number;
  y: number;
}

export type GameStatus =
  | "connecting"
  | "reconnecting"
  | "connected"
  | "closing";

export type GameDialogId = "sandbox-config" | "summary";

// ---

const emit = defineEmits<{
  leave: [];
  openHelp: [];
}>();

const props = defineProps<{
  worldId: string;
  worldName: string;
  botId?: string;
  server: Server;
}>();

const map = ref<GameMap>(null);
const mode = ref(null);
const bot = ref<GameBot>(null);
const bots = ref<GameBots>(null);
const camera = ref<GameCamera>(null);
const status = ref<GameStatus>("connecting");
const paused = ref(false);
const dialog = ref<GameDialogId>(null);

const server: Server = props.server;

const tableBots = computed(() => {
  let result: GameTableBot[] = [];

  for (const [id, bot] of Object.entries(bots.value ?? {})) {
    result.push({
      id,
      age: bot.age,
      score: (mode.value ?? {}).scores[id] ?? 0,
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

function join(newBotId?: string): void {
  server.onClose(null);
  server.leave();

  map.value = null;
  mode.value = null;
  bot.value = null;
  bots.value = null;
  camera.value = null;
  status.value = status.value == "reconnecting" ? "reconnecting" : "connecting";
  paused.value = false;

  if (newBotId) {
    bot.value = {
      id: newBotId,
      following: true,
      status: "unknown",
      events: [],
    };
  }

  server.join(props.worldId, newBotId);

  server.onOpen(() => {
    status.value = "connected";

    storeSession({
      worldId: props.worldId,
      botId: bot.value?.id,
    });
  });

  server.onClose(() => {
    if (status.value == "connected" || status.value == "connecting") {
      status.value = "reconnecting";

      setTimeout(() => {
        join(newBotId);
      }, 250);
    }
  });

  server.onError(() => {
    server.onError(null);
    server.onClose(null);

    if (status.value == "reconnecting") {
      setTimeout(() => {
        join(newBotId);
      }, 250);
    } else {
      if (newBotId) {
        alert(`couldn't find bot ${newBotId}`);

        // LocalServer needs an extra tick before we're able to join() again
        setTimeout(() => {
          join(null);
        }, 0);
      } else {
        window.onerror(`couldn't join world ${props.worldId}`);
      }
    }
  });

  server.onUpdate((msg) => {
    if (msg.map) {
      map.value = {
        size: msg.map.size,
        tiles: msg.map.tiles,
        bots: [],
      };

      camera.value = {
        x: Math.round(msg.map.size[0] / 2),
        y: Math.round(msg.map.size[1] / 2),
      };
    }

    if (msg.mode) {
      mode.value = msg.mode;
    }

    if (msg.bots) {
      let mapBots = [];

      for (const [botId, bot] of Object.entries(msg.bots)) {
        const tileIdx = bot.pos[1] * map.value.size[0] + bot.pos[0];

        mapBots[tileIdx] = {
          id: botId,
        };
      }

      bots.value = msg.bots;
      map.value.bots = mapBots;

      if (bot.value?.following) {
        const botEntry = msg.bots[bot.value.id];

        if (botEntry) {
          camera.value = {
            x: botEntry.pos[0],
            y: botEntry.pos[1],
          };
        }
      }
    }

    if (bot.value) {
      const old = bot.value;

      const events = (msg.bot?.events ?? []).map((event: any) => {
        return {
          at: new Date(event.at),
          msg: event.msg,
        };
      });

      bot.value = {
        ...msg.bot,
        ...{
          id: old.id,
          events: (old.events ?? []).concat(events),
          following: old.following,
        },
      };

      bot.value.events.sort((a, b) => {
        return b.at.getTime() - a.at.getTime();
      });

      bot.value.events = bot.value.events.slice(0, 64);
    }
  });
}

function handlePause(): void {
  paused.value = !paused.value;

  if (paused.value) {
    server.onClose(null);
    server.leave();
  } else {
    // TODO don't restart camera position
    join(bot.value?.id);
  }
}

async function handleBotUpload(src: File): Promise<void> {
  try {
    const bot = await server.uploadBot(src);

    join(bot.id);
  } catch (error) {
    alert("err, your bot couldn't be uploaded:\n\n" + error);
  }
}

async function handleBotSpawnPrefab(ty: string): Promise<void> {
  if (!(server instanceof LocalServer)) {
    return;
  }

  const instances = parseInt(
    prompt(`how many instances of ${ty} you'd like to spawn?`, "1"),
  );

  for (let i = 0; i < instances; i += 1) {
    try {
      const bot = await server.spawnPrefabBot(ty);

      if (instances == 1) {
        join(bot.id);
      }
    } catch (error) {
      alert("err, prefab couldn't be spawned:\n\n" + error);
      break;
    }
  }
}

function handleBotConnect(id?: string): void {
  join(id);
}

function handleBotDisconnect(): void {
  join(null);
}

function handleBotClick(id?: string): void {
  if (bot.value?.id == id && !paused.value) {
    join(null);
  } else {
    join(id);
  }
}

function handleBotDestroy(): void {
  if (!(server instanceof LocalServer)) {
    return;
  }

  if (bot.value?.id) {
    server.destroyBot(bot.value.id);
    bot.value = null;
  }
}

function handleBotRestart(): void {
  if (!(server instanceof LocalServer)) {
    return;
  }

  if (bot.value?.id) {
    server.restartBot(bot.value.id);
  }
}

function openDialog(id: GameDialogId): void {
  dialog.value = dialog.value == id ? undefined : id;
}

function handleRecreateSandbox(config: any): void {
  if (!(server instanceof LocalServer)) {
    return;
  }

  dialog.value = undefined;
  server.recreate(config);

  join(null);
}

// ---

onMounted(() => {
  document.onkeydown = (event) => {
    const moveCamera = (dx: number, dy: number): void => {
      if (camera.value) {
        camera.value.x += dx;
        camera.value.y += dy;

        if (bot.value) {
          bot.value.following = false;
        }
      }
    };

    switch (event.key) {
      case "ArrowLeft":
      case "a":
        moveCamera(-8, 0);
        break;

      case "ArrowUp":
      case "w":
        moveCamera(0, -8);
        break;

      case "ArrowRight":
      case "d":
        moveCamera(8, 0);
        break;

      case "ArrowDown":
      case "s":
        moveCamera(0, 8);
        break;

      case " ":
        handlePause();
        break;

      case "Escape":
        dialog.value = undefined;
        break;
    }
  };

  window.onbeforeunload = () => {
    status.value = "closing";
  };
});

join(props.botId);
</script>

<template>
  <div class="game">
    <Nav
      :worldId="worldId"
      :worldName="worldName"
      :status="status"
      :paused="paused"
      @leave="emit('leave')"
      @pause="handlePause"
      @open-help="emit('openHelp')"
      @open-sandbox-config="openDialog('sandbox-config')"
    />

    <main>
      <Canvas
        :map="map"
        :bot="bot"
        :bots="bots"
        :camera="camera"
        :status="status"
        :paused="paused"
      />

      <Side
        :worldId="worldId"
        :mode="mode"
        :bot="bot"
        :bots="tableBots"
        :status="status"
        :paused="paused"
        @bot-upload="handleBotUpload"
        @bot-spawn-prefab="handleBotSpawnPrefab"
        @bot-connect="handleBotConnect"
        @bot-disconnect="handleBotDisconnect"
        @bot-click="handleBotClick"
        @bot-destroy="handleBotDestroy"
        @bot-restart="handleBotRestart"
        @open-summary="openDialog('summary')"
      />

      <Summary
        :open="dialog == 'summary'"
        :bots="tableBots"
        @close="dialog = undefined"
        @bot-click="handleBotClick"
      />

      <SandboxConfig
        :open="dialog == 'sandbox-config'"
        @close="dialog = undefined"
        @recreate-sandbox="handleRecreateSandbox"
      />
    </main>
  </div>
</template>

<style scoped>
.game {
  display: flex;
  flex-direction: column;
  flex-grow: 1;
  align-self: stretch;

  main {
    display: flex;
    align-items: stretch;
    flex-grow: 1;
  }
}
</style>
