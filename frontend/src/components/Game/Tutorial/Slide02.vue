<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();
</script>

<template>
  <main>
    <p>
      cool! the repository you cloned contains a skeleton project with a simple
      robot already implemented
    </p>

    <p>now:</p>

    <ul class="compact">
      <li>
        if you're more into discovering things yourself and you'd like just a
        quick introduction to the interface,

        <button @click="ctrl.openTutorialSlide(3)">click here</button>
      </li>

      <p>or</p>

      <li>
        if you're more into step-by-step tutorials,

        <button @click="ctrl.openTutorialSlide(4)" class="highlighted">
          be my guest here
        </button>
      </li>
    </ul>
  </main>
</template>
