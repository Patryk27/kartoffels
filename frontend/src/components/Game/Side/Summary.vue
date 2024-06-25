<script setup>
  import { botIdToColor } from '@/utils/bot.ts';

  const emit = defineEmits(['close']);
  const props = defineProps(['opened', 'sortedBots']);
</script>

<template>
  <dialog class="game-side-summary" :open="opened">
    <nav>
      <button @click="emit('close')">
        close
      </button>
    </nav>

    <table>
      <thead>
        <tr>
          <th>place</th>
          <th>bot</th>
          <th>status</th>
          <th>score</th>
        </tr>
      </thead>

      <tbody>
        <tr v-for="entry in sortedBots">
          <td>
            #{{ entry.nth }}&nbsp;
          </td>

          <td>
            <span :style="`color: ${botIdToColor(entry.id)}`">
              {{ entry.id }}
            </span>
          </td>

          <td>
            alive
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
  .game-side-summary {
    table {
      th {
        &:not(:last-child) {
          padding-right: 0.5em;
        }
      }

      td {
        &:not(:last-child) {
          padding-right: 0.5em;
        }

        &:nth-child(4) {
          text-align: right;
        }
      }
    }
  }
</style>
