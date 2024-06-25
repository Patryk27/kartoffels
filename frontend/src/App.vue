<script setup>
  import { ref, onMounted } from 'vue';
  import Crash from './components/Crash.vue';
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
    <Crash
      :msg="route.msg" />
  </template>
</template>
