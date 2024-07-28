<script setup lang="ts">
import type { GameCtrl } from "../Ctrl";

const { ctrl } = defineProps<{
  ctrl: GameCtrl;
}>();

ctrl.onSlide(4, () => {
  ctrl.waitFor("tutorial.continue").then(() => {
    ctrl.openSlide(5);
  });
});
</script>

<template>
  <p>
    okie, for the purposes of this tutorial, we'll be starting from scratch -
    open
    <kbd>src/main.rs</kbd> and replace the code there with:
  </p>

  <pre>
#![no_std]
#![no_main]

use kartoffel::*;

#[no_mangle]
fn main() {
    //
}</pre
  >

  <p>this doesn't do anything yet, but baby steps</p>

  <p>
    as compared to usual rust code, implementing a firmware is a bit funky -
    what's most important is that there's no standard library, so you can't
    access <kbd>std::fs</kbd>, use <kbd>println!()</kbd> etc.
  </p>

  <p>
    that's because the game simulates an actual custom robot, with limited CPU
    and RAM, and without access to the external world
  </p>

  <p>
    the only things your code has access to are the <b>peripherals</b>: motor,
    radar and arm
  </p>

  <p style="text-align: right">
    <button @click="ctrl.emit('tutorial.continue')" class="highlighted">
      so far so good
    </button>
  </p>
</template>
