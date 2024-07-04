<script setup>
  import { botIdToColor } from '@/utils/bot.ts';
  import { durationToHuman } from '@/utils/other.ts';

  const emit = defineEmits(['botClock', 'close']);
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
          <th>age</th>
          <th>score</th>
        </tr>
      </thead>

      <tbody>
        <tr v-for="entry in sortedBots">
          <td>
            #{{ entry.nth }}&nbsp;
          </td>

          <td>
            <a @click="emit('botClick', entry.id)"
               :style="`color: ${botIdToColor(entry.id)}`">
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
  .game-side-summary {
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
