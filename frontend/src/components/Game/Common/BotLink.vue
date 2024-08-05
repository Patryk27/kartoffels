<script setup lang="ts">
import { botIdToColor } from "@/utils/bot";
import type { GameTableBot } from "../World";

defineProps<{
  bot: GameTableBot;
}>();
</script>

<template>
  <template v-if="bot.known">
    <a
      style="color: #ffffff"
      :style="`background: ${botIdToColor(bot.id, 'bg')}`"
    >
      {{ bot.id }}
    </a>
  </template>

  <template v-else>
    <a :style="`color: ${botIdToColor(bot.id)}`">
      {{ bot.id }}
    </a>
  </template>
</template>
