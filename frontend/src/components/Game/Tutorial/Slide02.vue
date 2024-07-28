<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

ctrl.onSlide(2, () => {
  ctrl.on("tutorial.continue.a", () => {
    ctrl.openSlide(3);
  });

  ctrl.on("tutorial.continue.b", () => {
    ctrl.openSlide(4);
  });
});
</script>

<template>
  <p>
    cool! the repository you cloned contains a skeleton project with a simple
    robot already implemented
  </p>

  <p>now:</p>

  <ul>
    <li>
      if you're more into discovering things yourself and you'd like just a
      quick introduction to the interface,

      <button @click="ctrl.emit('tutorial.continue.a')">click here</button>
    </li>

    <p>or</p>

    <li>
      if you're more into step-by-step tutorials,

      <button @click="ctrl.emit('tutorial.continue.b')" class="highlighted">
        be my guest here
      </button>
    </li>
  </ul>
</template>
