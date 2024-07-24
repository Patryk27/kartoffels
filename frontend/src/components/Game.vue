<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { PlayerBots, storeSession } from "@/logic/State";
import Canvas from "./Game/Canvas.vue";
import Nav from "./Game/Nav.vue";
import Help, * as help from "./Game/Help.vue";
import SandboxConfig from "./Game/SandboxConfig.vue";
import Side from "./Game/Side.vue";
import Summary from "./Game/Summary.vue";
import GameTutorial, * as tutorial from "./Game/Tutorial.vue";
import { LocalServer, type Server } from "@/logic/Server";
import type { GameDialogId } from "./Game/State";
import { GameCtrl } from "./Game/Ctrl";
import { GameWorld } from "./Game/State";
import { isValidBotId } from "@/utils/bot";

const emit = defineEmits<{
  leave: [];
}>();

const { worldId, worldName, botId, server } = defineProps<{
  worldId: string;
  worldName: string;
  botId?: string;
  server: Server;
}>();

const paused = ref(false);
const dialog = ref<GameDialogId>(null);

const ctrl = new GameCtrl(paused);
const playerBots = new PlayerBots(worldId);
const world = new GameWorld(worldId, worldName, playerBots);

// TODO this is cursed, but currently there's no better way to have this logic
//      available both when user pauses and when the GameCtrl wants to pause
watch(paused, (oldValue, newValue) => {
  if (oldValue == newValue) {
    return;
  }

  if (server instanceof LocalServer) {
    server.pause(paused.value);
  } else {
    // Since can't pause remote connections, just drop the connection and
    // transparently reacquire it on unpausing.
    //
    // TODO restore camera position

    if (paused.value) {
      server.leave();
    } else {
      join(world.bot.value?.id);
    }
  }
});

switch (worldId) {
  case "tutorial":
    tutorial.start(ctrl).then(() => {
      emit("leave");
    });

    break;

  case "sandbox":
    if (help.canOpenSandboxHelp()) {
      dialog.value = "help";
    }

    break;
}

async function join(newBotId?: string): Promise<void> {
  paused.value = false;

  try {
    if (newBotId && !isValidBotId(newBotId)) {
      alert(`\`${newBotId}\` is not a valid bot id`);
      newBotId = null;
    }

    await world.join(server, playerBots, newBotId);

    storeSession({
      worldId: worldId,
      botId: newBotId,
    });

    ctrl.emit("server.ready");
  } catch (err) {
    window.onerror(`couldn't join world ${worldId}`);
    console.log(err);
  }
}

function handlePause(): void {
  paused.value = !paused.value;
}

async function handleBotUpload(src: File): Promise<void> {
  try {
    const bot = await server.uploadBot(src);

    playerBots.add(bot.id);

    await join(bot.id);

    ctrl.emit("join");
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
  join(id);
}

function handleBotDestroy(): void {
  if (!(server instanceof LocalServer)) {
    return;
  }

  if (world.bot.value?.id) {
    server.destroyBot(world.bot.value.id);
    world.bot.value = null;
  }
}

function handleBotRestart(): void {
  if (!(server instanceof LocalServer)) {
    return;
  }

  if (world.bot.value?.id) {
    server.restartBot(world.bot.value.id);
  }
}

function toggleDialog(id: GameDialogId): void {
  dialog.value = dialog.value == id ? null : id;
}

function handleRecreateSandbox(config: any): void {
  if (!(server instanceof LocalServer)) {
    return;
  }

  dialog.value = null;
  server.recreate(config);

  join(null);
}

// ---

onMounted(() => {
  document.onkeydown = (event) => {
    const moveCamera = (dx: number, dy: number): void => {
      if (ctrl.tutorialSlide.value) {
        return;
      }

      if (world.camera.value) {
        world.camera.value.x += dx;
        world.camera.value.y += dy;

        if (world.bot.value) {
          world.bot.value.following = false;
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
        dialog.value = null;
        break;
    }
  };

  window.onbeforeunload = () => {
    world.status.value = "closing";
  };
});

join(botId);
</script>

<template>
  <div class="game">
    <Nav
      :ctrl="ctrl"
      :world="world"
      :paused="paused"
      @leave="emit('leave')"
      @pause="handlePause"
      @open-help="toggleDialog('help')"
      @open-config="toggleDialog('sandboxConfig')"
    />

    <main>
      <Canvas :world="world" :paused="paused" />

      <Side
        :ctrl="ctrl"
        :world="world"
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

      <Help :open="dialog == 'help'" :world="world" @close="dialog = null" />

      <GameTutorial :ctrl="ctrl" />

      <Summary
        :open="dialog == 'summary'"
        :world="world"
        @close="dialog = null"
        @bot-click="handleBotClick"
      />

      <SandboxConfig
        :open="dialog == 'sandboxConfig'"
        @close="dialog = null"
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
