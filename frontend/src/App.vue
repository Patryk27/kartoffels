<script setup lang="ts">
import { ref, onMounted } from "vue";
import Crash from "./components/Crash.vue";
import Game from "./components/Game.vue";
import Home from "./components/Home.vue";
import { type Server, RemoteServer, LocalServer } from "./logic/Server";
import * as SandboxPresets from "./components/Game/SandboxConfig/Presets";

type Route =
  | { id: "home" }
  | {
      id: "game";
      worldId: string;
      worldName: string;
      botId?: string;
      server: Server;
    }
  | { id: "crash"; msg: string };

const route = ref<Route>({ id: "home" });

function handleStart(worldId: string, worldName: string, botId?: string) {
  if (route.value.id == "game") {
    route.value.server.close();
    route.value.server = null;
  }

  let server: Server;

  switch (worldId) {
    case "tutorial":
      server = new LocalServer(SandboxPresets.getTutorialWorld());
      break;

    case "sandbox":
      server = new LocalServer(SandboxPresets.getDefaultWorld());
      break;

    default:
      server = new RemoteServer(worldId);
      break;
  }

  route.value = {
    id: "game",
    worldId,
    worldName,
    botId,
    server,
  };
}

function handleLeave(): void {
  if (route.value.id == "game") {
    route.value.server.close();
    route.value.server = null;
  }

  route.value = { id: "home" };
}

onMounted(() => {
  window.onerror = (msg) => {
    route.value = { id: "crash", msg: msg.toString() };
  };
});
</script>

<template>
  <template v-if="route.id == 'home'">
    <Home @start="handleStart" />
  </template>

  <template v-if="route.id == 'game'">
    <Game
      :worldId="route.worldId"
      :worldName="route.worldName"
      :botId="route.botId"
      :server="route.server"
      @leave="handleLeave()"
    />
  </template>

  <template v-if="route.id == 'crash'">
    <Crash :msg="route.msg" />
  </template>
</template>
