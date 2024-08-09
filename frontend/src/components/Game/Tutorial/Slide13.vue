<script setup lang="ts">
import { ref } from "vue";
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

const server = ctrl.getLocalServer();
const status = ref("intro.1");
const timer = ref(null);

let enemyBotIds = [];

function handleGoToIntro2() {
  status.value = "intro.2";

  prepareMap();
}

function handleStart() {
  status.value = "awaiting-upload";

  ctrl.alterUi((ui) => {
    ui.enableUploadBot = true;
    ui.highlightUploadBot = true;
  });
}

async function prepareMap() {
  enemyBotIds = [];

  const enemies = [
    [18, 16],
    [20, 16],
    [20, 18],
    [20, 20],
    [18, 20],
    [18, 22],
    [16, 22],
    [14, 22],
    [12, 22],
    [12, 20],
    [12, 17],
    [12, 10],
    [11, 10],
    [13, 10],
  ];

  for (const [x, y] of enemies) {
    const enemy = await server.createPrefabBot("dummy", x, y, true);

    enemyBotIds.push(enemy.id);
  }
}

ctrl.on("server.bot-create", async (playerBotId) => {
  ctrl.alterUi((ui) => {
    ui.enableUploadBot = false;
    ui.highlightUploadBot = false;
  });

  status.value = "observing-robot";
  timer.value = 25;

  const oncePlayerIsKilled = ctrl
    .onceBotIsKilled(playerBotId)
    .then((_) => "player-killed");

  const onceEnemiesAreKilled = ctrl
    .onceBotsAreKilled(enemyBotIds)
    .then((_) => "enemies-killed");

  const onceTimeRunsOut = ctrl
    .onceTimerIsCompleted(timer)
    .then((_) => "timed-out");

  const outcome = await Promise.race([
    oncePlayerIsKilled,
    onceEnemiesAreKilled,
    onceTimeRunsOut,
  ]);

  server.destroyAllBots();

  let restartMap = false;

  switch (outcome) {
    case "player-killed":
      status.value = "awaiting-repeat.player-killed";
      restartMap = true;
      break;

    case "enemies-killed":
      status.value = "done";
      break;

    case "timed-out":
      status.value = "awaiting-repeat.timed-out";
      restartMap = true;
      break;
  }

  if (restartMap) {
    prepareMap();

    ctrl.alterUi((ui) => {
      ui.enableUploadBot = true;
      ui.highlightUploadBot = true;
    });
  }
});
</script>

<template>
  <template v-if="status == 'intro.1'">
    <main>
      <p>alright, let's wrap things up with an exercise!</p>

      <p>
        the purpose of a bot is to fight enemies, so how about we spawn a
        couple?
      </p>
    </main>

    <footer style="text-align: right">
      <button class="highlighted" @click="handleGoToIntro2()">yessssss</button>
    </footer>
  </template>

  <template v-else-if="status == 'intro.2'">
    <main>
      <p>lo and behold!</p>
    </main>

    <footer style="text-align: right">
      <button class="highlighted" @click="status = 'intro.3'">
        what now ??
      </button>
    </footer>
  </template>

  <template v-else-if="status == 'intro.3'">
    <main>
      <p>now, my dear, it's time for you to fight... and win, hopefully!</p>

      <p>
        in order to kill a bot, drive right towards it and then call
        <kbd>arm_stab()</kbd>, easy as A B C
      </p>

      <p>
        using all the knowledge gathered so far, implement a bot that uses radar
        to locate the closest enemy (visible as <kbd>'@'</kbd> on the scan) -
        then let the bot drive towards this enemy, stab it, and repeat
      </p>

      <p>
        when no enemy is in sight, let the robot continue driving in its current
        direction
      </p>

      <p>a couple of hints:</p>

      <ul>
        <li>the 5x5 scan will come handy here</li>

        <li>
          for the purposes of this tutorial, the enemies will remain stationary
          and won't try to attack your bot
        </li>

        <li>
          you can debug your code using the
          <kbd>serial_send_str()</kbd> function - the output of serial port is
          visible on the right panel once your bot is spawned
        </li>

        <li>
          if you run out of ideas, feel free to <kbd>git checkout tutorial</kbd>
          to see the solution
        </li>
      </ul>
    </main>

    <footer style="text-align: right">
      <button class="highlighted" @click="handleStart()">
        leeeeeroy jenkins, let's do this!
      </button>
    </footer>
  </template>

  <template v-else-if="status == 'awaiting-upload'">
    <main>
      <p>waiting for you to upload the bot...</p>
    </main>
  </template>

  <template v-else-if="status == 'observing-robot'">
    <main>
      <p>observing robot ({{ timer }}s)...</p>
    </main>
  </template>

  <template v-else-if="status == 'awaiting-repeat.player-killed'">
    <main>
      <p>ouch, your robot fell off the map - try again, best of luck!</p>
    </main>
  </template>

  <template v-else-if="status == 'awaiting-repeat.timed-out'">
    <main>
      <p>ouch, you ran out of time - try again, best of luck!</p>
    </main>
  </template>

  <template v-else-if="status == 'done'">
    <main>
      <p>hey mate, <b class="text-rainbow">you made it!</b></p>

      <p>
        i feel confident in your abilities, you're ready to go and conquer the
        world!
      </p>

      <p>but before you jump into action, note that during actual gameplay:</p>

      <ul class="compact">
        <li>
          your robot will be attacked by other robots (d'oh!), so get creative
        </li>
        <li>
          you will be able to use W/A/S/D or arrow keys to navigate the camera
          and browse around the map
        </li>
        <li>
          you will be able to watch and follow other bots (you obviously can't
          download their firmwares, though)
        </li>
      </ul>

      <p>... and that's it, i think - the rest, you'll figure out!</p>

      <p style="text-align: center">
        have fun!
        <a target="_blank" href="https://github.com/Patryk27/">~pwy</a>
      </p>
    </main>

    <footer style="text-align: right">
      <button class="highlighted" @click="ctrl.emit('tutorial.done')">
        let's play!
      </button>
    </footer>
  </template>
</template>
