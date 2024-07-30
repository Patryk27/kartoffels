<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();
</script>

<template>
  <p>okie, here's a quick introduction:</p>

  <ul>
    <li>
      see <kbd>README.md</kbd> in the repository for building instructions
    </li>

    <li>use the <kbd>[upload bot]</kbd> button to upload the firmware</li>
    <li>navigate map using <kbd>W/A/S/D</kbd> or arrow keys</li>
    <li>bots are represented with the @ char</li>

    <li>
      bots uploaded by you will have
      <span class="inverted">inverted colors</span>
    </li>

    <li>
      use your ide's <kbd>go to definition</kbd> feature to discover how
      robot-specific functions, such as <kbd>radar_scan()</kbd>, work
    </li>
  </ul>

  <p>
    when you press the button below, you'll be redirected back to the home page
    - from there, just choose any world and start playing
  </p>

  <p style="text-align: center">good luck and have fun!</p>

  <footer>
    <button @click="ctrl.openTutorialSlide(4)">
      you know, let's do the full tutorial
    </button>

    <button @click="ctrl.emit('tutorial.done')" class="highlighted">
      let's roll!
    </button>
  </footer>
</template>

<style scoped>
footer {
  display: flex;
  justify-content: space-between;
}
</style>
