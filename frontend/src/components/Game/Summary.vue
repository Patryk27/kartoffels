<script setup lang="ts">
import { botIdToColor } from "@/utils/bot";
import { durationToHuman } from "@/utils/other";
import type { GameTableBot } from "../Game.vue";

const emit = defineEmits<{
  botClick: [string];
  close: [];
}>();

defineProps<{
  bots: GameTableBot[];
}>();
</script>

<template>
  <dialog class="game-summary">
    <nav>
      <div class="dialog-title">summary</div>

      <div class="dialog-buttons">
        <button @click="emit('close')">close</button>
      </div>
    </nav>

    <table>
      <thead>
        <tr>
          <th>place</th>
          <th>bot</th>
          <th>age</th>
          <th>score</th>
        </tr>
      </thead>

      <tbody>
        <tr v-for="entry in bots">
          <td>#{{ entry.nth }}&nbsp;</td>

          <td>
            <a
              @click="emit('botClick', entry.id)"
              :style="`color: ${botIdToColor(entry.id)}`"
            >
              {{ entry.id }}
            </a>
          </td>

          <td>
            {{ durationToHuman(Math.round(entry.age)) }}
          </td>

          <td>
            {{ entry.score }}
          </td>
        </tr>
      </tbody>
    </table>
  </dialog>
</template>

<style scoped>
.game-summary {
  table {
    th {
      &:not(:last-child) {
        padding-right: 1em;
      }
    }

    td {
      &:not(:last-child) {
        padding-right: 1em;
      }

      &:nth-child(3) {
        text-align: right;
      }

      &:nth-child(4) {
        text-align: right;
      }
    }
  }
}
</style>
