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
import type { GameDialogId } from "./Game/World";
import { GameCtrl } from "./Game/Ctrl";
import { GameWorld } from "./Game/World";
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

const ctrl = new GameCtrl(server, paused);
const playerBots = new PlayerBots(worldId);
const world = new GameWorld(worldId, worldName, playerBots);

// HACK this is cursed, but currently there's no better way to have this logic
//      available both when the user pauses and when the GameCtrl wants to pause
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
      world.leave();
    } else {
      join(world.bot.value?.id);
    }
  }

  if (paused.value) {
    ctrl.emit("server.pause");
  } else {
    ctrl.emit("server.resume");
  }
});

async function join(newBotId?: string): Promise<void> {
  paused.value = false;

  try {
    if (newBotId && !isValidBotId(newBotId)) {
      alert(`\`${newBotId}\` is not a valid bot id`);
      newBotId = null;
    }

    await world.join(server, playerBots, newBotId, () => {
      alert(`couldn't find bot \`${newBotId}\``);
    });

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

async function handleBotUpload(src: File): Promise<void> {
  try {
    const bot = await server.uploadBot(src);

    playerBots.add(bot.id);

    await join(bot.id);

    ctrl.emit("server.bot-upload");
  } catch (error) {
    alert("err, your bot couldn't be uploaded:\n\n" + error);
  }
}

async function handleBotSpawnPrefab(ty: string): Promise<void> {
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
      const bot = await ctrl.getLocalServer().spawnPrefabBot(ty);

      // If the user isn't currently connected to any robot, join the first
      // spawned prefab, for convenience
      if (!world.bot.value && i == 0) {
        join(bot.id);
      }
    } catch (error) {
      alert("err, prefab couldn't be spawned:\n\n" + error);
      break;
    }
  }
}

function handleBotDestroy(): void {
  if (world.bot.value?.id) {
    ctrl.getLocalServer().destroyBot(world.bot.value.id);
    world.bot.value = null;
  }
}

function handleBotRestart(): void {
  if (world.bot.value?.id) {
    ctrl.getLocalServer().restartBot(world.bot.value.id);
  }
}

function toggleDialog(id: GameDialogId): void {
  dialog.value = dialog.value == id ? null : id;
}

function handleSandboxRecreate(config: any): void {
  dialog.value = null;
  ctrl.getLocalServer().recreate(config);

  join(null);
}

onMounted(() => {
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

  // TODO extract camera to a separate component
  document.onkeydown = (event) => {
    if (ctrl.tutorialSlide.value) {
      return;
    }

    const moveCamera = (dx: number, dy: number): void => {
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
        paused.value = !paused.value;
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
      @pause="paused = !paused"
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
        @bot-join="join"
        @bot-leave="join(null)"
        @bot-destroy="handleBotDestroy"
        @bot-restart="handleBotRestart"
        @summary-open="toggleDialog('summary')"
      />

      <Help :open="dialog == 'help'" :world="world" @close="dialog = null" />

      <GameTutorial :ctrl="ctrl" />

      <Summary
        :open="dialog == 'summary'"
        :world="world"
        @close="dialog = null"
        @bot-click="join"
      />

      <SandboxConfig
        :open="dialog == 'sandboxConfig'"
        @close="dialog = null"
        @recreate="handleSandboxRecreate"
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
