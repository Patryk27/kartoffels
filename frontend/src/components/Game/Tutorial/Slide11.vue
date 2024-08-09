<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();
</script>

<template>
  <p>alright, cool - the robot seems to be able to survive!</p>

  <p>
    now it's the time to talk about trade-offs, e.g. what's the point of having
    different scanning areas for the radar (3x3, 5x5 etc.), couldn't you always
    just pick the largest one?
  </p>

  <p>here's when the <b>cooldown</b> effect comes into play</p>

  <footer style="text-align: right">
    <button class="highlighted" @click="ctrl.openTutorialSlide(12)">
      let's read about the cooldown effect
    </button>
  </footer>
</template>
