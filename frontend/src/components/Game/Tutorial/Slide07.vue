<script setup lang="ts">
import { ref } from "vue";
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

const status = ref("awaiting-resume");

ctrl.alterUi((ui) => {
  ui.enableDisconnectFromBot = false;
  ui.enablePause = true;
  ui.enableUploadBot = false;
  ui.highlightPause = true;
  ui.highlightUploadBot = false;
  ui.showBotList = false;
});

ctrl.onOnce("server.resume", () => {
  ctrl.alterUi((ui) => {
    ui.enablePause = false;
    ui.highlightPause = false;
  });

  status.value = "awaiting-robot";

  const onceBotIsKilled = new Promise(async (resolve) => {
    const events = await ctrl.getLocalServer().listen();

    for await (const event of events) {
      if (event.ty == "bot-killed") {
        resolve(null);
        break;
      }
    }
  });

  // TODO timeout
  onceBotIsKilled.then(() => {
    ctrl.pause();
    ctrl.openTutorialSlide(8);
  });
});
</script>

<template>
  <template v-if="status == 'awaiting-resume'">
    <p>nice, the bot has been spawned!</p>

    <p>
      on the right side you can see your bot's id (generated randomly), its
      status, serial port output (which allows for the bot to communicate with
      you) and the history
    </p>

    <p>
      now, press the <kbd>[resume]</kbd> button (top-right corner) to unpause
      the game - if everything goes correctly, we should see the bot driving
      forward and eventually falling outside the map!
    </p>
  </template>

  <template v-else>
    <p>observing robot...</p>
  </template>
</template>
