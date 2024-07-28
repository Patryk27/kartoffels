<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

ctrl.onSlide(1, () => {
  ctrl.waitFor("tutorial.continue").then(() => {
    ctrl.openSlide(2);
  });
});
</script>

<template>
  <p>hey there soldier and welcome to the tutorial 🫡🫡🫡</p>

  <p>
    in here we'll go through the basics of programming the robots and navigating
    the game - this will take just a couple of minutes
  </p>

  <p>
    <b>note:</b> if you'd like to see how the game looks like without uploading
    your own robot just yet, simply
    <a href="#" @click="ctrl.emit('tutorial.done')">go back</a>, choose any
    world and play around as long as you want
  </p>

  <p>otherwise:</p>

  <p>
    i'm assuming you know a bit of rust (calling functions, using ifs etc.,
    nothing too advanced) and that you've got <kbd>cargo</kbd> at hand
  </p>

  <p>alrighty - start by following this carefully crafted instruction:</p>

  <pre>
$ git clone https://github.com/patryk27/kartoffel
$ cd kartoffel</pre
  >

  <p>
    ... and then press
    <button class="highlighted" @click="ctrl.emit('tutorial.continue')">
      continue
    </button>
    once you're ready.
  </p>

  <footer>
    <p>
      <b>cool bear's hot tip:</b> you can leave the tutorial at any time by
      using the <kbd>[leave]</kbd> button in the top left corner
    </p>
  </footer>
</template>
