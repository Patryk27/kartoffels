<script setup lang="ts">
import { ref } from "vue";
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

ctrl.alterUi((ui) => {
  ui.enableUploadBot = true;
  ui.highlightUploadBot = true;
});

const status = ref("awaiting-first-upload");
const timer = ref(null);

ctrl.on("server.bot-create", async () => {
  ctrl.alterUi((ui) => {
    ui.enableUploadBot = false;
    ui.highlightUploadBot = false;
  });

  status.value = "observing-robot";
  timer.value = 20;

  const server = ctrl.getLocalServer();

  server.createPrefabBot("dummy", 14, 14);

  const outcome = await new Promise(async (resolve, reject) => {
    const playerBotId = (await server.getBots())[0].id;
    let remainingEnemies = 4;

    for await (const botId of await ctrl.listenForKilledBots()) {
      if (botId == playerBotId) {
        resolve("player-got-killed");
        break;
      } else {
        remainingEnemies -= 1;

        if (remainingEnemies == 0) {
          resolve("enemies-got-killed");
        }
      }
    }

    reject();
  });

  alert(outcome);
});
</script>

<template>
  <template v-if="status == 'awaiting-first-upload'">
    <p>alright, let's wrap things up with an exercise!</p>

    <p>
      the purpose of bots is, of course, to fight - and they do so using the
      <kbd>arm_stab()</kbd> function
    </p>

    <p>
      basically, you have to drive towards an enemy-bot, so that you're right
      next to it, and then call <kbd>arm_stab()</kbd> in order to kill it
    </p>

    <p>
      now, using all the knowledge gathered so far, implement a bot that uses
      radar to find the closest enemy (visible as <kbd>'@'</kbd> on the radar
      scan) and then make it drive towards that bot and stab it
    </p>

    <p>a couple of hints:</p>

    <ul>
      <li>using the 3x3 scan will suffice here, but feel free to experiment</li>

      <li>
        for the purposes of this tutorial, the enemy will remain stationary and
        won't try to attack your bot
      </li>

      <li>
        you can debug your code using the <kbd>serial_send_str()</kbd> function
        - the output of serial port is visible on the right panel once your bot
        is spawned
      </li>
    </ul>
  </template>

  <template v-else-if="status == 'observing-robot'">
    <p>observing robot ({{ timer }}s)...</p>
  </template>
</template>
