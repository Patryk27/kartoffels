<script setup lang="ts">
import { computed } from "vue";
import type { GameTableBot, GameWorld } from "../World";
import BotLink from "../Common/BotLink.vue";

const emit = defineEmits<{
  botClick: [string];
  summaryOpen: [];
}>();

const props = defineProps<{
  world: GameWorld;
}>();

const filteredBots = computed<(GameTableBot & { ty: string })[]>(() => {
  const bot = props.world.bot.value;
  const bots = props.world.botsTable.value;

  let result = [];

  for (let nth = 0; nth < 5; nth += 1) {
    const entry = bots[nth];

    if (entry) {
      result.push({
        ty: "bot",
        ...entry,
      });
    } else {
      result.push(null);
    }
  }

  if (bot) {
    const connectedBotNth = bots.findIndex((entry) => {
      return entry.id == bot.id;
    });

    if (connectedBotNth >= result.length) {
      result[result.length - 2] = {
        ty: "sep",
      };

      result[result.length - 1] = {
        ty: "bot",
        ...bots[connectedBotNth],
      };
    }
  }

  return result;
});
</script>

<template>
  <div v-if="world.botsTable.value.length > 0" class="game-side-bots">
    <table>
      <thead>
        <tr>
          <th></th>
          <th>bot</th>
          <th>score</th>
        </tr>
      </thead>

      <tbody>
        <tr
          v-for="entry in filteredBots"
          :class="{ 'connected-bot': entry && entry.id == world.bot.value?.id }"
        >
          <template v-if="entry?.ty == 'bot'">
            <td>#{{ entry.nth }}&nbsp;</td>

            <td>
              <BotLink :bot="entry" @click="emit('botClick', entry.id)" />
            </td>

            <td>
              {{ entry.score }}
            </td>
          </template>

          <template v-else-if="entry?.ty == 'sep'">
            <td></td>
            <td style="text-align: center">...</td>
            <td></td>
          </template>

          <template v-else>
            <td>-</td>
            <td style="text-align: center">-</td>
            <td>-</td>
          </template>
        </tr>
      </tbody>
    </table>

    <div style="text-align: right; margin-top: 1em">
      <button @click="emit('summaryOpen')">show more</button>
    </div>
  </div>
</template>

<style scoped>
.game-side-bots {
  margin-top: 1em;
  padding-top: 1em;
  border-top: 1px solid var(--gray);

  table {
    width: 100%;

    tr {
      &.connected-bot {
        background-color: #202020;
      }

      td {
        &:nth-child(3) {
          text-align: right;
        }
      }
    }
  }
}
</style>
