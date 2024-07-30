<script setup lang="ts">
import { ref } from "vue";
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

const altDisabled = ref(false);

function handleAlt() {
  alert(
    "listen, i'm not here to judge you, because i know you're still learning - but we *are* going to use the radar",
  );

  altDisabled.value = true;
}

ctrl.onOnce("tutorial.continue", () => {
  ctrl.pause();
  ctrl.openTutorialSlide(9);
});
</script>

<template>
  <p>nice!</p>

  <p>
    i mean, you know, not nice, because we're dead - but relatively speaking
    it's progress
  </p>

  <p>
    the <kbd>.</kbd> tile repesents floor (like in nethack or similar games) -
    the robot can drive over a floor, but once it drives off of it, it'll fall
    into the hell or something and die, standard stuff
  </p>

  <p>now, in order for the robot not to fall, we can use <b>the radar</b></p>

  <footer>
    <button :disabled="altDisabled" @click="handleAlt()">
      no, let's not use the radar
    </button>

    <button @click="ctrl.emit('tutorial.continue')">
      yes, let's use the radar
    </button>
  </footer>
</template>

<style scoped>
footer {
  display: flex;
  justify-content: space-between;
}
</style>
