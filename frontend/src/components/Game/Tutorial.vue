<script setup lang="ts">
import type { GameCtrl } from "./Ctrl";
import Slide01 from "./Tutorial/Slide01.vue";
import Slide02 from "./Tutorial/Slide02.vue";
import Slide03 from "./Tutorial/Slide03.vue";
import Slide04 from "./Tutorial/Slide04.vue";
import Slide05 from "./Tutorial/Slide05.vue";
import Slide06 from "./Tutorial/Slide06.vue";
import Slide07 from "./Tutorial/Slide07.vue";
import Slide08 from "./Tutorial/Slide08.vue";
import Slide09 from "./Tutorial/Slide09.vue";
import Slide10 from "./Tutorial/Slide10.vue";
import Slide11 from "./Tutorial/Slide11.vue";
import Slide12 from "./Tutorial/Slide12.vue";
import Slide13 from "./Tutorial/Slide13.vue";

defineProps<{
  ctrl: GameCtrl;
}>();
</script>

<script lang="ts">
// Starts the tutorial and returns a promise that resolves once the entire
// tutorial is completed.
export function start(ctrl: GameCtrl): Promise<void> {
  return new Promise((resolve, _) => {
    ctrl.getLocalServer().setSpawnPoint(16, 16);

    ctrl.alterUi((ui) => {
      ui.enableConnectToBot = false;
      ui.enableHelp = false;
      ui.enablePause = false;
      ui.enableUploadBot = false;
    });

    ctrl.onOnce("server.ready", () => {
      ctrl.openTutorialSlide(1);
    });

    ctrl.on("tutorial.before-slide", () => {
      const dialog = document.querySelector(".game-tutorial");

      if (dialog) {
        dialog.scrollTop = 0;
      }
    });

    ctrl.onOnce("tutorial.done", resolve);
  });
}
</script>

<template>
  <dialog class="game-tutorial" :open="ctrl.tutorialSlide.value != null">
    <nav>
      <div class="dialog-title">
        tutorial ({{ ctrl.tutorialSlide.value }} / 13)
      </div>
    </nav>

    <Slide01 :ctrl v-if="ctrl.tutorialSlide.value == 1" />
    <Slide02 :ctrl v-else-if="ctrl.tutorialSlide.value == 2" />
    <Slide03 :ctrl v-else-if="ctrl.tutorialSlide.value == 3" />
    <Slide04 :ctrl v-else-if="ctrl.tutorialSlide.value == 4" />
    <Slide05 :ctrl v-else-if="ctrl.tutorialSlide.value == 5" />
    <Slide06 :ctrl v-else-if="ctrl.tutorialSlide.value == 6" />
    <Slide07 :ctrl v-else-if="ctrl.tutorialSlide.value == 7" />
    <Slide08 :ctrl v-else-if="ctrl.tutorialSlide.value == 8" />
    <Slide09 :ctrl v-else-if="ctrl.tutorialSlide.value == 9" />
    <Slide10 :ctrl v-else-if="ctrl.tutorialSlide.value == 10" />
    <Slide11 :ctrl v-else-if="ctrl.tutorialSlide.value == 11" />
    <Slide12 :ctrl v-else-if="ctrl.tutorialSlide.value == 12" />
    <Slide13 :ctrl v-else-if="ctrl.tutorialSlide.value == 13" />
  </dialog>
</template>

<style>
.game-tutorial {
  width: 768px;

  pre {
    margin-left: 4ch;
  }
}
</style>
