<script setup lang="ts">
import { ref } from "vue";
import type { GameCtrl } from "../Ctrl";
import { onUnmounted } from "vue";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

const abort = new AbortController();
const status = ref("awaiting-resume");
const timer = ref(null);

ctrl.alterUi((ui) => {
  ui.enableDisconnectFromBot = false;
  ui.enablePause = true;
  ui.enableUploadBot = false;
  ui.highlightPause = true;
  ui.highlightUploadBot = false;
  ui.showBotList = false;
});

ctrl.onOnce("server.resume", async () => {
  ctrl.alterUi((ui) => {
    ui.enablePause = false;
    ui.highlightPause = false;
  });

  status.value = "awaiting-kill";
  timer.value = 10;

  ctrl.on("server.bot-create", () => {
    status.value = "awaiting-kill";
    timer.value = 10;
  });

  while (true) {
    const outcome = await Promise.race([
      ctrl.onceAnyBotIsKilled().then((_) => "bot-killed"),
      ctrl.onceTimerIsCompleted(timer).then((_) => "timed-out"),
    ]);

    if (abort.signal.aborted) {
      break;
    }

    switch (outcome) {
      case "bot-killed":
        ctrl.pause();
        ctrl.openTutorialSlide(8);
        return;

      case "timed-out":
        status.value = "timed-out";

        ctrl.alterUi((ui) => {
          ui.enableUploadBot = true;
          ui.highlightUploadBot = true;
        });

        ctrl.getLocalServer().destroyAllBots();
        break;
    }
  }
});

function handleGoBack() {
  ctrl.alterUi((ui) => {
    ui.enableUploadBot = false;
    ui.highlightUploadBot = false;
  });

  ctrl.openTutorialSlide(5);
}

onUnmounted(() => {
  abort.abort();
});
</script>

<template>
  <template v-if="status == 'awaiting-resume'">
    <p>nice, the bot has been spawned!</p>

    <p>
      on the right side you can now see your bot's id and its other parameters
    </p>

    <p>
      now, press the <kbd>[resume]</kbd> button (top-right corner) to unpause
      the game - if everything goes correctly, we should see the bot driving
      forward and quickly falling out the map!
    </p>
  </template>

  <template v-else-if="status == 'awaiting-kill'">
    <p>observing robot ({{ timer }}s)...</p>
  </template>

  <template v-else>
    <p>ouch, it looks like the robot is still alive!</p>
    <p>make sure you uploaded the correct firmware and try again</p>

    <footer>
      <p>
        (need help?
        <a href="#" @click="handleGoBack()">see the previous slide</a>)
      </p>
    </footer>
  </template>
</template>
