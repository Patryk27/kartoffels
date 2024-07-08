<script setup lang="ts">
import Bot from "./Side/Bot.vue";
import Bots from "./Side/Bots.vue";
import type { GameBot, GameStatus, GameTableBot } from "../Game.vue";

const emit = defineEmits<{
  botUpload: [File];
  botConnect: [string];
  botDisconnect: [];
  botClick: [string];
  botDestroy: [];
  botRestart: [];
  openSummary: [];
}>();

defineProps<{
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
      :worldId="worldId"
      :bot="bot"
      :paused="paused"
      @bot-upload="(file) => emit('botUpload', file)"
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
