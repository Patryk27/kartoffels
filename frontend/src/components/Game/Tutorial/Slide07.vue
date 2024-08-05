<script setup lang="ts">
import { ref } from "vue";
import type { GameCtrl } from "../Ctrl";
import { onUnmounted } from "vue";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

const abort = new AbortController();
const status = ref("awaiting-resume");
const remainingSeconds = ref(null);

function onceBotIsKilled(): Promise<"bot-killed"> {
  return new Promise(async (resolve) => {
    const events = await ctrl.getLocalServer().listen();

    for await (const event of events) {
      if (event.ty == "bot-killed") {
        resolve("bot-killed");
        break;
      }
    }
  });
}

function onceTimeRunsOut(): Promise<"timed-out"> {
  return new Promise((resolve) => {
    const handle = setInterval(() => {
      remainingSeconds.value -= 1;

      if (remainingSeconds.value == 0) {
        resolve("timed-out");
        clearInterval(handle);
      }
    }, 1000);
  });
}

function onceComponentIsUnmounted(): Promise<"unmounted"> {
  return new Promise((resolve) => {
    onUnmounted(() => {
      resolve("unmounted");
    });
  });
}

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
  remainingSeconds.value = 10;

  ctrl.on("server.bot-upload", () => {
    if (abort.signal.aborted) {
      return;
    }

    status.value = "awaiting-kill";
    remainingSeconds.value = 10;
  });

  while (true) {
    const outcome = await Promise.race([
      onceBotIsKilled(),
      onceTimeRunsOut(),
      onceComponentIsUnmounted(),
    ]);

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

      case "unmounted":
        return;
    }
  }
});

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
    <p>observing robot ({{ remainingSeconds }}s)...</p>
  </template>

  <template v-else>
    <p>whoopsie - it looks like the robot is still alive!</p>
    <p>make sure you uploaded the correct firmware and try again</p>
  </template>
</template>
