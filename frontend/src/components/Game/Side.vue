<script setup lang="ts">
import type { GameCtrl } from "./Ctrl";
import Bot from "./Side/Bot.vue";
import Bots from "./Side/Bots.vue";
import type { GameWorld } from "./World";

const emit = defineEmits<{
  botCreate: [File];
  botCreatePrefab: [string];
  botJoin: [string];
  botLeave: [];
  botDestroy: [];
  botRestart: [];
  summaryOpen: [];
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
      @bot-create="(file) => emit('botCreate', file)"
      @bot-create-prefab="(id) => emit('botCreatePrefab', id)"
      @bot-join="(id) => emit('botJoin', id)"
      @bot-leave="emit('botLeave')"
      @bot-destroy="emit('botDestroy')"
      @bot-restart="emit('botRestart')"
    />

    <Bots
      v-if="ctrl.ui.value.showBotList"
      :world="world"
      @bot-click="(id) => emit('botJoin', id)"
      @summary-open="emit('summaryOpen')"
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
