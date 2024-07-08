<script setup lang="ts">
import { computed, ref } from "vue";
import Bot from "./Side/Bot.vue";
import Bots from "./Side/Bots.vue";
import Summary from "./Side/Summary.vue";
import type { GameBot, GameBots, GameStatus } from "../Game.vue";

export interface GameSideBot {
  id: string;
  age: number;
  score: number;
  nth: number;
}

const emit = defineEmits<{
  botUpload: [File];
  botConnect: [string];
  botDisconnect: [];
  botClick: [string];
  botDestroy: [];
  botRestart: [];
}>();

const props = defineProps<{
  worldId: string;
  mode: any;
  bot?: GameBot;
  bots?: GameBots;
  status: GameStatus;
  paused: boolean;
}>();

const isSummaryOpen = ref(false);

const sideBots = computed(() => {
  let result: GameSideBot[] = [];

  for (const [id, bot] of Object.entries(props.bots ?? {})) {
    result.push({
      id,
      age: bot.age,
      score: (props.mode ?? {}).scores[id] ?? 0,
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

function handleOpenSummary() {
  isSummaryOpen.value = !isSummaryOpen.value;
}

function handleCloseSummary() {
  isSummaryOpen.value = false;
}
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
      :bots="sideBots"
      :mode="mode"
      @bot-click="(id) => emit('botClick', id)"
      @show-more="handleOpenSummary()"
    />

    <Summary
      :open="isSummaryOpen"
      :bots="sideBots"
      @bot-click="
        (id) => {
          emit('botClick', id);
          handleCloseSummary();
        }
      "
      @close="handleCloseSummary()"
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
