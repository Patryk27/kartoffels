<script setup>
  import { ref, onMounted } from 'vue';
  import Game from './components/Game.vue';
  import Home from './components/Home.vue';
  import Intro from './components/Intro.vue';

  const route = ref({
    id: 'home',
  });

  function handleJoinOrRestore(worldId, botId) {
    route.value = { id: 'game', worldId, botId };
  }

  function handleLeave() {
    route.value = { id: 'home' };
  }

  function handleOpenIntro() {
    route.value = { id: 'intro' };
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
      @open-intro="handleOpenIntro" />
  </template>

  <template v-if="route.id == 'game'">
    <Game
      :worldId="route.worldId"
      :botId="route.botId"
      @leave="handleLeave"
      @open-intro="handleOpenIntro" />
  </template>

  <template v-if="route.id == 'intro'">
    <Intro
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
        please refresh the page and try again
      </p>
    </main>
  </template>
</template>
