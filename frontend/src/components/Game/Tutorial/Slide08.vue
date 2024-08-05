<script setup lang="ts">
import { ref } from "vue";
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

const altDisabled = ref(false);

function handleAlt() {
  alert(
    "i'm sure you do buddy -- listen, i'm not here to judge you, because i \
     know you're still learning, but we *are* going to use the radar",
  );

  altDisabled.value = true;
}

ctrl.onOnce("tutorial.continue", () => {
  ctrl.pause();
  ctrl.openTutorialSlide(9);
});
</script>

<template>
  <p class="text-rainbow">nice!</p>

  <p>
    i mean, you know, not nice, because we're dead - but relatively speaking
    it's progress
  </p>

  <p>
    the <kbd>.</kbd> tile repesents floor (like in nethack) - the robot can
    drive over a floor, but once it drives off of it, it falls into the hell or
    something and dies, capiche?
  </p>

  <p>now, in order for the robot not to fall, we can use <b>the radar</b></p>

  <footer>
    <button :disabled="altDisabled" @click="handleAlt()">
      no, i've got a better idea
    </button>

    <button class="highlighted" @click="ctrl.emit('tutorial.continue')">
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
