<script setup lang="ts">
import { onUnmounted } from "vue";
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

const abort = new AbortController();

ctrl.onTutorialSlide(2, () => {
  ctrl.onOnce("tutorial.continue.a", () => {
    if (abort.signal.aborted) {
      return;
    }

    ctrl.openTutorialSlide(3);
  });

  ctrl.onOnce("tutorial.continue.b", () => {
    if (abort.signal.aborted) {
      return;
    }

    ctrl.openTutorialSlide(4);
  });
});

onUnmounted(() => {
  abort.abort();
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
