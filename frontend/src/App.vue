<script setup lang="ts">
import { ref, onMounted } from "vue";
import Crash from "./components/Crash.vue";
import Game from "./components/Game.vue";
import Home from "./components/Home.vue";
import Intro from "./components/Intro.vue";
import { type Server, RemoteServer, LocalServer } from "./logic/Server";

type Route =
  | { id: "home"; worldId?: string }
  | { id: "intro" }
  | { id: "game"; worldId: string; worldName: string; botId?: string }
  | { id: "crash"; msg: string };

const route = ref<Route>({ id: "home" });

let server: Server = undefined;

function handleStart(worldId: string, worldName: string, botId?: string) {
  if (worldId === "sandbox") {
    if (server instanceof LocalServer) {
      //
    } else {
      if (server) {
        server.close();
      }

      server = new LocalServer({
        name: "total mayhem",
        mode: {
          type: "deathmatch",
        },
        theme: {
          type: "arena",
          radius: 20,
        },
        policy: {
          max_alive_bots: 32,
          max_queued_bots: 64,
        },
      });
    }
  } else {
    if (server) {
      server.close();
    }

    server = new RemoteServer();
  }

  route.value = {
    id: "game",
    worldId,
    worldName,
    botId,
  };
}

function handleLeave() {
  route.value = { id: "home" };
}

function handleOpenIntro() {
  route.value = { id: "intro" };
  server = null;
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
    <Home
      :worldId="route.worldId"
      @start="handleStart"
      @open-intro="handleOpenIntro"
    />
  </template>

  <template v-if="route.id == 'game'">
    <Game
      :worldId="route.worldId"
      :worldName="route.worldName"
      :botId="route.botId"
      :server="server"
      @leave="handleLeave"
      @open-intro="handleOpenIntro"
    />
  </template>

  <template v-if="route.id == 'intro'">
    <Intro @leave="handleLeave" />
  </template>

  <template v-if="route.id == 'crash'">
    <Crash :msg="route.msg" />
  </template>
</template>
