<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

ctrl.onSlide(7, () => {
  ctrl.disableButton("uploadBot");
  ctrl.unhighlightButton("uploadBot");
});
</script>

<template>
  <p>nice! -- let's see how your bot's doing:</p>
</template>
