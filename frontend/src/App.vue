<script setup>
  import { ref, onMounted } from 'vue';
  import Game from './components/Game.vue';
  import Home from './components/Home.vue';
  import Tutorial from './components/Tutorial.vue';

  const route = ref({
    id: 'home',
  });

  function handleJoinOrRestore(worldId, botId) {
    route.value = { id: 'game', worldId, botId };
  }

  function handleLeave() {
    route.value = { id: 'home' };
  }

  function handleOpenTutorial() {
    route.value = { id: 'tutorial' };
  }

  onMounted(() => {
    window.onerror = (msg) => {
      route.value = { id: 'bsod', msg };
    };
  });
</script>

<template>
  <template v-if="route.id == 'home'">
    <Home
      @join="handleJoinOrRestore"
      @restore="handleJoinOrRestore"
      @open-tutorial="handleOpenTutorial" />
  </template>

  <template v-if="route.id == 'game'">
    <Game
      :worldId="route.worldId"
      :botId="route.botId"
      @leave="handleLeave"
      @open-tutorial="handleOpenTutorial" />
  </template>

  <template v-if="route.id == 'tutorial'">
    <Tutorial
      @leave="handleLeave" />
  </template>

  <template v-if="route.id == 'bsod'">
    <main style="padding: 1em">
      <p style="margin: 0">
        whoopsie, kartoffels have fell out of pot and ✨ crashed ✨
      </p>

      <p>
        {{ route.msg }}
      </p>

      <p style="margin-top: 0">
        please refresh the page to restart
      </p>
    </main>
  </template>
</template>
