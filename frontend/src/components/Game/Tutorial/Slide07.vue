<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

ctrl.onSlide(7, () => {
  ctrl.disableButton("uploadBot");
  ctrl.unhighlightButton("uploadBot");
  ctrl.pause();
});
</script>

<template>
  <p>nice!</p>
</template>
