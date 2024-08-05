<script setup lang="ts">
import { ref } from "vue";
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

const remainingSeconds = ref(20);
</script>

<template>
  <template>
    <p>observing robot ({{ remainingSeconds }}s)...</p>
  </template>
</template>
