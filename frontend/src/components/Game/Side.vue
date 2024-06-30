<script setup>
  import { computed, ref } from 'vue';

  import Bot from './Side/Bot.vue';
  import Bots from './Side/Bots.vue';
  import Summary from './Side/Summary.vue';

  const emit = defineEmits([
    'botUpload',
    'botConnect',
    'botDisconnect',
    'botClick',
  ]);

  const props = defineProps(['mode', 'bot', 'bots', 'status', 'paused']);
  const isSummaryOpened = ref(false);

  const sortedBots = computed(() => {
    let result = [];

    for (const [id, bot] of Object.entries(props.bots ?? { })) {
      result.push({
        id,
        age: bot.age,
        score: (props.mode ?? { }).scores[id] ?? 0,
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
    isSummaryOpened.value = !isSummaryOpened.value;
  }

  function handleCloseSummary() {
    isSummaryOpened.value = false;
  }
</script>

<template>
  <div v-if="status == 'connected' || status == 'connecting'"
       class="game-side">
    <Bot :bot="bot"
         :paused="paused"
         @bot-upload="file => emit('botUpload', file)"
         @bot-connect="id => emit('botConnect', id)"
         @bot-disconnect="emit('botDisconnect')" />

    <Bots :bot="bot"
          :sortedBots="sortedBots"
          :mode="mode"
          @bot-click="id => emit('botClick', id)"
          @show-more="handleOpenSummary()" />

    <Summary :opened="isSummaryOpened"
             :sortedBots="sortedBots"
             @bot-click="id => { emit('botClick', id); handleCloseSummary(); }"
             @close="handleCloseSummary()" />
  </div>
</template>

<style scoped>
  .game-side {
    display: flex;
    flex-direction: column;
    margin-left: 1em;
  }
</style>
