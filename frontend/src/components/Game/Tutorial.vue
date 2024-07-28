<script setup lang="ts">
import { ref } from "vue";
import type { GameCtrl } from "./Ctrl";
import Slide01 from "./Tutorial/Slide01.vue";
import Slide02 from "./Tutorial/Slide02.vue";
import Slide03 from "./Tutorial/Slide03.vue";
import Slide04 from "./Tutorial/Slide04.vue";
import Slide05 from "./Tutorial/Slide05.vue";
import Slide06 from "./Tutorial/Slide06.vue";
import Slide07 from "./Tutorial/Slide07.vue";

defineProps<{
  ctrl: GameCtrl;
}>();
</script>

<script lang="ts">
// Starts the tutorial and returns a `Promise` that resolves once the entire
// tutorial is completed.
export function start(ctrl: GameCtrl): Promise<void> {
  return new Promise((resolve, _) => {
    ctrl.disableButton("help");
    ctrl.disableButton("pause");
    ctrl.disableButton("connectToBot");
    ctrl.disableButton("uploadBot");

    ctrl.on("server.ready", () => {
      ctrl.openSlide(1);
    });

    ctrl.on("tutorial.before-slide", () => {
      const dialog = document.querySelector(".game-tutorial");

      if (dialog) {
        dialog.scrollTop = 0;
      }
    });

    ctrl.on("tutorial.done", resolve);
  });
}
</script>

<template>
  <dialog class="game-tutorial" :open="ctrl.tutorialSlide.value != null">
    <nav>
      <div class="dialog-title">tutorial</div>
    </nav>

    <Slide01 :ctrl v-if="ctrl.tutorialSlide.value == 1" />
    <Slide02 :ctrl v-else-if="ctrl.tutorialSlide.value == 2" />
    <Slide03 :ctrl v-else-if="ctrl.tutorialSlide.value == 3" />
    <Slide04 :ctrl v-else-if="ctrl.tutorialSlide.value == 4" />
    <Slide05 :ctrl v-else-if="ctrl.tutorialSlide.value == 5" />
    <Slide06 :ctrl v-else-if="ctrl.tutorialSlide.value == 6" />
    <Slide07 :ctrl v-else-if="ctrl.tutorialSlide.value == 7" />
  </dialog>
</template>

<style>
.game-tutorial {
  width: 768px;

  p,
  ul {
    &:last-child {
      margin-bottom: 0;
    }
  }

  pre {
    margin-left: 4ch;
  }

  .inverted {
    background: white;
    color: black;
  }
}
</style>
