<script setup lang="ts">
import { ref, onMounted } from "vue";
import Crash from "./components/Crash.vue";
import Game from "./components/Game.vue";
import Home from "./components/Home.vue";
import { type Server, RemoteServer, LocalServer } from "./logic/Server";
import * as SandboxPresets from "./components/Game/SandboxConfig/Presets";

type Route =
  | { id: "home" }
  | { id: "game"; worldId: string; worldName: string; botId?: string }
  | { id: "crash"; msg: string };

const route = ref<Route>({ id: "home" });

let server: Server = null;

function handleStart(worldId: string, worldName: string, botId?: string) {
  switch (worldId) {
    case "tutorial":
      if (server) {
        server.close();
      }

      server = new LocalServer(SandboxPresets.getTutorialWorld());
      break;

    case "sandbox":
      if (server instanceof LocalServer) {
        //
      } else {
        if (server) {
          server.close();
        }

        server = new LocalServer(SandboxPresets.getDefaultWorld());
      }

      break;

    default:
      if (server) {
        server.close();
      }

      server = new RemoteServer(worldId);
      break;
  }

  route.value = {
    id: "game",
    worldId,
    worldName,
    botId,
  };
}

function handleLeave(): void {
  route.value = { id: "home" };
}

onMounted(() => {
  window.onerror = (msg) => {
    if (typeof msg === "string") {
      route.value = { id: "crash", msg };
    } else {
      // TODO
    }
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
      :server="server"
      @leave="handleLeave()"
    />
  </template>

  <template v-if="route.id == 'crash'">
    <Crash :msg="route.msg" />
  </template>
</template>
