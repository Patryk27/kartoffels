<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { PlayerBots, storeSession } from "@/logic/State";
import Canvas from "./Game/Canvas.vue";
import Nav from "./Game/Nav.vue";
import Help from "./Game/Help.vue";
import SandboxConfig from "./Game/SandboxConfig.vue";
import Side from "./Game/Side.vue";
import Summary from "./Game/Summary.vue";
import GameTutorial, * as tutorial from "./Game/Tutorial.vue";
import { LocalServer, type Server } from "@/logic/Server";
import type { GameDialogId, GameTableBot } from "./Game/State";
import { GameController } from "./Game/Controller";
import { GameWorld } from "./Game/State";

const emit = defineEmits<{
  leave: [];
}>();

const props = defineProps<{
  worldId: string;
  worldName: string;
  botId?: string;
  server: Server;
}>();

const ctrl = new GameController();
const world = new GameWorld();
const server = props.server;
const playerBots = new PlayerBots(props.worldId);

const paused = ref(false);
const dialog = ref<GameDialogId>(null);

if (props.worldId == "tutorial") {
  tutorial.setup(ctrl);
}

const tableBots = computed(() => {
  let result: GameTableBot[] = [];

  for (const [id, bot] of Object.entries(world.bots.value ?? {})) {
    result.push({
      id,
      age: bot.age,
      score: (world.mode.value ?? {}).scores[id] ?? 0,
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

async function join(newBotId?: string): Promise<void> {
  paused.value = false;

  try {
    await world.join(server, playerBots, newBotId);

    storeSession({
      worldId: props.worldId,
      botId: newBotId,
    });

    ctrl.emit("server.ready");
  } catch (err) {
    window.onerror(`couldn't join world ${props.worldId}`);
  }
}

function handlePause(): void {
  paused.value = !paused.value;

  if (server instanceof LocalServer) {
    server.pause(paused.value);
  } else {
    // We can't pause remote connections, so in that case let's just drop the
    // connection and transparently re-acquire it on unpausing.
    //
    // TODO restore camera position

    if (paused.value) {
      server.onClose(null);
      server.leave();
    } else {
      join(bot.value?.id);
    }
  }
}

async function handleBotUpload(src: File): Promise<void> {
  try {
    const bot = await server.uploadBot(src);

    playerBots.add(bot.id);

    join(bot.id);
  } catch (error) {
    alert("err, your bot couldn't be uploaded:\n\n" + error);
  }
}

async function handleBotSpawnPrefab(ty: string): Promise<void> {
  if (!(server instanceof LocalServer)) {
    return;
  }

  const instancesStr = prompt(
    `how many instances of ${ty} you'd like to spawn?`,
    "1",
  );

  if (!instancesStr) {
    return;
  }

  const instances = parseInt(instancesStr.trim());

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

function toggleDialog(id: GameDialogId): void {
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
      :ctrl="ctrl"
      :world="{ id: worldId, name: worldName }"
      :status="status"
      :paused="paused"
      @leave="emit('leave')"
      @pause="handlePause"
      @open-help="toggleDialog('help')"
      @open-config="toggleDialog('sandboxConfig')"
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
        :ctrl="ctrl"
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
        @open-summary="toggleDialog('summary')"
      />

      <Help
        :worldId="worldId"
        :open="dialog == 'help'"
        @close="dialog = undefined"
      />

      <GameTutorial :ctrl="ctrl" />

      <Summary
        :open="dialog == 'summary'"
        :bots="tableBots"
        @close="dialog = undefined"
        @bot-click="handleBotClick"
      />

      <SandboxConfig
        :open="dialog == 'sandboxConfig'"
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
