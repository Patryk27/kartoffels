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
    first, as compared to usual rust code, implementing a firmware is a bit
    funky - what's most important is that there's no standard library, so you
    can't access <kbd>std::fs</kbd>, use <kbd>println!()</kbd> etc.
  </p>

  <p>
    that's because the game simulates an actual custom robot, with limited CPU
    and RAM, and without access to the external world (besides specific emulated
    peripherals such as the radar)
  </p>

  <p>
    you can use all the language constructs, though: functions, traits etc. all
    work - you can even allocate, just remember to import <kbd>Vec</kbd> from
    the <kbd>alloc</kbd> crate instead of <kbd>std</kbd>
  </p>

  <p style="text-align: right">
    <button @click="ctrl.emit('tutorial.continue')" class="highlighted">
      so far so good
    </button>
  </p>
</template>
