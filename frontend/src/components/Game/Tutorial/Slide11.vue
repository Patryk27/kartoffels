<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();
</script>

<template>
  <main>
    <p>alright, cool - the robot is able to survive!</p>

    <p>
      you seem to be a quick-learner, so let's move on to something more
      difficult, the trade-offs
    </p>

    <p>
      say, what's the point of having different scanning areas for the radar
      (3x3, 5x5 etc.) - wouldn't it be the best to always pick the largest scan?
    </p>

    <p>here's when the <b>cooldown</b> effect comes into play</p>
  </main>

  <footer style="text-align: right">
    <button class="highlighted" @click="ctrl.openTutorialSlide(12)">
      let's read about the cooldown effect
    </button>
  </footer>
</template>
