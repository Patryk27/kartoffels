<script setup lang="ts">
import type { GameCtrl } from "./Ctrl";
import Bot from "./Side/Bot.vue";
import Bots from "./Side/Bots.vue";
import type { GameWorld } from "./State";

const emit = defineEmits<{
  botUpload: [File];
  botSpawnPrefab: [string];
  botConnect: [string];
  botDisconnect: [];
  botClick: [string];
  botDestroy: [];
  botRestart: [];
  openSummary: [];
}>();

defineProps<{
  ctrl: GameCtrl;
  world: GameWorld;
  paused: boolean;
}>();
</script>

<template>
  <div
    v-if="
      world.status.value == 'connected' || world.status.value == 'connecting'
    "
    class="game-side"
  >
    <Bot
      :ctrl="ctrl"
      :world="world"
      :paused="paused"
      @bot-upload="(file) => emit('botUpload', file)"
      @bot-spawn-prefab="(id) => emit('botSpawnPrefab', id)"
      @bot-connect="(id) => emit('botConnect', id)"
      @bot-disconnect="emit('botDisconnect')"
      @bot-destroy="emit('botDestroy')"
      @bot-restart="emit('botRestart')"
    />

    <Bots
      :world="world"
      @bot-click="(id) => emit('botClick', id)"
      @open-summary="emit('openSummary')"
    />
  </div>
</template>

<style scoped>
.game-side {
  display: flex;
  flex-direction: column;
  margin-left: 1em;
}
</style>
