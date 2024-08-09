<script setup lang="ts">
import { durationToHuman } from "@/utils/other";
import type { GameWorld } from "./World";
import BotLink from "./Common/BotLink.vue";

const emit = defineEmits<{
  botClick: [string];
  close: [];
}>();

defineProps<{
  world: GameWorld;
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

    <main>
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
          <tr
            v-for="entry in world.botsTable.value"
            :class="{ 'connected-bot': entry.id == world.bot.value?.id }"
          >
            <td>#{{ entry.nth }}</td>

            <td>
              <BotLink :bot="entry" @click="emit('botClick', entry.id)" />
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
    </main>
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

    tr {
      &.connected-bot {
        background-color: #202020;
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
}
</style>
