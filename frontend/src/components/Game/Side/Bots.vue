<script setup lang="ts">
import { computed } from "vue";
import { botIdToColor } from "@/utils/bot";
import type { GameBot, GameTableBot } from "@/components/Game.vue";

const emit = defineEmits<{
  botClick: [string];
  openSummary: [];
}>();

const props = defineProps<{
  bot?: GameBot;
  bots: GameTableBot[];
  mode: any;
}>();

const filteredBots = computed(() => {
  let result = [];

  for (let nth = 0; nth < 8; nth += 1) {
    const entry = props.bots[nth];

    if (entry) {
      result.push({
        ty: "bot",
        ...entry,
      });
    } else {
      result.push(null);
    }
  }

  if (props.bot) {
    const connectedBotNth = props.bots.findIndex((entry) => {
      return entry.id == props.bot.id;
    });

    if (connectedBotNth >= result.length) {
      result[result.length - 2] = {
        ty: "sep",
      };

      result[result.length - 1] = {
        ty: "bot",
        ...props.bots[connectedBotNth],
      };
    }
  }

  return result;
});
</script>

<template>
  <div v-if="bots.length > 0" class="game-side-bots">
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
          :class="entry && entry.id == bot?.id ? 'connected-bot' : ''"
        >
          <template v-if="entry?.ty == 'bot'">
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
      <button @click="emit('openSummary')">show more</button>
    </div>
  </div>
</template>

<style scoped>
.game-side-bots {
  border-top: 1px solid #444444;
  padding-top: 1em;

  table {
    width: 100%;

    tr {
      &.connected-bot {
        background-color: #202020;

        a {
          font-weight: 600;
        }
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
