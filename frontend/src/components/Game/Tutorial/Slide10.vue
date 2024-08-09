<script setup lang="ts">
import { onUnmounted, ref } from "vue";
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

const abort = new AbortController();
const status = ref("awaiting");
const timer = ref(15);

new Promise(async () => {
  while (true) {
    const outcome = await Promise.race([
      ctrl.onceAnyBotIsKilled().then((_) => "bot-killed"),
      ctrl.onceTimerIsCompleted(timer).then((_) => "timer-completed"),
    ]);

    if (abort.signal.aborted) {
      return;
    }

    ctrl.getLocalServer().destroyAllBots();

    switch (outcome) {
      case "bot-killed":
        status.value = "dead";

        ctrl.alterUi((ui) => {
          ui.enableUploadBot = true;
          ui.highlightUploadBot = true;
        });

        await ctrl.waitFor("server.bot-create");

        if (abort.signal.aborted) {
          return;
        }

        status.value = "awaiting";
        timer.value = 15;

        ctrl.alterUi((ui) => {
          ui.enableUploadBot = false;
          ui.highlightUploadBot = false;
        });

        break;

      case "timer-completed":
        ctrl.openTutorialSlide(11);
        break;
    }
  }
});

onUnmounted(() => {
  abort.abort();
});
</script>

<template>
  <template v-if="status == 'awaiting'">
    <p>observing robot ({{ timer }}s)...</p>
  </template>

  <template v-else>
    <p>ouch, it looks like the robot got killed!</p>
    <p>make sure you uploaded the correct firmware and try again</p>

    <footer>
      <p>
        (need help?
        <a href="#" @click="ctrl.openTutorialSlide(9)">
          see the previous slide</a
        >)
      </p>
    </footer>
  </template>
</template>
