<script setup lang="ts">
import Bot from "./Side/Bot.vue";
import Bots from "./Side/Bots.vue";
import type {
  GameBot,
  GameController,
  GameStatus,
  GameTableBot,
} from "../Game.vue";

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
  ctrl: GameController;
  worldId: string;
  mode: any;
  bot?: GameBot;
  bots?: GameTableBot[];
  status: GameStatus;
  paused: boolean;
}>();
</script>

<template>
  <div v-if="status == 'connected' || status == 'connecting'" class="game-side">
    <Bot
      :ctrl="ctrl"
      :worldId="worldId"
      :bot="bot"
      :paused="paused"
      @bot-upload="(file) => emit('botUpload', file)"
      @bot-spawn-prefab="(id) => emit('botSpawnPrefab', id)"
      @bot-connect="(id) => emit('botConnect', id)"
      @bot-disconnect="emit('botDisconnect')"
      @bot-destroy="emit('botDestroy')"
      @bot-restart="emit('botRestart')"
    />

    <Bots
      :bot="bot"
      :bots="bots"
      :mode="mode"
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
